//! Turns reading data into the SVG card. Pure functions, no I/O.
//!
//! The look follows the Libriax app: Playfair headings, the green
//! Currently Reading bar, and stat cards with raised icon badges.

use serde::Deserialize;

const WIDTH: u32 = 480;
const HEIGHT: u32 = 290;
const BAR_WIDTH: f32 = 240.0;
const STAT_WIDTH: u32 = 140;
const STAT_GAP: u32 = 10;
const LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");

#[derive(Deserialize)]
pub struct Reading {
    pub name: String,
    pub year: i32,
    pub books: u32,
    pub pages: u32,
    pub streak: Option<u32>,
    pub current: Option<Book>,
}

#[derive(Deserialize)]
pub struct Book {
    pub title: String,
    pub author: String,
    /// A URL from the source, swapped for a `data:` URI before rendering.
    pub cover: Option<String>,
    /// (pages read, total pages)
    pub progress: Option<(u32, u32)>,
}

pub struct Theme {
    bg: &'static str,
    border: &'static str,
    text: &'static str,
    muted: &'static str,
    card: &'static str,
    card_border: &'static str,
    badge: &'static str,
    badge_border: &'static str,
    track: &'static str,
}

/// The app's day palette.
static LIGHT: Theme = Theme {
    bg: "#ffffff",
    border: "#eaeaea",
    text: "#192231",
    muted: "#7e8794",
    card: "#ffffff",
    card_border: "#f0f2f5",
    badge: "#ffffff",
    badge_border: "#f0f0f0",
    track: "#eef0f3",
};

/// The app's night palette.
static DARK: Theme = Theme {
    bg: "#10151f",
    border: "#2b3852",
    text: "#f4f6f8",
    muted: "#a8b0bd",
    card: "#192231",
    card_border: "#2b3852",
    badge: "#222d42",
    badge_border: "#2b3852",
    track: "#222d42",
};

impl Theme {
    pub fn named(name: &str) -> &'static Theme {
        match name {
            "dark" => &DARK,
            _ => &LIGHT,
        }
    }
}

/// A Font Awesome Free 6.5.0 solid icon (CC BY 4.0, fontawesome.com),
/// the same ones the app and newsletter use. All are 512 units tall.
struct Icon {
    width: u32,
    path: &'static str,
}

const BOOK_ICON: Icon = Icon {
    width: 448,
    path: "M96 0C43 0 0 43 0 96V416c0 53 43 96 96 96H384h32c17.7 0 32-14.3 32-32s-14.3-32-32-32V384c17.7 0 32-14.3 32-32V32c0-17.7-14.3-32-32-32H384 96zm0 384H352v64H96c-17.7 0-32-14.3-32-32s14.3-32 32-32zm32-240c0-8.8 7.2-16 16-16H336c8.8 0 16 7.2 16 16s-7.2 16-16 16H144c-8.8 0-16-7.2-16-16zm16 48H336c8.8 0 16 7.2 16 16s-7.2 16-16 16H144c-8.8 0-16-7.2-16-16s7.2-16 16-16z",
};

const PAGES_ICON: Icon = Icon {
    width: 384,
    path: "M64 0C28.7 0 0 28.7 0 64V448c0 35.3 28.7 64 64 64H320c35.3 0 64-28.7 64-64V160H256c-17.7 0-32-14.3-32-32V0H64zM256 0V128H384L256 0zM112 256H272c8.8 0 16 7.2 16 16s-7.2 16-16 16H112c-8.8 0-16-7.2-16-16s7.2-16 16-16zm0 64H272c8.8 0 16 7.2 16 16s-7.2 16-16 16H112c-8.8 0-16-7.2-16-16s7.2-16 16-16zm0 64H272c8.8 0 16 7.2 16 16s-7.2 16-16 16H112c-8.8 0-16-7.2-16-16s7.2-16 16-16z",
};

