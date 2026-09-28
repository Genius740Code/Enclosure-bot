//! Lane O: test shape-breaking counters vs COLLAPSER
//! Q3b: early contest of corridor root, pre-build deny second wall, price-denial bank race

#[path = "support/scoutbase.rs"]
mod scoutbase;
mod opp_collapser;

use meridian_engine::notation::parse_square;
use meridian_engine::{Game, Move, Player};

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

fn play_with_solo(mut game: Game, ret_blue: bool, solo: &str, ret_fn: fn(&meridian_engine::Position) -> Option<Move>) -> (f64, f64, u16, u16) {
    let mut first = true;
    let mut ret_cuts = 0;
    let mut opp_cuts = 0;
    while !game.is_over() {
        let ret_moves = (game.position().to_move() == Player::Blue) == ret_blue;
        let mv = if first {
            first = false;
            solo_move(&game, solo)
        } else if ret_moves {
            ret_fn(game.position())
        } else {
            opp_collapser::best_move(game.position())
        };
        let outcome = game.play(mv.unwrap()).unwrap();
        if outcome.broken.is_some() {
            if ret_moves { ret_cuts += 1; } else { opp_cuts += 1; }
        }
    }
    let rs = game.position().score(if ret_blue { Player::Blue } else { Player::Red }).to_f64();
    let ss = game.position().score(if ret_blue { Player::Red } else { Player::Blue }).to_f64();
    (rs, ss, ret_cuts, opp_cuts)
}

fn run_matchup(ret_fn: fn(&meridian_engine::Position) -> Option<Move>, label: &str) {
    println!("=== {} vs COLLAPSER (skip=0, ret=Blue, 8 solos) ===", label);
    let mut w = 0; let mut l = 0; let mut sum_margin = 0.0;
    for g in 0..8 {
        let (rs, ss, ret_cuts, opp_cuts) = play_with_solo(Game::new(), true, SOLOS[g], ret_fn);
        let margin = (rs - ss) / rs * 100.0;
        let win = rs > ss;
        if win { w += 1; } else { l += 1; }
        sum_margin += margin;
        println!(
            "game{} solo={}: rs={:.0} ss={:.0} margin={:+.1}% cuts={}/{} {}",
            g + 1, SOLOS[g], rs, ss, margin, ret_cuts, opp_cuts, if win { "WIN" } else { "LOSS" }
        );
    }
    println!("Total: {}-{} avg margin={:+.1}%\n", w, l, sum_margin / 8.0);
}

// Counter 1: Early corridor contest - rank cuts of enemy long corridors higher
// This is implemented by modifying the search to add a term for cutting enemy
// corridors (long enemy edges that form open paths).
fn ret_with_corridor_contest(pos: &meridian_engine::Position) -> Option<Move> {
    // Use baseline search for now - will modify search.rs directly for real test
    retaliator::search::best_move(pos)
}

// Counter 2: Pre-build deny second wall - build remote/thicket structure early
fn ret_with_prebuild_deny(pos: &meridian_engine::Position) -> Option<Move> {
    retaliator::search::best_move(pos)
}

// Counter 3: Price-denial bank race - early-phase banked-area weight
fn ret_with_price_denial(pos: &meridian_engine::Position) -> Option<Move> {
    retaliator::search::best_move(pos)
}

fn main() {
    println!("=== SHAPE-BREAKING COUNTERS vs COLLAPSER ===\n");
    
    // Baseline
    run_matchup(retaliator::search::best_move, "BASELINE");
    
    // These will be run after modifying search.rs
    println!("Run with modified search.rs for each counter:");
    println!("1. Corridor contest: add enemy-corridor-cut bonus");
    println!("2. Pre-build deny: increase REMOTE_BONUS / add early remote bias");
    println!("3. Price-denial: add early banked-area weight (reduce HORIZON cap early)");
}