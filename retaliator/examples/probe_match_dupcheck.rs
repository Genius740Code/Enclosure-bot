//! Lane D dup-check: why do tournament games 7/8 (solos C10-E13 / B10-C7)
//! come out identical in every matchup? Prints each solo's legality from the
//! starting position and the first 12 moves of games 7/8 for one matchup.

#[path = "support/scoutbase.rs"]
mod scoutbase;
#[path = "support/v1base.rs"]
mod v1base;
#[path = "opp_gbstyle.rs"]
mod opp_gbstyle;

use meridian_engine::notation::{move_text, parse_square};
use meridian_engine::{Game, Move};

const SOLOS: [&str; 8] = [
    "D10-F11", "D10-C7", "A10-C11", "B10-E12", "A10-D13", "D10-E12", "C10-E13", "B10-C7",
];

fn solo_move(game: &Game, text: &str) -> Option<Move> {
    let (from, to) = text.split_once('-')?;
    let mv = Move::between(parse_square(from)?, parse_square(to)?)?;
    if game.legal_moves().contains(mv) {
        Some(mv)
    } else {
        println!("  solo {text} ILLEGAL -> first legal move");
        game.legal_moves().iter().next()
    }
}

fn play_with_trace(solo: &str) -> Vec<String> {
    let mut game = Game::new();
    let mut trace = Vec::new();
    let mut first = true;
    while !game.is_over() {
        let mv = if first {
            first = false;
            solo_move(&game, solo)
        } else {
            let base_moves = game.position().to_move() == meridian_engine::Player::Blue;
            if base_moves {
                v1base::best_move(game.position())
            } else {
                opp_gbstyle::best_move(game.position())
            }
        };
        let played = mv.unwrap();
        trace.push(move_text(played.source, played.target().expect("on board")));
        game.play(played).unwrap();
        if trace.len() >= 12 {
            break;
        }
    }
    trace
}

fn main() {
    println!("Solo legality from the starting position:");
    let mut game = Game::new();
    for solo in [
        "D10-F11", "D10-C7", "A10-C11", "B10-E12", "A10-D13", "D10-E12", "C10-E13", "B10-C7",
        "A10-B13", "D10-G10", "C10-C7", "B10-B13", "A10-C7", "D10-D13", "B10-D11", "A10-A13",
    ] {
        let (from, to) = solo.split_once('-').unwrap();
        match Move::between(parse_square(from).unwrap(), parse_square(to).unwrap()) {
            Some(mv) => println!(
                "  {solo}: legal={}",
                game.legal_moves().contains(mv)
            ),
            None => println!("  {solo}: not constructible"),
        }
    }
    println!("v1(base,blue) vs gb game 7 first moves: {:?}", play_with_trace(SOLOS[6]));
    println!("v1(base,blue) vs gb game 8 first moves: {:?}", play_with_trace(SOLOS[7]));
}