const FIRE_ICON: Icon = Icon {
    width: 448,
    path: "M159.3 5.4c7.8-7.3 19.9-7.2 27.7 .1c27.6 25.9 53.5 53.8 77.7 84c11-14.4 23.5-30.1 37-42.9c7.9-7.4 20.1-7.4 28 .1c34.6 33 63.9 76.6 84.5 118c20.3 40.8 33.8 82.5 33.8 111.9C448 404.2 348.2 512 224 512C98.4 512 0 404.1 0 276.5c0-38.4 17.8-85.3 45.4-131.7C73.3 97.7 112.7 48.6 159.3 5.4zM225.7 416c25.3 0 47.7-7 68.8-21c42.1-29.4 53.4-88.2 28.1-134.4c-4.5-9-16-9.6-22.5-2l-25.2 29.3c-6.6 7.6-18.5 7.4-24.7-.5c-16.5-21-46-58.5-62.8-79.8c-6.3-8-18.3-8.1-24.7-.1c-33.8 42.5-50.8 69.3-50.8 99.4C112 375.4 162.6 416 225.7 416z",
};

pub fn render(reading: &Reading, theme: &Theme) -> String {
    // Colours match the newsletter's Books, Pages and Day streak cards.
    let stats: Vec<_> = [
        Some((&BOOK_ICON, "#1e63ff", reading.books, format!("{} in {}", plural(reading.books, "book"), reading.year))),
        Some((&PAGES_ICON, "#9b7cff", reading.pages, plural(reading.pages, "page"))),
        reading.streak.map(|days| (&FIRE_ICON, "#ee5253", days, "day streak".to_owned())),
    ]
    .into_iter()
    .flatten()
    .collect();

    let count = stats.len() as u32;
    let start = (WIDTH - (count * STAT_WIDTH + (count - 1) * STAT_GAP)) / 2;
    let cards: String = stats
        .iter()
        .enumerate()
        .map(|(i, (icon, colour, value, label))| {
            stat_card(start + i as u32 * (STAT_WIDTH + STAT_GAP), icon, colour, *value, label)
        })
        .collect();

    let body = format!(
        "{}{}{cards}",
        header(&format!("{}'s reading", reading.name)),
        book_block(reading.current.as_ref()),
    );
    frame(theme, &describe(reading), &body)
}

/// A card that still renders in the README when there's nothing to show.
pub fn render_message(message: &str, theme: &Theme) -> String {
    let body = format!(
        r#"{}<text x="{}" y="170" class="title muted" text-anchor="middle">{}</text>"#,
        header("Reading card"),
        WIDTH / 2,
        esc(message),
    );
    frame(theme, message, &body)
}

fn frame(theme: &Theme, description: &str, body: &str) -> String {
    let Theme { bg, border, text, muted, card, card_border, badge, badge_border, track } = theme;
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}" role="img" aria-labelledby="desc"><title id="desc">{}</title><defs><clipPath id="cover"><rect x="20" y="64" width="60" height="90" rx="3"/></clipPath><clipPath id="logo"><rect x="434" y="20" width="26" height="26" rx="6"/></clipPath><filter id="lift" x="-30%" y="-30%" width="160%" height="170%"><feDropShadow dx="0" dy="3" stdDeviation="3" flood-opacity=".08"/></filter><filter id="shelf" x="-40%" y="-30%" width="190%" height="170%"><feDropShadow dx="3" dy="5" stdDeviation="6" flood-opacity=".22"/></filter></defs><style>text{{font-family:Inter,-apple-system,BlinkMacSystemFont,'Segoe UI',Helvetica,Arial,sans-serif;fill:{text}}}.heading,.brand{{font-family:'Playfair Display',Georgia,'Times New Roman',serif;font-weight:700}}.heading{{font-size:20px}}.brand{{font-size:15px}}.label,.stat-label{{font-size:10px;font-weight:600;letter-spacing:.5px}}.title{{font-size:16px;font-weight:700}}.author{{font-size:13px}}.small{{font-size:11px;font-weight:600}}.value{{font-size:20px;font-weight:800}}.label,.stat-label,.author,.small,.muted{{fill:{muted}}}.card{{fill:{card};stroke:{card_border}}}.badge{{fill:{badge};stroke:{badge_border};stroke-width:1.5}}.track{{fill:{track}}}</style><rect x="0.5" y="0.5" width="{}" height="{}" rx="14" fill="{bg}" stroke="{border}"/>{body}</svg>"#,
        esc(description),
        WIDTH - 1,
        HEIGHT - 1,
    )
}

