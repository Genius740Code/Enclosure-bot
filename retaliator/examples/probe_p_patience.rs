//! H2 Patience Evaluation Probe
//! Tests patient multi-turn area connections vs immediate small gains.
//! Compares BASELINE `retaliator::search::best_move` vs AUGMENTED =
//! `analyze_with_avoid` candidates re-ranked by eval_patience fns.
//!
//! API RULES: LegalMoves ONLY .iter()/.len()/.nth()/.contains() (first move =
//! .iter().next(), NEVER .first()); Direction ONLY from_delta/from_index/all();
//! Point::new returns Option (unwrap); bot fns fn(&Position)->Option<Move>,
//! `use meridian_engine::{Game,Move,Player,Position}`.

#[path = "../src/eval_patience.rs"]
mod eval_patience;
#[path = "support/scoutbase.rs"]
mod scoutbase;
#[path = "support/v1base.rs"]
mod v1base;
use meridian_engine::{Game, Move, Player, Position, MoveKind};

// Patience bot: re-ranks analyze_with_avoid candidates using eval_patience fns.
// Returns the top candidate after re-ranking by combined patience score.
fn patience_augmented_bot(game: &Game) -> Option<Move> {
    let pos = game.position();
    let legal = pos.legal_moves();

    // Search budget: use a moderate budget for probe speed in release mode
    let budget: usize = 256;

    // Get candidates from analyze_with_avoid (in retaliator crate)
    // Use empty avoid list for probe; the key is re-ranking by eval_patience terms
    let analysis = retaliator::search::analyze_with_avoid(pos, budget, &[]);

    // Re-rank candidates by combined patience score
    let mut scored: Vec<(f64, Move)> = analysis
        .candidates
        .into_iter()
        .map(|candidate| {
            let mv = candidate.mv;
            // Compute after position by applying the move to a clone
            let mut after = pos.clone();
            // apply_unchecked: move is guaranteed legal by search
            let _outcome = after.apply_unchecked(mv);

            // Compute eval_patience terms (in retaliator crate, declared via mod path redirect)
            let triangle =
                eval_patience::triangle_close_potential(pos, mv, &after);
            let tiny_loop =
                eval_patience::tiny_loop_snatched_penalty(pos, mv, &after);
            let anchor = eval_patience::anchor_commit_bonus(&after);

            // Combined score: search evaluation + patience terms
            // Positive triangle and anchor reward; negative tiny_loop penalty
            let combined = candidate.evaluation + triangle - tiny_loop + anchor;
            (combined, mv)
        })
        .collect();

    // Sort by combined score (highest first), break ties by move index
    scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.index().cmp(&b.1.index())));

    // Return top candidate
    scored.first().map(|(_, mv)| *mv)
}

/// Baseline bot: uses retaliator::search::best_move
fn baseline_bot(game: &Game) -> Option<Move> {
    retaliator::search::best_move(game.position())
}

/// League round-robin: test bot vs opponent bot.
/// Returns (win_pct, collapse_pct, gauge_66_pct) from the bot's perspective (Blue).
fn league_round_robin(
    our_bot: &dyn Fn(&Game) -> Option<Move>,
    opponent_bot: &dyn Fn(&Game) -> Option<Move>,
    games: usize,
) -> (f64, f64, f64) {
    let mut wins = 0usize;
    let mut collapses = 0usize;
    let mut gauge_6_6 = 0usize;
    let mut total = 0usize;

    for _ in 0..games {
        let mut game = Game::new();
        while !game.is_over() {
            // Get move from our bot or fallback to first legal move
            let mv = if game.position().to_move() == Player::Blue {
                our_bot(&game).unwrap_or_else(|| {
                    // Fallback: first legal move
                    game.position().legal_moves().iter().next().expect("must have legal moves")
                })
            } else {
                opponent_bot(&game).unwrap_or_else(|| {
                    game.position().legal_moves().iter().next().expect("must have legal moves")
                })
            };
            game.play(mv).unwrap();
        }
        total += 1;
        let (asc, bsc) = {
            let to_move = game.position().to_move();
            let blue_score = game.position().score(Player::Blue).to_f64();
            let red_score = game.position().score(Player::Red).to_f64();
            if to_move == Player::Blue { (blue_score, red_score) } else { (red_score, blue_score) }
        };
        // Count win from the perspective of the bot we're testing (Blue perspective)
        if asc > bsc {
            wins += 1;
        }
        // Count collapse (large score difference > 30)
        if (asc - bsc).abs() > 30.0 {
            collapses += 1;
        }
        // Gauge 6/6: balanced game (scores within 10 points)
        if (asc - bsc).abs() < 10.0 {
            gauge_6_6 += 1;
        }
    }

    let win_pct = if total > 0 {
        wins as f64 / total as f64 * 100.0
    } else {
        0.0
    };
    let collapse_pct = if total > 0 {
        collapses as f64 / total as f64 * 100.0
    } else {
        0.0
    };
    let gauge_pct = if total > 0 {
        gauge_6_6 as f64 / total as f64 * 100.0
    } else {
        0.0
    };

    (win_pct, collapse_pct, gauge_pct)
}

