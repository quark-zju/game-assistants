#![allow(dead_code)]

use std::env;

mod cjul;
mod cribbage_solitaire;
mod dungeons;
pub(crate) mod util;

const SUPPORTED_GAMES: &[(&str, fn())] = &[
    ("cjul", cjul::main),
    ("cribbage", cribbage_solitaire::main),
    ("dungeons", dungeons::main),
];

fn common_prefix(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(a, b)| a == b).count()
}

fn main() {
    let name = env::args()
        .nth(1)
        .or_else(|| env::var("M").ok())
        .unwrap_or_default();
    let best = SUPPORTED_GAMES
        .iter()
        .max_by_key(|(n, _)| common_prefix(&name, n));
    match best {
        None => {
            eprintln!("Ambigious or unsupported game name: {:?}", name);
            eprintln!(
                "Supported game names: {:?}",
                SUPPORTED_GAMES.iter().map(|v| v.0).collect::<Vec<_>>()
            );
        }
        Some((name, entry_point)) => {
            eprintln!("Selected game: {}", name);
            entry_point();
        }
    }
}
