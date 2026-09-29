//! h2h_base — Head-to-head runner: LIVE engine vs v7base frozen baseline.
//!
//! Plays 10 games alternating colors (5 Blue / 5 Red), prints per-chair splits + total.
//! Deterministic: fixed seeds/budgets, engine legality enforced.
//! Self-identity check: v7base vs live engine on ≥20 positions must be byte-identical picks.

use meridian_engine::{Game, Move, Player, Position};
use std::time::Instant;

#[path = "support/v7base.rs"]
mod v7base;

/// Deterministic seed for reproducible games. Each game gets a unique seed derived from this.
const BASE_SEED: u64 = 0xC0FFEE_1234_5678;

/// Fixed budget for both engines (matches MOVE_BUDGET = 4096).
const BUDGET: usize = 4096;

/// Plays a single game with the given color assignment.
/// Returns (live_score, v7base_score, live_breaks, v7base_breaks, game_duration_ms).
fn play_game(live_is_blue: bool, game_idx: usize) -> (f64, f64, u32, u32, u64) {
    let mut game = Game::new();
    let (mut live_breaks, mut base_breaks) = (0u32, 0u32);
    let t0 = Instant::now();

    // Deterministic per-game seed (not used by engines directly, but available for any RNG)
    let _game_seed = BASE_SEED.wrapping_add(game_idx as u64);

    while !game.is_over() {
        let to_move = game.position().to_move();
        let live_moves = (to_move == Player::Blue) == live_is_blue;

        let mv = if live_moves {
            // LIVE engine: uses best_move_routed with mesh prefix + opener + timed search
            // For deterministic h2h, we use analyze_with_avoid with fixed budget (no timing variance)
            retaliator::search::analyze_with_avoid(game.position(), BUDGET, &[])
                .candidates
                .first()
                .map(|c| c.mv)
        } else {
            // v7base frozen baseline: uses best_move_routed with mesh prefix + opener + timed search
            // For deterministic h2h, we use analyze_with_avoid with fixed budget
            v7base::analyze_with_avoid(game.position(), BUDGET, &[])
                .candidates
                .first()
                .map(|c| c.mv)
        };

        let Some(mv) = mv else { break };

        // Verify legality before playing
        if game.position().check_move(mv).is_err() {
            eprintln!("WARNING: Illegal move {} detected, skipping", mv.index());
            break;
        }

        let outcome = game.play(mv).expect("bot moves are legal");

        if outcome.broken.is_some() {
            if live_moves {
                live_breaks += 1;
            } else {
                base_breaks += 1;
            }
        }
    }

    let b = game.position().score(Player::Blue).to_f64();
    let r = game.position().score(Player::Red).to_f64();
    let duration_ms = t0.elapsed().as_millis() as u64;

    (b, r, live_breaks, base_breaks, duration_ms)
}

/// Self-identity check: compare v7base vs live engine picks on ≥20 positions.
/// Returns (identical_count, total_tested).
fn self_identity_check() -> (usize, usize) {
    use std::collections::HashSet;

    // Generate diverse positions by playing partial games with different seeds
    let mut test_positions: Vec<(Position, Vec<Move>)> = Vec::new();
    let mut seen_hashes = HashSet::new();

    for seed_idx in 0..100 {
        if test_positions.len() >= 30 {
            break;
        }
        let mut game = Game::new();
        let seed = BASE_SEED.wrapping_add(seed_idx);

        // Play a few moves to get varied positions
        let moves_to_play = (seed % 20) as usize + 5; // 5-24 moves
        let mut legal = true;

        for _ in 0..moves_to_play {
            if game.is_over() {
                legal = false;
                break;
            }
            let mv = retaliator::search::analyze_with_avoid(game.position(), BUDGET, &[])
                .candidates
                .first()
                .map(|c| c.mv);
            let Some(mv) = mv else { legal = false; break };
            if game.position().check_move(mv).is_err() {
                legal = false;
                break;
            }
            game.play(mv).ok();
        }

        if legal && !game.is_over() {
            let pos = game.position().clone();
            let history: Vec<Move> = game.moves().iter().cloned().collect();
            // Use a hash of the position to deduplicate
            let hash = format!("{:?}", pos);
            if seen_hashes.insert(hash) {
                test_positions.push((pos, history));
            }
        }
    }

    if test_positions.len() < 20 {
        eprintln!("WARNING: Only generated {} test positions (need ≥20)", test_positions.len());
    }

    let mut identical = 0;
    for (pos, history) in test_positions.iter().take(20) {
        let live_pick = retaliator::search::best_move_routed(pos, history, &[]);
        let base_pick = v7base::best_move_routed(pos, history, &[]);

        match (live_pick, base_pick) {
            (Some(l), Some(b)) if l == b => identical += 1,
            (None, None) => identical += 1,
            (Some(l), Some(b)) => {
                eprintln!("MISMATCH: live={} v7base={} pos_actions={}", l.index(), b.index(), pos.actions_played());
            }
            (Some(l), None) => {
                eprintln!("MISMATCH: live={} v7base=None pos_actions={}", l.index(), pos.actions_played());
            }
            (None, Some(b)) => {
                eprintln!("MISMATCH: live=None v7base={} pos_actions={}", b.index(), pos.actions_played());
            }
        }
    }

    (identical, test_positions.len().min(20))
}

