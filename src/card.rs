//! Turns reading data into the SVG card. Pure functions, no I/O.

use serde::Deserialize;

const WIDTH: u32 = 460;
const HEIGHT: u32 = 190;
const BAR_WIDTH: f32 = 220.0;

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
    accent: &'static str,
    track: &'static str,
}

static LIGHT: Theme = Theme {
    bg: "#ffffff",
    border: "#e4e2dd",
    text: "#1f2b46",
    muted: "#6b7280",
    accent: "#8c6a3d",
    track: "#ece9e4",
};

static DARK: Theme = Theme {
    bg: "#0d1117",
    border: "#30363d",
    text: "#e6edf3",
    muted: "#8b949e",
    accent: "#cfaa6e",
    track: "#21262d",
};

static LIBRIAX: Theme = Theme {
    bg: "#192231",
    border: "#2b3852",
    text: "#f4f6f8",
    muted: "#a9b2c3",
    accent: "#cfaa6e",
    track: "#222d42",
};

impl Theme {
    pub fn named(name: &str) -> &'static Theme {
        match name {
            "dark" => &DARK,
            "libriax" => &LIBRIAX,
            _ => &LIGHT,
        }
    }
}

pub fn render(reading: &Reading, theme: &Theme) -> String {
    let stats: String = [
        Some((reading.books, format!("{} in {}", plural(reading.books, "book"), reading.year))),
        Some((reading.pages, plural(reading.pages, "page"))),
        reading.streak.map(|days| (days, "day streak".to_owned())),
    ]
    .into_iter()
    .flatten()
    .enumerate()
    .map(|(column, (value, label))| {
        format!(
            r#"<text x="{}" y="174" class="stat"><tspan class="num">{}</tspan> {label}</text>"#,
            20 + column * 150,
            thousands(value),
        )
    })
    .collect();

    let body = format!(
        r#"{}{}<line x1="20" y1="146" x2="440" y2="146" class="rule"/>{stats}"#,
        header(&format!("{}'s reading", reading.name)),
        book_block(reading.current.as_ref()),
    );
    frame(theme, &describe(reading), &body)
}

/// A card that still renders in the README when there's nothing to show.
pub fn render_message(message: &str, theme: &Theme) -> String {
    let body = format!(
        r#"{}<text x="230" y="105" class="title muted" text-anchor="middle">{}</text>"#,
        header("Reading card"),
        esc(message),
    );
    frame(theme, message, &body)
}

fn frame(theme: &Theme, description: &str, body: &str) -> String {
    let Theme { bg, border, text, muted, accent, track } = theme;
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}" role="img" aria-labelledby="desc"><title id="desc">{}</title><style>text{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Helvetica,Arial,sans-serif;fill:{text}}}.head{{font-size:14px;font-weight:600}}.brand{{font-size:12px;font-weight:700;letter-spacing:.04em;fill:{accent}}}.label{{font-size:10px;font-weight:600;letter-spacing:.08em}}.title{{font-size:16px;font-weight:600}}.author,.small,.stat{{font-size:13px}}.small{{font-size:11px}}.label,.author,.small,.stat,.muted{{fill:{muted}}}.num{{font-size:15px;font-weight:700;fill:{text}}}.track{{fill:{track}}}.fill{{fill:{accent}}}.rule{{stroke:{border}}}</style><clipPath id="cover"><rect x="20" y="44" width="56" height="84" rx="4"/></clipPath><rect x="0.5" y="0.5" width="{}" height="{}" rx="10" fill="{bg}" stroke="{border}"/>{body}</svg>"#,
        esc(description),
        WIDTH - 1,
        HEIGHT - 1,
    )
}

fn header(title: &str) -> String {
    format!(
        r#"<text x="20" y="30" class="head">{}</text><text x="440" y="30" class="brand" text-anchor="end">libriax</text>"#,
        esc(&fit(title, 320.0, 14.0)),
    )
}

fn book_block(book: Option<&Book>) -> String {
    let slot = r#"<rect x="20" y="44" width="56" height="84" rx="4" class="track"/><text x="92" y="60" class="label">CURRENTLY READING</text>"#;
    let Some(book) = book else {
        return format!(r#"{slot}<text x="92" y="84" class="title muted">Between books</text>"#);
    };

    let cover = book.cover.as_deref().map_or(String::new(), |uri| {
        format!(
            r#"<image href="{}" x="20" y="44" width="56" height="84" preserveAspectRatio="xMidYMid slice" clip-path="url(#cover)"/>"#,
            esc(uri),
        )
    });
    let progress = match book.progress {
        Some((read, total)) if total > 0 => {
            let read = read.min(total);
            format!(
                r#"<rect x="92" y="113" width="{BAR_WIDTH}" height="6" rx="3" class="track"/><rect x="92" y="113" width="{:.1}" height="6" rx="3" class="fill"/><text x="322" y="119" class="small">{} / {} pages</text>"#,
                BAR_WIDTH * read as f32 / total as f32,
                thousands(read),
                thousands(total),
            )
        }
        _ => String::new(),
    };
    format!(
        r#"{slot}{cover}<text x="92" y="82" class="title">{}</text><text x="92" y="101" class="author">{}</text>{progress}"#,
        esc(&fit(&book.title, 348.0, 16.0)),
        esc(&fit(&book.author, 348.0, 13.0)),
    )
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
        assert_eq!(fit("Dune", 348.0, 16.0), "Dune");
        let long = fit(&"word ".repeat(30), 348.0, 16.0);
        assert!(long.ends_with('…') && long.chars().count() <= 38);
    }

    #[test]
    fn markup_is_escaped_and_controls_dropped() {
        assert_eq!(esc("a<b>&\"c'\u{0}"), "a&lt;b&gt;&amp;&quot;c&#39;");
    }

    #[test]
    fn card_shows_escaped_book_and_capped_progress() {
        let svg = render(&sample(Some(9)), Theme::named("dark"));
        assert!(svg.contains("Kafka &amp; &lt;Me&gt;"));
        assert!(svg.contains("400 / 400 pages"));
        assert!(svg.contains(">1</tspan> book in 2026"));
        assert!(svg.contains(">12,345</tspan> pages"));
        assert!(svg.contains("1 book and 12,345 pages in 2026, 9-day streak."));
        assert!(svg.contains("day streak"));
        assert!(svg.contains(DARK.bg));
    }

    #[test]
    fn streak_column_is_dropped_when_unknown() {
        let svg = render(&sample(None), Theme::named("light"));
        assert!(!svg.contains("day streak"));
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
