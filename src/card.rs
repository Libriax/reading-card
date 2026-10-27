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
    /// Shown instead when nothing is being read right now.
    #[serde(default)]
    pub last_read: Option<Book>,
}

impl Reading {
    /// The book on the card, with its label.
    fn shown(&self) -> Option<(&Book, &'static str)> {
        match (&self.current, &self.last_read) {
            (Some(book), _) => Some((book, "CURRENTLY READING")),
            (None, Some(book)) => Some((book, "LAST READ")),
            (None, None) => None,
        }
    }

    pub fn shown_mut(&mut self) -> Option<&mut Book> {
        self.current.as_mut().or(self.last_read.as_mut())
    }
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
        book_block(reading.shown()),
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
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}" role="img" aria-labelledby="desc"><title id="desc">{}</title><defs><clipPath id="cover"><rect x="20" y="64" width="60" height="90" rx="3"/></clipPath><clipPath id="logo"><rect x="434" y="20" width="26" height="26" rx="6"/></clipPath><filter id="lift" x="-30%" y="-30%" width="160%" height="170%"><feDropShadow dx="0" dy="3" stdDeviation="3" flood-opacity=".08"/></filter><filter id="shelf" x="-40%" y="-30%" width="190%" height="170%"><feDropShadow dx="3" dy="5" stdDeviation="6" flood-opacity=".22"/></filter></defs><style>text{{font-family:Inter,-apple-system,BlinkMacSystemFont,'Segoe UI',Helvetica,Arial,sans-serif;fill:{text}}}.heading,.brand{{font-family:'Playfair Display',Georgia,'Times New Roman',serif;font-weight:700}}.heading{{font-size:20px}}.brand{{font-size:15px}}.label,.stat-label{{font-size:10px;font-weight:600;letter-spacing:.5px}}.title{{font-size:16px;font-weight:700}}.author{{font-size:13px}}.small{{font-size:11px;font-weight:600}}.value{{font-size:20px;font-weight:800}}.label,.stat-label,.author,.small,.muted{{fill:{muted}}}.card{{fill:{card};stroke:{card_border}}}.badge{{fill:{badge};stroke:{badge_border};stroke-width:1.5}}.track{{fill:{track}}}.cover-title{{font-size:8px;font-weight:700;fill:#fff}}.cover-author{{font-size:6.5px;font-weight:500;fill:#fff;fill-opacity:.85}}</style><rect x="0.5" y="0.5" width="{}" height="{}" rx="14" fill="{bg}" stroke="{border}"/>{body}</svg>"#,
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

fn book_block(shown: Option<(&Book, &str)>) -> String {
    let Some((book, label)) = shown else {
        return r#"<rect x="20" y="64" width="60" height="90" rx="3" class="track"/><text x="96" y="78" class="label">CURRENTLY READING</text><text x="96" y="104" class="title muted">Between books</text>"#.to_owned();
    };

    let cover = match book.cover.as_deref() {
        Some(uri) => format!(
            r#"<image href="{}" x="20" y="64" width="60" height="90" preserveAspectRatio="xMidYMid slice" clip-path="url(#cover)"/>"#,
            esc(uri),
        ),
        None => fallback_cover(book),
    };
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
        r#"<g filter="url(#shelf)"><rect x="20" y="64" width="60" height="90" rx="3" class="track"/>{cover}</g><text x="96" y="78" class="label">{label}</text><text x="96" y="102" class="title">{}</text><text x="96" y="121" class="author">{}</text>{progress}"#,
        esc(&fit(&book.title, 364.0, 16.0)),
        esc(&fit(&book.author, 364.0, 13.0)),
    )
}

/// The app's fallback for books without artwork, as on the newsletter
/// shelf: a coloured book with a darker spine and the title printed on it.
/// The colour comes from the title, so a book always gets the same one.
fn fallback_cover(book: &Book) -> String {
    const COLOURS: [&str; 8] =
        ["#4a6cf7", "#0fb9b1", "#a55eea", "#f5a623", "#20bf6b", "#ee5253", "#2d98da", "#8854d0"];
    let hash = book.title.bytes().fold(0usize, |h, b| h.wrapping_mul(31).wrapping_add(b.into()));
    let colour = COLOURS[hash % COLOURS.len()];
    let title: String = wrap(&book.title, 10, 5)
        .iter()
        .enumerate()
        .map(|(i, line)| {
            format!(r#"<text x="31" y="{}" class="cover-title">{}</text>"#, 79 + i * 10, esc(line))
        })
        .collect();
    format!(
        r#"<g clip-path="url(#cover)"><rect x="20" y="64" width="60" height="90" fill="{colour}"/><rect x="20" y="64" width="7" height="90" fill="black" fill-opacity=".2"/>{title}<text x="31" y="147" class="cover-author">{}</text></g>"#,
        esc(&fit(&book.author, 46.0, 6.5)),
    )
}

/// Greedy word wrap into lines of at most `width` characters. Text that
/// needs more than `max_lines` ends with an ellipsis.
fn wrap(text: &str, width: usize, max_lines: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_owned()),
        }
    }
    if lines.len() > max_lines {
        lines.truncate(max_lines);
        let kept: String = lines[max_lines - 1].chars().take(width - 1).collect();
        lines[max_lines - 1] = format!("{}…", kept.trim_end());
    }
    lines.iter().map(|line| cut(line, width)).collect()
}

fn cut(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let kept: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{}…", kept.trim_end())
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
    let current = match reading.shown() {
        Some((book, "LAST READ")) => format!("between books, last read {} by {}", book.title, book.author),
        Some((book, _)) => format!("reading {} by {}", book.title, book.author),
        None => "between books".to_owned(),
    };
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
    cut(text, (max_px / (font_px * 0.56)) as usize)
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
            last_read: None,
        }
    }

    fn finished(title: &str) -> Option<Book> {
        Some(Book { title: title.into(), author: "Ann Leckie".into(), cover: None, progress: None })
    }

    #[test]
    fn last_read_shows_when_nothing_is_current() {
        let mut reading = sample(None);
        reading.current = None;
        reading.last_read = finished("Ancillary Justice");
        let svg = render(&reading, Theme::named(""));
        assert!(svg.contains(">LAST READ<") && svg.contains("Ancillary Justice"));
        assert!(svg.contains("Joe is between books, last read Ancillary Justice by Ann Leckie."));
    }

    #[test]
    fn current_book_wins_over_last_read() {
        let mut reading = sample(None);
        reading.last_read = finished("Ancillary Justice");
        let svg = render(&reading, Theme::named(""));
        assert!(svg.contains(">CURRENTLY READING<") && !svg.contains("Ancillary"));
    }

    #[test]
    fn missing_cover_gets_a_drawn_one() {
        let svg = render(&sample(None), Theme::named(""));
        assert!(svg.contains(r#"class="cover-title">Kafka &amp;"#));
        let other = fallback_cover(&finished("Dune").unwrap());
        assert_eq!(other, fallback_cover(&finished("Dune").unwrap()));
    }

    #[test]
    fn titles_wrap_onto_the_cover() {
        assert_eq!(wrap("The Left Hand of Darkness", 10, 5), ["The Left", "Hand of", "Darkness"]);
        assert_eq!(wrap("Supercalifragilistic", 10, 5), ["Supercali…"]);
        let long = wrap("a b c d e f g h i j k l m n o p q r s t u v w x y z", 3, 2);
        assert_eq!(long, ["a b", "c…"]);
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