fn main() {
    println!("=== h2h_base: LIVE vs v7base (frozen baseline) ===");
    println!("Budget: {} positions, 10 games (5 Blue / 5 Red each), deterministic", BUDGET);
    println!();

    // Self-identity check first
    println!("--- Self-identity check (v7base vs LIVE on 20 positions) ---");
    let (identical, tested) = self_identity_check();
    println!("Identical picks: {}/{} ({:.1}%)", identical, tested, identical as f64 / tested as f64 * 100.0);
    if identical == tested {
        println!("✓ IDENTITY CHECK PASSED: v7base is a faithful snapshot");
    } else {
        println!("✗ IDENTITY CHECK FAILED: v7base diverges from LIVE engine");
    }
    println!();

    // H2H games
    println!("--- H2H Games (LIVE vs v7base) ---");
    let mut live_wins_blue = 0;
    let mut live_wins_red = 0;
    let mut live_wins_total = 0;
    let mut live_wins_blue = 0;
    let mut live_wins_red = 0;
    let mut total_live_breaks = 0;
    let mut total_base_breaks = 0;
    let mut total_duration = 0u64;

    for g in 0..10 {
        let live_is_blue = g % 2 == 0; // Alternate: 0,2,4,6,8 = Blue (5 games), 1,3,5,7,9 = Red (5 games)
        let (b, r, live_brk, base_brk, dur) = play_game(live_is_blue, g);

        let (live_score, base_score) = if live_is_blue { (b, r) } else { (r, b) };
        let live_won = live_score > base_score;

        if live_is_blue {
            if live_won { live_wins_blue += 1; }
        } else {
            if live_won { live_wins_red += 1; }
        }
        if live_won { live_wins_total += 1; }

        total_live_breaks += live_brk;
        total_base_breaks += base_brk;
        total_duration += dur;

        let margin = if live_score + base_score > 0.0 {
            (live_score - base_score) / (live_score + base_score) * 100.0
        } else { 0.0 };

        println!(
            "game {:2}: live={:4} base={:4} color={:4} margin={:+.1}% breaks L={} B={} ({}ms)",
            g + 1,
            live_score as i64,
            base_score as i64,
            if live_is_blue { "Blue" } else { "Red" },
            margin,
            live_brk,
            base_brk,
            dur
        );
    }

    println!();
    println!("=== SUMMARY ===");
    println!("Total:          LIVE {}/10 wins ({:.0}%)", live_wins_total, live_wins_total as f64 / 10.0 * 100.0);
    println!("As Blue (5 games):  LIVE {}/5 wins ({:.0}%)", live_wins_blue, live_wins_blue as f64 / 5.0 * 100.0);
    println!("As Red  (5 games):  LIVE {}/5 wins ({:.0}%)", live_wins_red, live_wins_red as f64 / 5.0 * 100.0);
    println!("Total breaks:   LIVE {}  v7base {}", total_live_breaks, total_base_breaks);
    println!("Total time:     {:.2}s", total_duration as f64 / 1000.0);
    println!();

    // Gate check: per-chair splits (red ≥3/5 AND blue ≥3/5 per v8-plan §2)
    let blue_ok = live_wins_blue >= 3;
    let red_ok = live_wins_red >= 3;
    println!("=== GATE CHECK (v8-plan §2: per-chair ≥3/5) ===");
    println!("Blue chair: {}/5 — {}", live_wins_blue, if blue_ok { "PASS" } else { "FAIL" });
    println!("Red chair:  {}/5 — {}", live_wins_red, if red_ok { "PASS" } else { "FAIL" });
    println!("Overall h2h: {}/10 — {}", live_wins_total, if live_wins_total >= 6 { "PASS" } else { "FAIL" });
}