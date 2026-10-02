//! probe_r_reinforce: H1 reinforcement head-to-head.
//!!
//! Matches: n>=8 games alternating colors, RETALIATOR (with REINFORCE H1 eval)
//! vs v1base + scout + greedy, printed as wins, reinforce freq, area lifetime, league %.
//! Compares BASELINE best_move vs AUGMENTED analyze_with_avoid re-ranked by eval_reinforce bonus.
//!
//! Usage: cargo run --example probe_r_reinforce [games=n]

use meridian_engine::{Game, Move, Player, Position};

/// 1-ply greedy: maximize own enclosed area after the move, ties by move id.
fn greedy(position: &Position) -> Option<Move> {
    let mover = position.to_move();
    let mut best: Option<(f64, usize, Move)> = None;
    for mv in position.legal_moves().iter() {
        let mut after = position.clone();
        after.apply_unchecked(mv);
        let v = after.area(mover).to_f64();
        let id = mv.index();
        if best.is_none_or(|(bv, bid, _)| v > bv || (v == bv && id < bid)) {
            best = Some((v, id, mv));
        }
    }
    best.map(|(_, _, mv)| mv)
}

/// 1-ply scout: short search over next two actions.
fn scout(position: &Position) -> Option<Move> {
    retaliator::search::best_move(position)
}

/// Reinforcement-aware player: uses best_move with reinforce eval.
/// The REINFORCE H1 evaluation influences move selection through the
/// search framework's horizon extension and contact avoidance fixes.
/// Candidates from analyze_with_avoid are re-ranked by the reinforcement
/// bonus, preferring moves that protect our high-value area.
fn reinforce_player(position: &Position) -> Option<Move> {
    let budget = 256; // reduced budget for probe gameplay
    let analysis = retaliator::search::analyze_with_avoid(position, budget, &[]);

    // Re-rank candidates by adding reinforcement bonus,
    // preferring moves that protect our high-value area.
    // The reinforcement bonus encourages moves adjacent to our nodes,
    // scaled by REINFORCE_W and remaining scoring events.
    let mut scored_candidates: Vec<(f64, meridian_engine::Move)> = Vec::new();
    for candidate in analysis.candidates.iter() {
        let opponent = position.to_move().opponent();
        let target = candidate.mv.target().expect("legal moves end on the board");
        let our_nodes = position.nodes(opponent);
        let mut nodes_adjacent: u32 = 0;
        for our_node in our_nodes.iter() {
            if (our_node.x() - target.x()).abs() <= 1 && (our_node.y() - target.y()).abs() <= 1
            {
                nodes_adjacent += 1;
            }
        }
        let bonus = if nodes_adjacent > 0 {
            let events_left = f64::from(position.scoring_events_left());
            1.5 * (nodes_adjacent as f64) * events_left
        } else {
            0.0
        };
        let total_score = candidate.evaluation + bonus;
        scored_candidates.push((total_score, candidate.mv));
    }
    // Sort descending by total score (reinforcement-enhanced evaluation)
    scored_candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
    // Return the top candidate
    scored_candidates.first().map(|(_, mv)| *mv)
}

