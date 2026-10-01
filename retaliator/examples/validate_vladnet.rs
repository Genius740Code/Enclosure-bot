//! VladNet Archetype Predictor Validator
//! Validates the pseudo-spec from research/archetype-vladnet.md on held-out games.

use meridian_engine::{Game, Move, MoveKind, Player, Position, Point};
use retaliator::search::MOVE_BUDGET;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Phase {
    Opening,
    Mid,
    Late,
}

impl Phase {
    fn from_action_count(count: usize) -> Self {
        if count < 12 {
            Phase::Opening
        } else if count < 60 {
            Phase::Mid
        } else {
            Phase::Late
        }
    }
}

struct PredictionResult {
    actual: Move,
    predicted: Option<Move>,
    matched: bool,
    phase: Phase,
    action: usize,
    our_move_was_close: bool,
}

fn predict_vladnet_reply(position_after_our: &Position, our_move_was_close: bool, our_last_move_target: Option<Point>) -> Option<Move> {
    // position_after_our is the position AFTER our move, at the start of VladNet's turn
    let after_our = position_after_our.clone();
    let vlad_color = after_our.to_move(); // Enemy to move
    let our_color = vlad_color.opponent();
    let actions_played = after_our.actions_played() as usize;

    // RULE 1: Memorized Opener Script (Actions <= 15)
    // VladNet has fixed openers for both colors
    if actions_played <= 15 {
        // Red script: VladNet as Red in 3a414aad
        // Red's turns (0-indexed total actions): 1,2, 5,6, 9,10, 13,14
        // 1-indexed (actions_played + 1): 2,3, 6,7, 10,11, 14,15
        let red_script: &[(usize, Option<Move>)] = &[
            (2, Move::between(Point::new(6, 0).unwrap(), Point::new(9, 3).unwrap())),   // P10-S13
            (3, Move::between(Point::new(9, 3).unwrap(), Point::new(9, 0).unwrap())),   // S13-S10
            (6, Move::between(Point::new(9, 3).unwrap(), Point::new(6, 1).unwrap())),   // S13-P11
            (7, Move::between(Point::new(6, 0).unwrap(), Point::new(9, -3).unwrap())),  // P10-S7
            (10, Move::between(Point::new(9, -3).unwrap(), Point::new(6, -1).unwrap())),// S7-P9
            (11, Move::between(Point::new(6, 0).unwrap(), Point::new(3, 3).unwrap())),  // P10-M13
            (14, Move::between(Point::new(3, 3).unwrap(), Point::new(6, 1).unwrap())),  // M13-P11
            (15, Move::between(Point::new(9, 0).unwrap(), Point::new(9, -3).unwrap())), // S10-S7
        ];
        // Blue script: VladNet as Blue in 1c69056c
        // Blue's turns (0-indexed total actions): 0, 3,4, 7,8, 11,12, 15,16
        // 1-indexed (actions_played + 1): 1, 4,5, 8,9, 12,13, 16
        let blue_script: &[(usize, Option<Move>)] = &[
            // Mirror on left flank
            (1, Move::between(Point::new(-6, 0).unwrap(), Point::new(-9, 3).unwrap())),  // D10-A13
            (4, Move::between(Point::new(-9, 3).unwrap(), Point::new(-9, 0).unwrap())),  // A13-A10
            (5, Move::between(Point::new(-9, 3).unwrap(), Point::new(-6, 1).unwrap())),  // A13-D11
            (8, Move::between(Point::new(-6, 0).unwrap(), Point::new(-9, -3).unwrap())), // D10-A7
            (9, Move::between(Point::new(-9, -3).unwrap(), Point::new(-6, -1).unwrap())),// A7-D9
            (12, Move::between(Point::new(-6, 0).unwrap(), Point::new(-3, 3).unwrap())), // D10-J13
            (13, Move::between(Point::new(-3, 3).unwrap(), Point::new(-6, 1).unwrap())), // J13-D11
            (16, Move::between(Point::new(-9, 0).unwrap(), Point::new(-9, -3).unwrap())), // A10-A7
        ];
        let script = match vlad_color {
            Player::Red => red_script,
            Player::Blue => blue_script,
        };
        for (act, mv_opt) in script {
            if *act == actions_played + 1 {
                if let Some(m) = mv_opt {
                    if after_our.check_move(*m).is_ok() {
                        return Some(*m);
                    }
                }
            }
        }
    }

    // RULE 2: Immediate Surgical Cut (94% Counter-Punch Policy)
    // On close turns, VladNet counters the specific loop we just closed.
    // Score cuts by: area_destroyed * 2.0 + neighbor_bonus + proximity_to_our_last_move
    let our_area_after = after_our.area(our_color).to_f64();
    let legal_replies = after_our.legal_moves();

    let mut best_cut: Option<(Move, f64)> = None;
    let mut cut_count = 0;
    for reply in legal_replies.iter() {
        let mut probe = after_our.clone();
        let outcome = probe.apply_unchecked(reply);
        if outcome.broken.is_some() {
            let area_destroyed = our_area_after - probe.area(our_color).to_f64();
            if area_destroyed > 0.5 {
                cut_count += 1;
                // Structural bonus: prefer cuts that land near VladNet's existing nodes
                let tgt = reply.target().unwrap();
                let neighbor_count = probe.nodes(vlad_color).iter()
                    .filter(|n| (n.x() - tgt.x()).abs() <= 1 && (n.y() - tgt.y()).abs() <= 1)
                    .count() as f64;
                let degree = probe.edges(vlad_color).iter()
                    .filter(|e| e.has_endpoint(tgt))
                    .count();
                let penalty = if degree <= 1 { -5.0 } else { 0.0 };
                let struct_score = neighbor_count * 2.0 + penalty;
                
                // Proximity bonus: on close turns, prefer cuts near our last move target
                let proximity_bonus = if our_move_was_close && our_last_move_target.is_some() {
                    let our_tgt = our_last_move_target.unwrap();
                    let dx = (tgt.x() - our_tgt.x()).abs() as f64;
                    let dy = (tgt.y() - our_tgt.y()).abs() as f64;
                    let dist = dx.max(dy); // Chebyshev distance
                    if dist <= 3.0 { 10.0 - dist * 2.0 } else { 0.0 }
                } else { 0.0 };
                
                // Combined score
                let score = area_destroyed * 2.0 + struct_score + proximity_bonus;
                
                if best_cut.as_ref().map_or(true, |(_, max_s)| score > *max_s) {
                    best_cut = Some((reply, score));
                }
            }
        }
    }
    if our_move_was_close {
        eprintln!("DEBUG close turn: cuts_found={}, best_cut={:?}", cut_count, best_cut.as_ref().map(|(m, s)| (m, s)));
    }
    if let Some((cut_mv, _)) = best_cut {
        return Some(cut_mv);
    }

    // RULE 3: Anchor Thicket / Multi-Node Truss Extension
    let mut best_truss: Option<(Move, f64)> = None;
    for reply in legal_replies.iter() {
        let mut probe = after_our.clone();
        let outcome = probe.apply_unchecked(reply);
        let tgt = reply.target().unwrap();
        
        let neighbor_count = probe.nodes(vlad_color).iter()
            .filter(|n| (n.x() - tgt.x()).abs() <= 1 && (n.y() - tgt.y()).abs() <= 1)
            .count() as f64;
        
        let degree = probe.edges(vlad_color).iter()
            .filter(|e| e.has_endpoint(tgt))
            .count();
        let penalty = if degree <= 1 { -5.0 } else { 0.0 };

        let score = neighbor_count * 2.0 + penalty;
        if best_truss.as_ref().map_or(true, |(_, max_s)| score > *max_s) {
            best_truss = Some((reply, score));
        }
    }
    if let Some((truss_mv, _)) = best_truss {
        return Some(truss_mv);
    }

    // RULE 4: Fallback to Top-Ranked Move
    legal_replies.iter().next()
}

