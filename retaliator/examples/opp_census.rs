//! Lane O (v7) Q3a: GB unbreakable-share census.
//!
//! Per-game number: fraction of a player's BANKED area enclosed in 2+ touch
//! walls (walls whose nodes each touch 2+ own edges = near-unbreakable, any
//! cut touches 2+; cf. DENSE_BONUS rationale in search.rs).
//!
//! Operational definition (engine public API only):
//! - At every scoring event (MoveOutcome.scored.is_some()), snapshot
//!   thickness T(P) = P-edges with both endpoints degree>=2 / all P-edges.
//! - share(P) = sum_events gain_P * T_P / sum_events gain_P.
//! Reference implementation for C2; spec in research/v7-gb-census-spec.md.
//!
//! Arms: GB-mimic vs scoutbase, both colors x 8 solos (n=16). Prediction
//! test: does the higher-share player win?

mod opp_gbstyle;

#[path = "support/scoutbase.rs"]
mod scoutbase;

use meridian_engine::notation::parse_square;
use meridian_engine::{Game, Move, Player, Position};
use std::collections::HashMap;

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

/// Share of `player`'s edges with both endpoints touching 2+ own edges.
fn thickness(pos: &Position, player: Player) -> f64 {
    let mut degree: HashMap<usize, u32> = HashMap::new();
    for edge in pos.edges(player).iter() {
        for end in [edge.origin(), edge.far()] {
            *degree.entry(end.index()).or_insert(0) += 1;
        }
    }
    let edges = pos.edges(player);
    if edges.is_empty() {
        return 0.0;
    }
    let thick = edges
        .iter()
        .filter(|e| {
            degree.get(&e.origin().index()).unwrap_or(&0) >= &2
                && degree.get(&e.far().index()).unwrap_or(&0) >= &2
        })
        .count();
    thick as f64 / edges.len() as f64
}

fn census_game(gb_blue: bool, solo: &str) -> (f64, f64, f64, f64, bool) {
    let mut game = Game::new();
    let mut first = true;
    // (weighted sum, total gain) per color index 0=Blue 1=Red
    let mut acc = [0.0f64; 2];
    let mut tot = [0.0f64; 2];
    while !game.is_over() {
        let gb_moves = (game.position().to_move() == Player::Blue) == gb_blue;
        let mv = if first {
            first = false;
            solo_move(&game, solo)
        } else if gb_moves {
            opp_gbstyle::best_move(game.position())
        } else {
            scoutbase::best_move(game.position())
        };
        let outcome = game.play(mv.unwrap()).unwrap();
        if let Some(scored) = outcome.scored {
            let gains = [scored[0].to_f64(), scored[1].to_f64()];
            for (i, pl) in [Player::Blue, Player::Red].iter().enumerate() {
                if gains[i] > 0.0 {
                    acc[i] += gains[i] * thickness(game.position(), *pl);
                    tot[i] += gains[i];
                }
            }
        }
    }
    let share = |i: usize| if tot[i] > 0.0 { acc[i] / tot[i] } else { f64::NAN };
    let bs = game.position().score(Player::Blue).to_f64();
    let rs = game.position().score(Player::Red).to_f64();
    let gb_share = if gb_blue { share(0) } else { share(1) };
    let foe_share = if gb_blue { share(1) } else { share(0) };
    let gb_won = if gb_blue { bs > rs } else { rs > bs };
    (gb_share, foe_share, bs, rs, gb_won)
}

fn main() {
    println!("=== GB UNBREAKABLE-SHARE CENSUS (gb vs scoutbase, 8 solos x both colors) ===");
    let mut agree = 0;
    let mut decided = 0;
    let mut gb_wins = 0;
    for gb_blue in [true, false] {
        for solo in SOLOS {
            let (gb, foe, bs, rs, won) = census_game(gb_blue, solo);
            if won {
                gb_wins += 1;
            }
            let pred_gb = gb > foe;
            let ok = if gb.is_nan() || foe.is_nan() {
                "N/A"
            } else {
                decided += 1;
                if pred_gb == won {
                    agree += 1;
                    "AGREE"
                } else {
                    "DISAGREE"
                }
            };
            println!(
                "gb={} solo={}: gbShare={:.2} foeShare={:.2} score B={:.0} R={:.0} {} pred={}",
                if gb_blue { "Blue" } else { "Red" },
                solo,
                gb,
                foe,
                bs,
                rs,
                if won { "GB-WIN" } else { "GB-LOSS" },
                ok
            );
        }
    }
    println!(
        "GB W-L: {}-{}; higher-share-wins: {}/{} decided (n=16)",
        gb_wins,
        16 - gb_wins,
        agree,
        decided
    );
}
