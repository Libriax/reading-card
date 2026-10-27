//! Reads a public Goodreads profile from its shelf RSS feeds.

use std::borrow::Cow;

use crate::card::{Book, Reading};

const FEED: &str = "https://www.goodreads.com/review/list_rss";

pub fn current_feed(id: &str) -> String {
    format!("{FEED}/{id}?shelf=currently-reading&per_page=1")
}

// shortcut: counts at most 200 books read this year; page through the feed if anyone reads more.
pub fn read_feed(id: &str) -> String {
    format!("{FEED}/{id}?shelf=read&sort=date_read&order=d&per_page=200")
}

/// None when the feed isn't a bookshelf, which is what a private or
/// missing profile returns.
pub fn parse(current: &str, read: &str, year: i32) -> Option<Reading> {
    // The channel title reads "Otis 's bookshelf: read".
    let name = text(read, "title")?.split_once("'s bookshelf")?.0.trim().to_owned();

    let current = items(current).next().map(book);
    // The read feed is sorted newest first, so its top item is the last book finished.
    let last_read = items(read).next().map(book);

    let (books, pages) = items(read)
        .filter(|item| read_year(item) == Some(year))
        .fold((0, 0), |(books, pages), item| {
            let item_pages = text(item, "num_pages").and_then(|n| n.parse().ok());
            (books + 1, pages + item_pages.unwrap_or(0))
        });

    Some(Reading { name, year, books, pages, streak: None, current, last_read })
}

fn book(item: &str) -> Book {
    Book {
        title: text(item, "title").unwrap_or_default().into_owned(),
        author: text(item, "author_name").unwrap_or_default().into_owned(),
        cover: text(item, "book_large_image_url")
            .filter(|url| !url.contains("nophoto"))
            .map(Cow::into_owned),
        progress: None,
    }
}

fn items(feed: &str) -> impl Iterator<Item = &str> {
    feed.split("<item>").skip(1)
}

fn read_year(item: &str) -> Option<i32> {
    // RFC 2822, e.g. "Tue, 18 Aug 2026 00:00:00 +0000"
    text(item, "user_read_at")?.split_whitespace().nth(3)?.parse().ok()
}

/// The contents of the first `<name>` element. Goodreads wraps some fields
/// in CDATA and entity-escapes the rest, so both are handled.
fn text<'a>(xml: &'a str, name: &str) -> Option<Cow<'a, str>> {
    let open = format!("<{name}>");
    let start = xml.find(&open)? + open.len();
    let end = start + xml[start..].find(&format!("</{name}>"))?;
    let raw = xml[start..end].trim();
    Some(match raw.strip_prefix("<![CDATA[").and_then(|s| s.strip_suffix("]]>")) {
        Some(cdata) => Cow::Borrowed(cdata),
        None => unescape(raw),
    })
}

fn unescape(text: &str) -> Cow<'_, str> {
    if !text.contains('&') {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];
        match rest.find(';').and_then(|semi| Some((entity(&rest[1..semi])?, semi + 1))) {
            Some((c, len)) => {
                out.push(c);
                rest = &rest[len..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    Cow::Owned(out)
}

fn entity(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => {
            let number = name.strip_prefix('#')?;
            let code = match number.strip_prefix('x') {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => number.parse().ok()?,
            };
            char::from_u32(code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURRENT: &str = "<rss><channel><title>Otis 's bookshelf: currently-reading</title>
        <item><title>Galileo&apos;s Dream</title><author_name>Kim Stanley Robinson</author_name>
        <book_large_image_url><![CDATA[https://i.gr-assets.com/x.jpg]]></book_large_image_url></item>
        </channel></rss>";

    const READ: &str = "<rss><channel><title>Otis 's bookshelf: read</title>
        <item><title>A</title><book><num_pages>528</num_pages></book>
          <user_read_at><![CDATA[Tue, 18 Aug 2026 00:00:00 +0000]]></user_read_at></item>
        <item><title>B</title><book><num_pages></num_pages></book>
          <user_read_at><![CDATA[Mon, 7 Sep 2026 00:00:00 +0000]]></user_read_at></item>
        <item><title>C</title><book><num_pages>300</num_pages></book>
          <user_read_at><![CDATA[Tue, 4 Nov 2025 00:00:00 +0000]]></user_read_at></item>
        <item><title>D</title><book><num_pages>100</num_pages></book>
          <user_read_at><![CDATA[]]></user_read_at></item>
        </channel></rss>";

    #[test]
    fn counts_only_this_years_books() {
        let reading = parse(CURRENT, READ, 2026).unwrap();
        assert_eq!(reading.name, "Otis");
        assert_eq!((reading.books, reading.pages), (2, 528));
        assert_eq!(reading.streak, None);
        let book = reading.current.unwrap();
        assert_eq!(book.title, "Galileo's Dream");
        assert_eq!(book.cover.as_deref(), Some("https://i.gr-assets.com/x.jpg"));
        assert_eq!(reading.last_read.unwrap().title, "A");
    }

    #[test]
    fn empty_currently_reading_shelf() {
        let current = "<rss><channel><title>Otis 's bookshelf: currently-reading</title></channel></rss>";
        assert!(parse(current, READ, 2026).unwrap().current.is_none());
    }

    #[test]
    fn non_feed_pages_are_rejected() {
        let login = "<html><title>Sign in</title></html>";
        assert!(parse(login, login, 2026).is_none());
    }

    #[test]
    fn entities_decode() {
        assert_eq!(unescape("Tom &amp; Jerry &#8217;s &#x41; &bogus; & x"), "Tom & Jerry \u{2019}s A &bogus; & x");
        assert!(matches!(unescape("plain"), Cow::Borrowed("plain")));
    }
}
