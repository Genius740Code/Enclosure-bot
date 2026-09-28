//! Lane O: test v2+avoid against COLLAPSER with league skip conditions.

#[path = "support/scoutbase.rs"]
mod scoutbase;
#[path = "support/v2base.rs"]
mod v2base;
mod opp_collapser;

use meridian_engine::{Game, Move, Player, Point};
use std::io::{self, Write};

fn play_with_avoid(mut game: Game, ret_blue: bool) -> (f64, f64, u16, u16) {
    let mut cuts_by_ret = 0;
    let mut cuts_suffered = 0;
    let mut cuts_history: Vec<(u16, Player, meridian_engine::Edge)> = Vec::new();
    
    while !game.is_over() {
        let ret_moves = (game.position().to_move() == Player::Blue) == ret_blue;
        let mv = if ret_moves {
            // v2+avoid: track our cut endpoints from last 6 actions
            let mut avoid: Vec<Point> = Vec::new();
            let played = u16::from(game.position().actions_played());
            for &(action, owner, cut) in cuts_history.iter().rev() {
                if played - action > 6 { break; }
                if owner == game.position().to_move() {
                    avoid.push(cut.origin());
                    avoid.push(cut.far());
                }
            }
            v2base::best_move_with_avoid(game.position(), &avoid)
        } else {
            opp_collapser::best_move(game.position())
        };
        let outcome = game.play(mv.unwrap()).unwrap();
        if let Some(cut) = outcome.broken {
            cuts_history.push((u16::from(game.position().actions_played()), game.position().to_move().opponent(), cut));
        }
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
    let (rs, ss, cuts_by, cuts_suffered) = play_with_avoid(game, ret_blue);
    let diff = rs - ss;
    let win = diff > 0.0;
    (rs, ss, cuts_by, cuts_suffered, win)
}

fn main() {
    println!("=== v2+avoid vs COLLAPSER (League Skip Conditions) ===\n");
    
    for skip in [0, 10, 20, 30] {
        println!("--- skip={} ---", skip);
        for ret_blue in [true, false] {
            let color = if ret_blue { "Blue" } else { "Red" };
            let (rs, ss, cuts_by, cuts_suffered, win) = run_matchup("v2+avoid", ret_blue, skip);
            if rs == 0.0 && ss == 0.0 {
                println!("  v2+avoid ret={}: game ended during skip", color);
                continue;
            }
            let margin = (rs - ss) / rs * 100.0;
            println!(
                "  v2+avoid ret={} rs={:.0} ss={:.0} margin={:+.1}% cuts={}/{} {}",
                color, rs, ss, margin, cuts_by, cuts_suffered,
                if win { "WIN" } else { "LOSS" }
            );
        }
    }
    
    println!("\n=== Full 8-game matchup (skip=0) ===");
    let mut w = 0; let mut l = 0; let mut sum_margin = 0.0;
    for game_idx in 0..8 {
        let ret_blue = game_idx < 4;
        let (rs, ss, cuts_by, cuts_suffered, win) = run_matchup("v2+avoid", ret_blue, 0);
        if rs == 0.0 && ss == 0.0 { continue; }
        let margin = (rs - ss) / rs * 100.0;
        let diff = rs - ss;
        if diff > 0.0 { w += 1; } else { l += 1; }
        sum_margin += margin;
        let color = if ret_blue { "Blue" } else { "Red" };
        println!(
            "v2+avoid vs collapser game{} ret={} rs={:.0} ss={:.0} margin={:+.1}% cuts={}/{} {}",
            game_idx, color, rs, ss, margin, cuts_by, cuts_suffered,
            if win { "WIN" } else { "LOSS" }
        );
        io::stdout().flush().unwrap();
    }
    println!(
        "v2+avoid vs COLLAPSER: {}-{}, avg margin {:+.1}%",
        w, l, sum_margin / 8.0
    );
}