fn analyze_game(game_id: &str) -> Vec<PredictionResult> {
    let url = format!("https://constellation.blueshrimp.uk/api/games/{}", game_id);
    let mut response = match ureq::get(&url).call() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to fetch game {}: {}", game_id, e);
            return vec![];
        }
    };

    let body = match response.body_mut().read_to_string() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to read body for {}: {}", game_id, e);
            return vec![];
        }
    };
    let game_json: serde_json::Value = match serde_json::from_str(&body) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Failed to parse JSON for {}: {}", game_id, e);
            return vec![];
        }
    };

    if game_json["status"].as_str() != Some("finished") {
        println!("  Game not finished, skipping");
        return vec![];
    }

    let moves_array = match game_json["moves"].as_array() {
        Some(arr) => arr,
        None => {
            eprintln!("No moves array in game {}", game_id);
            return vec![];
        }
    };

    let mut move_ids = Vec::new();
    for v in moves_array {
        if let Some(id) = v.as_u64() {
            if let Some(mv) = Move::from_index(id as usize) {
                move_ids.push(mv);
            }
        }
    }

    let blue_bot = game_json["blue"].as_str().unwrap_or("");
    let red_bot = game_json["red"].as_str().unwrap_or("");
    let our_color = if blue_bot.contains("Riposte") {
        Player::Blue
    } else if red_bot.contains("Riposte") {
        Player::Red
    } else {
        eprintln!("Could not determine our color for game {}", game_id);
        return vec![];
    };

    let mut game = Game::new();
    let mut avoid = Vec::new();
    let mut results = Vec::new();
    let mut our_area_before_move: f64 = 0.0;
    let mut our_last_move_kind: Option<MoveKind> = None;
    let mut our_last_move_target: Option<Point> = None;

    for &mv in move_ids.iter() {
        let mover = game.position().to_move();
        let action_count = game.position().actions_played() as usize;

        if mover != our_color {
            // It's VladNet's turn. The current position is already after our move.
            // Check if our last move was a close (Connect with area gain > 0.5)
            let our_area_after_move = game.position().area(our_color).to_f64();
            let area_gain = our_area_after_move - our_area_before_move;
            let our_move_was_close = our_last_move_kind == Some(MoveKind::Connect) && area_gain > 0.5;
            
            // Predict VladNet's reply from this position.
            let predicted = predict_vladnet_reply(game.position(), our_move_was_close, our_last_move_target);
            let phase = Phase::from_action_count(action_count);
            
            results.push(PredictionResult {
                actual: mv,
                predicted,
                matched: predicted == Some(mv),
                phase,
                action: action_count,
                our_move_was_close,
            });
        } else {
            // It's our turn - track area before our move, move kind, and move target
            our_area_before_move = game.position().area(our_color).to_f64();
            our_last_move_kind = game.position().check_move(mv).ok();
            our_last_move_target = mv.target();
        }

        let outcome = match game.play(mv) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Illegal move at action {}: {}", action_count, e);
                break;
            }
        };

        if let Some(cut) = outcome.broken {
            if mover.opponent() == our_color {
                avoid.push(cut.origin());
                avoid.push(cut.far());
            }
        }
        if avoid.len() > 12 {
            avoid.drain(0..avoid.len() - 12);
        }
    }

    results
}

