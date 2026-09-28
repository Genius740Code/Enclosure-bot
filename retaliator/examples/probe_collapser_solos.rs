//! Lane O: probe the COLLAPSER mimic against our search with 8 solos + skip conditions.
//! Reproduces league conditions: skip=10/20/30, both colors, WITH 8 solos.

#[path = "support/scoutbase.rs"]
mod scoutbase;
mod opp_collapser;

use meridian_engine::notation::parse_square;
use meridian_engine::{Game, Move, Player};

/// Reasonable Blue solos (only the starting edge's ENDPOINTS A10/D10 can extend).
const SOLOS: [&str; 8] = [
    "D10-F11", "D10-C7", "A10-C11", "A10-D13", "D10-E12", "A10-B13", "D10-G10", "A10-C7",
];

fn solo_move(game: &Game, text: &str) -> Option<Move> {
    let (from, to) = text.split_once('-')?;
    let mv = Move::between(parse_square(from)?, parse_square(to)?)?;
    if game.legal_moves().contains(mv) {
        Some(mv)
    } else {
        game.legal_moves().iter().next()
    }
}

fn play_with_solo(mut game: Game, ret_blue: bool, solo: &str) -> (f64, f64) {
    let mut first = true;
    while !game.is_over() {
        let ret_moves = (game.position().to_move() == Player::Blue) == ret_blue;
        let mv = if first {
            first = false;
            solo_move(&game, solo)
        } else if ret_moves {
            retaliator::search::best_move(game.position())
        } else {
            opp_collapser::best_move(game.position())
        };
        game.play(mv.unwrap()).unwrap();
    }
    let rs = game.position().score(if ret_blue { Player::Blue } else { Player::Red }).to_f64();
    let ss = game.position().score(if ret_blue { Player::Red } else { Player::Blue }).to_f64();
    (rs, ss)
}

fn run_matchup(ret_blue: bool, skip: usize, solo: &str) -> (f64, f64, bool) {
    let mut game = Game::new();
    
    // Play skip moves by retaliator (starting from empty board)
    for _ in 0..skip {
        if game.is_over() { break; }
        let mv = retaliator::search::best_move(game.position()).unwrap();
        game.play(mv).unwrap();
    }
    if game.is_over() { 
        return (0.0, 0.0, false); 
    }
    
    // Play out with the solo
    let (rs, ss) = play_with_solo(game, ret_blue, solo);
    let diff = rs - ss;
    let win = diff > 0.0;
    (rs, ss, win)
}

fn main() {
    println!("=== COLLAPSER PROBE (League Reproduction - WITH 8 Solos) ===\n");
    
    for skip in [0, 10, 20, 30] {
        println!("--- skip={} ---", skip);
        for ret_blue in [true, false] {
            let color = if ret_blue { "Blue" } else { "Red" };
            let mut w = 0; let mut l = 0; let mut sum_margin = 0.0; let mut n = 0;
            for g in 0..8 {
                let (rs, ss, win) = run_matchup(ret_blue, skip, SOLOS[g]);
                if rs == 0.0 && ss == 0.0 { continue; }
                let margin = (rs - ss) / rs * 100.0;
                if win { w += 1; } else { l += 1; }
                sum_margin += margin;
                n += 1;
            }
            println!(
                "  ret={} W-L {}-{} avg margin={:+.1}% (n={})",
                color, w, l, sum_margin / n.max(1) as f64, n
            );
        }
    }
    
    println!("\n=== Full 8-game matchup per skip (margin from ret perspective) ===");
    for skip in [0, 10, 20, 30] {
        println!("--- skip={} ---", skip);
        for ret_blue in [true, false] {
            let color = if ret_blue { "Blue" } else { "Red" };
            let mut w = 0; let mut l = 0; let mut sum_margin = 0.0; let mut n = 0;
            for g in 0..8 {
                let (rs, ss, win) = run_matchup(ret_blue, skip, SOLOS[g]);
                if rs == 0.0 && ss == 0.0 { continue; }
                let margin = (rs - ss) / rs * 100.0;
                if win { w += 1; } else { l += 1; }
                sum_margin += margin;
                n += 1;
                println!(
                    "  game{} ret={} margin={:+.1}% {}",
                    g + 1, color, margin, if win { "WIN" } else { "LOSS" }
                );
            }
            println!(
                "  ret={} {}-{} avg={:+.1}%",
                color, w, l, sum_margin / n.max(1) as f64
            );
        }
    }
}