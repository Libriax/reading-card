//! Prints a sample card, so the design can be worked on without deploying:
//! `cargo run --example preview -- dark > card.svg`

use libriax_card::card::{self, Book, Reading, Theme};

fn main() {
    let theme = std::env::args().nth(1).unwrap_or_default();
    let reading = Reading {
        name: "Joe".into(),
        year: 2026,
        books: 23,
        pages: 7412,
        streak: Some(12),
        current: Some(Book {
            title: "The Left Hand of Darkness".into(),
            author: "Ursula K. Le Guin".into(),
            cover: None,
            progress: Some((143, 304)),
        }),
        last_read: None,
    };
    println!("{}", card::render(&reading, Theme::named(&theme)));
}
