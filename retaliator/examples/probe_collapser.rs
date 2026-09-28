//! Lane O: probe the COLLAPSER mimic against our search variants.
//! Reproduces league conditions: skip=10/20/30, both colors, NO solo.

#[path = "support/scoutbase.rs"]
mod scoutbase;
#[path = "support/v1base.rs"]
mod v1base;
#[path = "support/v2base.rs"]
mod v2base;
mod opp_collapser;

use meridian_engine::{Game, Move, Player};
use std::io::{self, Write};

fn play(mut game: Game, ret_blue: bool) -> (f64, f64, u16, u16) {
    let mut cuts_by_ret = 0;
    let mut cuts_suffered = 0;
    while !game.is_over() {
        let ret_moves = (game.position().to_move() == Player::Blue) == ret_blue;
        let mv = if ret_moves {
            retaliator::search::best_move(game.position())
        } else {
            opp_collapser::best_move(game.position())
        };
        let outcome = game.play(mv.unwrap()).unwrap();
        if outcome.broken.is_some() {
            if ret_moves { cuts_suffered += 1; } else { cuts_by_ret += 1; }
        }
    }
    let rs = game.position().score(if ret_blue { Player::Blue } else { Player::Red }).to_f64();
    let ss = game.position().score(if ret_blue { Player::Red } else { Player::Blue }).to_f64();
    (rs, ss, cuts_by_ret, cuts_suffered)
}

fn run_matchup(label: &str, ret_blue: bool, skip: usize) -> (f64, f64, u16, u16, bool) {
    let mut game = Game::new();
    
    // Play skip moves by retaliator (starting from empty board)
    for _ in 0..skip {
        if game.is_over() { break; }
        let mv = retaliator::search::best_move(game.position()).unwrap();
        game.play(mv).unwrap();
    }
    if game.is_over() { 
        return (0.0, 0.0, 0, 0, false); 
    }
    
    // Play out with the opponent
    let (rs, ss, cuts_by, cuts_suffered) = play(game, ret_blue);
    let diff = rs - ss;
    let win = diff > 0.0;
    (rs, ss, cuts_by, cuts_suffered, win)
}

fn main() {
    println!("=== COLLAPSER PROBE (League Reproduction - No Solo) ===\n");
    
    // Test the exact league collapse conditions: skip=10,20,30 with both colors
    for skip in [0, 10, 20, 30] {
        println!("--- skip={} ---", skip);
        for ret_blue in [true, false] {
            let color = if ret_blue { "Blue" } else { "Red" };
            let (rs, ss, cuts_by, cuts_suffered, win) = run_matchup("v3", ret_blue, skip);
            if rs == 0.0 && ss == 0.0 {
                println!("  v3 ret={}: game ended during skip", color);
                continue;
            }
            let margin = (rs - ss) / rs * 100.0;
            println!(
                "  v3 ret={} rs={:.0} ss={:.0} margin={:+.1}% cuts={}/{} {}",
                color, rs, ss, margin, cuts_by, cuts_suffered,
                if win { "WIN" } else { "LOSS" }
            );
        }
    }
    
    println!("\n=== Full matchups (skip=0, 8 games via color) ===");
    let mut results = Vec::new();
    for label in ["v3", "v1", "v2"] {
        let mut w = 0; let mut l = 0; let mut sum_margin = 0.0;
        for game_idx in 0..8 {
            // Use different skip patterns for variety? No, league uses skip=0 for these
            let ret_blue = game_idx < 4; // First 4 as Blue, next 4 as Red
            let (rs, ss, cuts_by, cuts_suffered, win) = run_matchup(label, ret_blue, 0);
            if rs == 0.0 && ss == 0.0 { continue; }
            let margin = (rs - ss) / rs * 100.0;
            let diff = rs - ss;
            if diff > 0.0 { w += 1; } else { l += 1; }
            sum_margin += margin;
            let color = if ret_blue { "Blue" } else { "Red" };
            println!(
                "{} vs collapser game{} ret={} rs={:.0} ss={:.0} margin={:+.1}% cuts={}/{} {}",
                label, game_idx, color, rs, ss, margin, cuts_by, cuts_suffered,
                if win { "WIN" } else { "LOSS" }
            );
            io::stdout().flush().unwrap();
        }
        println!(
            "{} vs COLLAPSER: {}-{}, avg margin {:+.1}%",
            label, w, l, sum_margin / 8.0
        );
        results.push((label, w, l, sum_margin / 8.0));
    }
    println!("\n=== SUMMARY ===");
    for (label, w, l, margin) in results {
        println!("{}: {}-{} ({:+.1}%)", label, w, l, margin);
    }
}