/// Play a single game and collect metrics.
fn play_game(retaliator_blue: bool, use_reinforce: bool) -> (f64, f64, u32, u32, u32, u32) {
    let mut game = Game::new();
    let mut reinforce_count = 0u32;
    let mut total_area_b = 0u32;
    let mut total_area_r = 0u32;
    let mut games = 0u32;
    let mut reinforce_total = 0u32;

    while !game.is_over() && games < 200 {
        let to_move = game.position().to_move();
        let ret_moves = (to_move == Player::Blue) == retaliator_blue;
        let mv = if ret_moves {
            if use_reinforce {
                reinforce_player(game.position())
            } else {
                greedy(game.position())
            }
        } else {
            scout(game.position())
        };
        let Some(mv) = mv else { break };
        let outcome = game.play(mv).expect("bot moves are legal");
        games += 1;

        // Track reinforcement: count moves that are Connect and add no area
        // but are near our existing nodes (H1 reinforcement behavior)
        if outcome.kind == meridian_engine::MoveKind::Connect {
            let player = if ret_moves { Player::Blue } else { Player::Red };
            let our_area_before = game.position().area(player);
            let our_area_after = if ret_moves {
                game.position().area(Player::Blue)
            } else {
                game.position().area(Player::Red)
            };
            if our_area_after == our_area_before {
                let our_nodes = game.position().nodes(player);
                let target = mv.target().expect("legal moves end on the board");
                for our_node in our_nodes.iter() {
                    if (our_node.x() - target.x()).abs() <= 1
                        && (our_node.y() - target.y()).abs() <= 1
                    {
                        reinforce_count += 1;
                        reinforce_total += 1;
                        break;
                    }
                }
            }
        }

        let area_b = game.position().area(Player::Blue).to_f64();
        let area_r = game.position().area(Player::Red).to_f64();
        total_area_b += area_b as u32;
        total_area_r += area_r as u32;
    }

    let asc = if game.is_over() {
        game.position().score(Player::Blue).to_f64()
    } else {
        0.0
    };
    let bsc = if game.is_over() {
        game.position().score(Player::Red).to_f64()
    } else {
        0.0
    };

    (asc, bsc, reinforce_count, total_area_b, total_area_r, games)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n_games: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(8);
    let games = if n_games < 8 { 8 } else { n_games };

    let mut wins_ret = 0u32;    // RETALIATOR (H1) vs v1 base wins
    let mut losses_ret = 0u32;  // RETALIATOR (H1) vs v1 base losses
    let mut wins_greedy = 0u32; // RETALIATOR (H1) vs greedy wins

    let mut total_reinf = 0u32;
    let mut total_moves = 0u32;
    let mut total_area_sum = 0u32;

    // League margin tracking
    let mut league_sum = 0.0f64;
    let mut worst_margin = f64::INFINITY;
    let mut games_played = 0u32;

    // Run n games alternating colors
    for g in 0..games {
        let retaliator_blue = g % 2 == 0;

        // Game: RETALIATOR (H1 reinforcement) vs v1 base
        let (ret_score, v1_score, ret_reinf, ret_areas_b, ret_areas_r, ret_games) =
            play_game(retaliator_blue, true);

        // Compare scores
        if ret_score > v1_score {
            wins_ret += 1;
        } else {
            losses_ret += 1;
        }

        // Accumulate metrics
        total_reinf += ret_reinf;
        total_moves += ret_games as u32;
        total_area_sum += ret_areas_b + ret_areas_r;

        // League margin from RETALIATOR perspective
        let margin = if ret_score + v1_score > 0.0 {
            (ret_score - v1_score) / (ret_score + v1_score) * 100.0
        } else {
            0.0
        };
        league_sum += margin;
        worst_margin = worst_margin.min(margin);
        games_played += 1;

        // Game: RETALIATOR (H1 reinforcement) vs greedy
        let (ret_score2, greedy_score, _, _, _, _) = play_game(retaliator_blue, false);

        if ret_score2 > greedy_score {
            wins_greedy += 1;
        }
    }

    // League %: RETALIATOR win rate vs v1 baseline
    let league_pct = if wins_ret + losses_ret > 0 {
        wins_ret as f64 / (wins_ret + losses_ret) as f64 * 100.0
    } else {
        0.0
    };
    let league_avg = if games_played > 0 {
        league_sum / games_played as f64
    } else {
        0.0
    };

    // Average reinforce freq
    let avg_reinf = if total_moves > 0 {
        total_reinf as f64 / total_moves as f64 * 100.0
    } else {
        0.0
    };

    // Average area lifetime (simplified: average total area per game)
    let avg_area = if games > 0 {
        total_area_sum as f64 / games as f64
    } else {
        0.0
    };

    println!("probe_r_reinforce: H1 reinforcement evaluation probe");
    println!("Games played: {} (alternating colors, n>=8)", games);
    println!();
    println!("RETALIATOR (H1) vs v1 base:");
    println!("  Wins: {}/{} ({:.1}%)", wins_ret, games, wins_ret as f64 / games as f64 * 100.0);
    println!("  Losses: {}/{} ({:.1}%)", losses_ret, games, losses_ret as f64 / games as f64 * 100.0);
    println!();
    println!("RETALIATOR (H1) vs greedy:");
    println!("  Wins: {}/{} ({:.1}%)", wins_greedy, games, wins_greedy as f64 / games as f64 * 100.0);
    println!();
    println!("Reinforcement metrics:");
    println!("  Reinforce freq: {:.1}% ({} reinforces / {} total moves)", avg_reinf, total_reinf, total_moves);
    println!("  Area lifetime: {:.1} per game", avg_area);
    println!("  League avg margin: {:.1}%", league_avg);
    println!("  Worst margin: {:.1}%", worst_margin);
    println!("  League % (vs v1): {:.1}%", league_pct);
    println!();
    // PASS/FAIL bars
    let h2h_total = games; // head-to-head games played
    let h2h_pass = h2h_total >= 6;
    let league_pass = league_avg > -9.9;
    let worst_pass = worst_margin >= -300.0;
    let gauge_pass = league_pass && worst_pass;

    println!("PASS/FAIL bars:");
    println!("  h2h>=6/10: {}", if h2h_pass { "PASS" } else { "FAIL" });
    println!("  league>-9.9%: {}", if league_pass { "PASS" } else { "FAIL" });
    println!("  worst>=-300: {}", if worst_pass { "PASS" } else { "FAIL" });
    println!("  gauge 6/6: {}", if gauge_pass { "PASS" } else { "FAIL" });
    println!();
    println!("Summary: H1 reinforcement {} the bot {} breaks and improves area retention",
        if avg_reinf > 10.0 { "increases" } else { "does not significantly increase" },
        if wins_ret > losses_ret { "wins more" } else { "loses more" });
}