fn header(title: &str) -> String {
    format!(
        r#"<text x="20" y="40" class="heading">{}</text><text x="426" y="38" class="brand" text-anchor="end">Libriax</text><image href="data:image/png;base64,{}" x="434" y="20" width="26" height="26" clip-path="url(#logo)"/>"#,
        esc(&fit(title, 300.0, 20.0)),
        crate::base64(LOGO_PNG),
    )
}

fn book_block(book: Option<&Book>) -> String {
    let label = r#"<text x="96" y="78" class="label">CURRENTLY READING</text>"#;
    let Some(book) = book else {
        return format!(
            r#"<rect x="20" y="64" width="60" height="90" rx="3" class="track"/>{label}<text x="96" y="104" class="title muted">Between books</text>"#
        );
    };

    let cover = book.cover.as_deref().map_or(String::new(), |uri| {
        format!(
            r#"<image href="{}" x="20" y="64" width="60" height="90" preserveAspectRatio="xMidYMid slice" clip-path="url(#cover)"/>"#,
            esc(uri),
        )
    });
    let progress = match book.progress {
        Some((read, total)) if total > 0 => {
            let read = read.min(total);
            let fraction = read as f32 / total as f32;
            format!(
                r#"<rect x="96" y="136" width="{BAR_WIDTH}" height="8" rx="4" class="track"/><rect x="96" y="136" width="{:.1}" height="8" rx="4" fill="{}"/><text x="{}" y="143" class="small">{} / {} pages</text>"#,
                BAR_WIDTH * fraction,
                progress_colour(fraction),
                96.0 + BAR_WIDTH + 10.0,
                thousands(read),
                thousands(total),
            )
        }
        _ => String::new(),
    };
    format!(
        r#"<g filter="url(#shelf)"><rect x="20" y="64" width="60" height="90" rx="3" class="track"/>{cover}</g>{label}<text x="96" y="102" class="title">{}</text><text x="96" y="121" class="author">{}</text>{progress}"#,
        esc(&fit(&book.title, 364.0, 16.0)),
        esc(&fit(&book.author, 364.0, 13.0)),
    )
}

/// A stat card with the app's raised icon badge: a white tile whose
/// coloured bottom edge is the stat's colour.
fn stat_card(x: u32, icon: &Icon, colour: &str, value: u32, label: &str) -> String {
    let centre = x + STAT_WIDTH / 2;
    let badge_x = centre - 17;
    let icon_width = icon.width as f32 * 15.0 / 512.0;
    format!(
        r#"<rect x="{x}" y="172" width="{STAT_WIDTH}" height="100" rx="12" class="card"/><g filter="url(#lift)"><rect x="{badge_x}" y="184" width="34" height="34" rx="9" fill="{colour}"/><rect x="{}" y="184.75" width="32.5" height="29" rx="8" class="badge"/></g><svg x="{:.2}" y="191.75" width="{icon_width:.2}" height="15" viewBox="0 0 {} 512"><path fill="{colour}" d="{}"/></svg><text x="{centre}" y="244" class="value" text-anchor="middle">{}</text><text x="{centre}" y="262" class="stat-label" text-anchor="middle">{}</text>"#,
        badge_x as f32 + 0.75,
        centre as f32 - icon_width / 2.0,
        icon.width,
        icon.path,
        thousands(value),
        esc(&label.to_uppercase()),
    )
}

