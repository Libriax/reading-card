//! A Cloudflare Worker that renders reading cards for GitHub READMEs.
//!
//! `/libriax/<username>` and `/goodreads/<user id>` return an SVG card;
//! `?theme=dark` switches to the night palette. Anything else redirects
//! to the repo.

pub mod card;
mod goodreads;

use std::collections::HashMap;

use worker::*;

use card::{Reading, Theme};

const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");
/// How long GitHub's image proxy, and our upstream fetches, reuse a result.
const CACHE_SECONDS: i32 = 1800;
const RETRY_SECONDS: i32 = 300;
/// Anything bigger isn't a thumbnail, and would bloat every README view.
const MAX_COVER_BYTES: usize = 200_000;
const COVER_HOSTS: [&str; 4] = [
    "books.google.com",
    "books.googleusercontent.com",
    "i.gr-assets.com",
    "images.gr-assets.com",
];
const COVER_TYPES: [&str; 4] = ["image/jpeg", "image/png", "image/webp", "image/gif"];
/// The card contains no script and loads nothing, so if someone opens the
/// SVG directly, anything that sneaks in still can't run or phone home.
const CSP: &str = "default-src 'none'; img-src data:; style-src 'unsafe-inline'";

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    let theme_name = url.query_pairs().find(|(key, _)| key == "theme").map(|(_, name)| name);
    let theme = Theme::named(&theme_name.unwrap_or_default());

    let segments: Vec<&str> = url.path_segments().map(Iterator::collect).unwrap_or_default();
    let reading = match segments.as_slice() {
        ["libriax", user] if is_libriax_username(user) => libriax(&env, user).await,
        ["goodreads", id] if is_goodreads_id(id) => goodreads(id).await,
        _ => return Response::redirect(Url::parse(REPO_URL)?),
    };

    let (svg, max_age) = match reading {
        Ok(Some(mut reading)) => {
            embed_cover(&mut reading).await;
            (card::render(&reading, theme), CACHE_SECONDS)
        }
        Ok(None) => (card::render_message("No public reading card here yet", theme), RETRY_SECONDS),
        Err(err) => {
            console_error!("{err}");
            (card::render_message("Couldn't load reading data", theme), RETRY_SECONDS)
        }
    };
    Ok(Response::builder()
        .with_header("content-type", "image/svg+xml; charset=utf-8")?
        .with_header("cache-control", &format!("public, max-age={max_age}"))?
        .with_header("content-security-policy", CSP)?
        .with_header("x-content-type-options", "nosniff")?
        .fixed(svg.into_bytes()))
}

async fn libriax(env: &Env, user: &str) -> Result<Option<Reading>> {
    let base = env.var("LIBRIAX_URL")?.to_string();
    match get(&format!("{base}/api/card/{user}")).await? {
        Some(mut res) => Ok(Some(res.json().await?)),
        None => Ok(None),
    }
}

async fn goodreads(id: &str) -> Result<Option<Reading>> {
    let current = get(&goodreads::current_feed(id)).await?;
    let read = get(&goodreads::read_feed(id)).await?;
    let (Some(mut current), Some(mut read)) = (current, read) else {
        return Ok(None);
    };
    let year = js_sys::Date::new_0().get_full_year() as i32;
    Ok(goodreads::parse(&current.text().await?, &read.text().await?, year))
}

/// An SVG shown through <img> can't load anything external, so the cover
/// has to travel inside it. A cover that can't be fetched is just left out.
async fn embed_cover(reading: &mut Reading) {
    let Some(book) = reading.shown_mut() else { return };
    book.cover = match book.cover.as_deref() {
        Some(url) => data_uri(url).await,
        None => None,
    };
}

async fn data_uri(url: &str) -> Option<String> {
    let mut url = Url::parse(url).ok()?;
    if !matches!(url.scheme(), "http" | "https") || !COVER_HOSTS.contains(&url.host_str()?) {
        return None;
    }
    // Google Books hands out http:// thumbnail links; both hosts serve https.
    url.set_scheme("https").ok()?;
    let mut res = get(url.as_str()).await.ok()??;
    let headers = res.headers();
    let mime = headers.get("content-type").ok()??;
    let mime = mime.split(';').next()?.trim().to_ascii_lowercase();
    let declared_size: usize = headers.get("content-length").ok()??.parse().ok()?;
    // Checked before downloading, so a huge file is never read into memory.
    if !COVER_TYPES.contains(&mime.as_str()) || declared_size > MAX_COVER_BYTES {
        return None;
    }
    let bytes = res.bytes().await.ok()?;
    (bytes.len() <= MAX_COVER_BYTES).then(|| format!("data:{mime};base64,{}", base64(&bytes)))
}

/// Fetches through Cloudflare's edge cache. A 403 or 404 means there's
/// nothing to show rather than a failure, so it comes back as None.
async fn get(url: &str) -> Result<Option<Response>> {
    let headers = Headers::new();
    headers.set("user-agent", concat!("libriax-card (+", env!("CARGO_PKG_REPOSITORY"), ")"))?;
    let mut init = RequestInit::new();
    // None of the upstreams redirect, so a 3xx is never followed and lands in
    // the error arm below. That keeps the cover host allowlist honest: an
    // allowed host can't bounce the Worker elsewhere. (Workers has no "error"
    // redirect mode.)
    init.with_redirect(RequestRedirect::Manual);
    init.with_headers(headers).with_cf_properties(CfProperties {
        cache_everything: Some(true),
        // Only successes are kept, so a card made public a minute ago shows up.
        cache_ttl_by_status: Some(HashMap::from([
            ("200-299".to_owned(), CACHE_SECONDS),
            ("300-599".to_owned(), 0),
        ])),
        ..CfProperties::default()
    });
    let res = Fetch::Request(Request::new_with_init(url, &init)?).send().await?;
    match res.status_code() {
        200..=299 => Ok(Some(res)),
        403 | 404 => Ok(None),
        status => Err(Error::RustError(format!("{url} responded {status}"))),
    }
}

/// Mirrors Libriax's signup rule, so nothing else reaches the upstream URL.
fn is_libriax_username(name: &str) -> bool {
    (3..=32).contains(&name.len()) && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

fn is_goodreads_id(id: &str) -> bool {
    (1..=12).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_digit())
}

pub fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        // Pack up to three bytes into the top of a 24-bit group, then read it as four 6-bit digits.
        let group = chunk.iter().fold(0u32, |acc, &b| (acc << 8) | u32::from(b)) << (8 * (3 - chunk.len()));
        for i in 0..4 {
            out.push(if i <= chunk.len() {
                ALPHABET[((group >> (18 - 6 * i)) & 63) as usize] as char
            } else {
                '='
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_rfc_4648_vectors() {
        let cases = [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ];
        for (input, expected) in cases {
            assert_eq!(base64(input.as_bytes()), expected);
        }
        assert_eq!(base64(&[0xff, 0xfe]), "//4=");
    }

    #[test]
    fn only_safe_names_reach_upstream() {
        assert!(is_libriax_username("joe_speed"));
        assert!(!is_libriax_username("jo"));
        assert!(!is_libriax_username("../admin"));
        assert!(is_goodreads_id("12345678"));
        assert!(!is_goodreads_id("123-joe"));
    }
}
