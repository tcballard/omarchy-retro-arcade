//! Development-only bridge for comparison with the pinned upstream JavaScript.
use omarchy_2048::engine::{slide, Board, Direction};
use serde::Deserialize;
use std::io::{self, Read};
#[derive(Deserialize)]
struct Case {
    board: Board,
    direction: Direction,
}
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let cases: Vec<Case> = serde_json::from_str(&input).unwrap();
    let results: Vec<_> = cases.iter().map(|c| slide(&c.board, c.direction)).collect();
    println!("{}", serde_json::to_string(&results).unwrap());
}