/// The same green ramp as the app's Currently Reading bars
/// (crProgressColor): pale at the start, deep near the end.
fn progress_colour(fraction: f32) -> String {
    format!("hsl(142, {:.0}%, {:.0}%)", 45.0 + fraction * 20.0, 78.0 - fraction * 52.0)
}

/// The alt text: screen readers get the whole card as one sentence.
fn describe(reading: &Reading) -> String {
    let current = reading.current.as_ref().map_or("between books".to_owned(), |book| {
        format!("reading {} by {}", book.title, book.author)
    });
    let streak = reading.streak.map_or(String::new(), |days| format!(", {days}-day streak"));
    format!(
        "{} is {current}. {} {} and {} {} in {}{streak}.",
        reading.name,
        thousands(reading.books),
        plural(reading.books, "book"),
        thousands(reading.pages),
        plural(reading.pages, "page"),
        reading.year,
    )
}

/// An SVG inside <img> can't measure its own text, so width is estimated
/// from an average glyph and long text is cut with an ellipsis.
// shortcut: assumes Latin-width glyphs, so CJK titles can overflow; measure per script if that bites.
fn fit(text: &str, max_px: f32, font_px: f32) -> String {
    let max_chars = (max_px / (font_px * 0.56)) as usize;
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let cut: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{}…", cut.trim_end())
}

fn esc(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            // One stray control character makes the whole SVG invalid XML.
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out
}

fn plural(n: u32, word: &str) -> String {
    if n == 1 { word.to_owned() } else { format!("{word}s") }
}

fn thousands(n: u32) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(streak: Option<u32>) -> Reading {
        Reading {
            name: "Joe".into(),
            year: 2026,
            books: 1,
            pages: 12345,
            streak,
            current: Some(Book {
                title: "Kafka & <Me>".into(),
                author: "A. N. Author".into(),
                cover: None,
                progress: Some((500, 400)),
            }),
        }
    }

    #[test]
    fn numbers_get_separators() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1000), "1,000");
        assert_eq!(thousands(1234567), "1,234,567");
    }

    #[test]
    fn long_text_is_cut_short() {
        assert_eq!(fit("Dune", 364.0, 16.0), "Dune");
        let long = fit(&"word ".repeat(30), 364.0, 16.0);
        assert!(long.ends_with('…') && long.chars().count() <= 40);
    }

    #[test]
    fn markup_is_escaped_and_controls_dropped() {
        assert_eq!(esc("a<b>&\"c'\u{0}"), "a&lt;b&gt;&amp;&quot;c&#39;");
    }

    #[test]
    fn progress_matches_the_app_ramp() {
        assert_eq!(progress_colour(0.0), "hsl(142, 45%, 78%)");
        assert_eq!(progress_colour(1.0), "hsl(142, 65%, 26%)");
    }

    #[test]
    fn card_shows_escaped_book_and_capped_progress() {
        let svg = render(&sample(Some(9)), Theme::named("dark"));
        assert!(svg.contains("Kafka &amp; &lt;Me&gt;"));
        assert!(svg.contains("400 / 400 pages"));
        assert!(svg.contains(">1</text>") && svg.contains("BOOK IN 2026"));
        assert!(svg.contains(">12,345</text>") && svg.contains(">PAGES<"));
        assert!(svg.contains("DAY STREAK"));
        assert!(svg.contains("1 book and 12,345 pages in 2026, 9-day streak."));
        assert!(svg.contains(DARK.bg));
    }

    #[test]
    fn two_stats_are_centred_when_streak_is_unknown() {
        let svg = render(&sample(None), Theme::named("light"));
        assert!(!svg.contains("DAY STREAK"));
        assert!(svg.contains(r#"<rect x="95" y="172""#));
    }

    #[test]
    fn empty_shelf_says_between_books() {
        let mut reading = sample(None);
        reading.current = None;
        let svg = render(&reading, Theme::named(""));
        assert!(svg.contains("Between books"));
        assert!(svg.contains("Joe is between books."));
    }
}