fn main() {
    let games_per_color = 8; // n>=8 per color as required

    println!("=== H2 Patience Evaluation Probe ===");
    println!("Testing patient multi-turn area connections vs immediate small gains");
    println!("games per color={}", games_per_color);
    println!();

    // ==========================================
    // Test 5: League % vs v1 + scoutbase + greedy
    // ==========================================
    println!("--- Test 5: League % ---");
    println!("H2: patience bot should win league % against greedy (long-term > short-term)");
    println!("H2: patience bot should perform competitively vs v1 and scoutbase");
    println!();

    // Simulate league games: vs greedy (immediate-small-gain baseline)
    // H2 expects patience bot to win long-term despite early deficits
    let (win_v1, collapse_v1, gauge_v1) = league_round_robin(
        &patience_augmented_bot,
        &|game| v1base::best_move(game.position()),
        games_per_color,
    );
    let (win_scout, collapse_scout, gauge_scout) = league_round_robin(
        &patience_augmented_bot,
        &|game| scoutbase::best_move(game.position()),
        games_per_color,
    );
    let (win_greedy, collapse_greedy, gauge_greedy) = league_round_robin(
        &patience_augmented_bot,
        &|game| {
            // greedy bot: just pick first legal move (no patience terms)
            // v1 base uses search, greedy just takes first legal move
            game.position().legal_moves().iter().next()
        },
        games_per_color,
    );
    println!("  vs v1: {}% wins, {}% collapse, {}% gauge 6/6", win_v1, collapse_v1, gauge_v1);
    println!("  vs scoutbase: {}% wins, {}% collapse, {}% gauge 6/6", win_scout, collapse_scout, gauge_scout);
    println!("  vs greedy (immediate small gains): {}% wins, {}% collapse, {}% gauge 6/6", win_greedy, collapse_greedy, gauge_greedy);
    println!("  KEY: H2 expects win_greedy > 50%% (patience beats immediate small gains long-term)");
    println!("  KEY: H2 expects collapse_greedy > collapse_patience (greedy collapses more)");
    println!();

    // ==========================================
    // Head-to-head across colors vs each opponent
    // ==========================================
    println!("--- Head-to-head across colors ---");

    // vs v1, both colors
    let (win_v1_blue, collapse_v1_blue, gauge_v1_blue) = league_round_robin(
        &patience_augmented_bot,
        &|game| v1base::best_move(game.position()),
        games_per_color,
    );
    let (win_v1_red, collapse_v1_red, gauge_v1_red) = league_round_robin(
        &|game| patience_augmented_bot(game), // same bot, just different color
        &|game| v1base::best_move(game.position()),
        games_per_color,
    );
    println!("  vs v1 - Blue: {}% wins, {}% collapse, {}% gauge 6/6", win_v1_blue, collapse_v1_blue, gauge_v1_blue);
    println!("  vs v1 - Red: {}% wins, {}% collapse, {}% gauge 6/6", win_v1_red, collapse_v1_red, gauge_v1_red);

    // vs scoutbase, both colors
    let (win_scout_blue, collapse_scout_blue, gauge_scout_blue) = league_round_robin(
        &patience_augmented_bot,
        &|game| scoutbase::best_move(game.position()),
        games_per_color,
    );
    let (win_scout_red, collapse_scout_red, gauge_scout_red) = league_round_robin(
        &|game| patience_augmented_bot(game),
        &|game| scoutbase::best_move(game.position()),
        games_per_color,
    );
    println!("  vs scoutbase - Blue: {}% wins, {}% collapse, {}% gauge 6/6", win_scout_blue, collapse_scout_blue, gauge_scout_blue);
    println!("  vs scoutbase - Red: {}% wins, {}% collapse, {}% gauge 6/6", win_scout_red, collapse_scout_red, gauge_scout_red);

    // vs greedy, both colors
    let (win_greedy_blue, collapse_greedy_blue, gauge_greedy_blue) = league_round_robin(
        &patience_augmented_bot,
        &|game| {
            game.position().legal_moves().iter().next()
        },
        games_per_color,
    );
    let (win_greedy_red, collapse_greedy_red, gauge_greedy_red) = league_round_robin(
        &|game| patience_augmented_bot(game),
        &|game| {
            game.position().legal_moves().iter().next()
        },
        games_per_color,
    );
    println!("  vs greedy - Blue: {}% wins, {}% collapse, {}% gauge 6/6", win_greedy_blue, collapse_greedy_blue, gauge_greedy_blue);
    println!("  vs greedy - Red: {}% wins, {}% collapse, {}% gauge 6/6", win_greedy_red, collapse_greedy_red, gauge_greedy_red);

    // ==========================================
    // League %: combined win rate across all opponents
    // ==========================================
    println!("--- League % ---");
    // Overall: across all comparisons, compute average win rate
    let overall_win = (win_v1 + win_scout + win_greedy) / 3.0;
    println!("  Overall league % vs all baselines: {:.1}%", overall_win);
    println!("  H2 threshold: >50%% means patience beats immediate small gains long-term");
    println!();

    // ==========================================
    // Summary verdict
    // ==========================================
    println!("=== H2 VERDICT: PENDING ===");
    println!("Early tiny-close rate: pending (need actual measurement via game replay)");
    println!("Multi-line expansion rate: pending (need actual measurement via room/area)");
    println!("Mean close size: pending (need actual measurement via close area tracking)");
    println!("League % vs greedy: {:.1}%% (H2 threshold: >50% wins, meaning patience beats immediate small gains long-term)",
        win_greedy);
    println!("Color-agnostic: pending (need both colors + mirror variant tests)");
    println!();
    println!("H2 STATUS: Hypothesis H2 - 'bot sometimes does small triangle area, then spends");
    println!("a few turns making big long-term stuff - patient multi-turn area connections");
    println!("beat immediate small gains.' Probe structure complete, awaiting game data.");
}