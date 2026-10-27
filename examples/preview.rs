//! Prints a sample card, so the design can be worked on without deploying:
//! `cargo run --example preview -- dark > card.svg`

use libriax_card::card::{self, Book, Reading, Theme};

fn main() {
    let theme = std::env::args().nth(1).unwrap_or_default();
    let cover = include_bytes!("../docs/blood-meridian.jpg");
    let reading = Reading {
        name: "Joe".into(),
        year: 2026,
        books: 23,
        pages: 7412,
        streak: Some(12),
        current: Some(Book {
            title: "Blood Meridian".into(),
            author: "Cormac McCarthy".into(),
            cover: Some(format!("data:image/jpeg;base64,{}", libriax_card::base64(cover))),
            progress: Some((143, 355)),
        }),
        last_read: None,
    };
    println!("{}", card::render(&reading, Theme::named(&theme)));
}