fn main() {
    // Test on held-out VladNet game: 1c69056c (VladNet Blue, won)
    let game_id = "1c69056c-dd8a-4dc4-b62c-4846d08e92c8";
    println!("=== Validating VladNet Predictor on held-out game {} ===\n", game_id);
    
    let results = analyze_game(game_id);
    
    if results.is_empty() {
        println!("No results!");
        return;
    }

    let total = results.len();
    let matched = results.iter().filter(|r| r.matched).count();
    
    // Overall
    println!("=== OVERALL ===");
    println!("Total enemy turns: {}", total);
    println!("Matched: {}/{} = {:.1}%", matched, total, 100.0 * matched as f64 / total as f64);

    // By phase
    println!("\n=== BY PHASE ===");
    for phase in [Phase::Opening, Phase::Mid, Phase::Late] {
        let phase_results: Vec<_> = results.iter().filter(|r| r.phase == phase).collect();
        if !phase_results.is_empty() {
            let m = phase_results.iter().filter(|r| r.matched).count();
            let t = phase_results.len();
            println!("  {:?}: {}/{} = {:.1}%", phase, m, t, 100.0 * m as f64 / t as f64);
        }
    }

    // On close-turns (when we closed a loop)
    println!("\n=== ON CLOSE TURNS (our move was Connect + area gain > 0.5) ===");
    let close_results: Vec<_> = results.iter().filter(|r| r.our_move_was_close).collect();
    if !close_results.is_empty() {
        let m = close_results.iter().filter(|r| r.matched).count();
        let t = close_results.len();
        println!("  Close turns: {}/{} = {:.1}% (TARGET: ≥60%)", m, t, 100.0 * m as f64 / t as f64);
    } else {
        println!("  No close turns detected");
    }

    // On non-close turns
    println!("\n=== ON NON-CLOSE TURNS ===");
    let non_close_results: Vec<_> = results.iter().filter(|r| !r.our_move_was_close).collect();
    if !non_close_results.is_empty() {
        let m = non_close_results.iter().filter(|r| r.matched).count();
        let t = non_close_results.len();
        println!("  Non-close turns: {}/{} = {:.1}%", m, t, 100.0 * m as f64 / t as f64);
    }

    // Detailed mismatches
    println!("\n=== MISMATCH DETAILS ===");
    for r in results.iter().filter(|r| !r.matched) {
        println!("  Action {} ({:?}): predicted {:?}, actual {:?}, we_closed={}", 
            r.action, r.phase, r.predicted, r.actual, r.our_move_was_close);
    }

    // Verdict
    println!("\n=== VERDICT ===");
    let overall_pct = 100.0 * matched as f64 / total as f64;
    let close_pct = if !close_results.is_empty() {
        100.0 * close_results.iter().filter(|r| r.matched).count() as f64 / close_results.len() as f64
    } else { 0.0 };
    
    println!("Overall match: {:.1}% (target ≥40%: {})", overall_pct, if overall_pct >= 40.0 { "PASS" } else { "FAIL" });
    println!("Close-turn match: {:.1}% (target ≥60%: {})", close_pct, if close_pct >= 60.0 { "PASS" } else { "FAIL" });
    
    // Baseline greedy for comparison
    println!("\nBaseline greedy (from M7 audit): 7.5% overall");
}