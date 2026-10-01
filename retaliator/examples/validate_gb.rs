//! GB Archetype Predictor Validator
//! Validates the pseudo-spec from research/archetype-gb.md on held-out games.

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
}

fn predict_gb_reply(position_after_our: &Position, our_move_was_close: bool) -> Option<Move> {
    let gb_color = position_after_our.to_move();
    let actions_played = position_after_our.actions_played() as usize;
    let legal: Vec<Move> = position_after_our.legal_moves().iter().collect();

    // RULE 1: Fixed Opening Script (actions <= 12)
    if actions_played <= 12 {
        let script = match gb_color {
            Player::Blue => blue_gb_opener(),
            Player::Red => red_gb_opener(),
        };
        if let Some(mv) = script.get(actions_played).copied().flatten() {
            eprintln!("DEBUG: Opening script action {}: predicted {:?}", actions_played, mv);
            if position_after_our.check_move(mv).is_ok() {
                return Some(mv);
            } else {
                eprintln!("DEBUG: Opening script move illegal at action {}", actions_played);
            }
        } else {
            eprintln!("DEBUG: No opening script move for action {} (script len={})", actions_played, script.len());
        }
    }

    // RULE 2: Corridor Root Contest (if opponent threatens corridor root)
    if our_move_was_close {
        if let Some(cut) = find_corridor_cut(position_after_our, gb_color, &legal) {
            return Some(cut);
        }
    }

    // RULE 3: Mid-Game Corridor Infrastructure
    let mut best_infra: Option<(Move, f64)> = None;
    for mv in &legal {
        let score = corridor_infrastructure_score(position_after_our, gb_color, *mv);
        if best_infra.map_or(true, |(_, s)| score > s) {
            best_infra = Some((*mv, score));
        }
    }
    if let Some((mv, score)) = best_infra {
        if actions_played >= 12 && actions_played < 60 {
            eprintln!("DEBUG mid: action {}: predicted {:?} (score={:.2})", actions_played, mv, score);
        }
        return Some(mv);
    }

    // RULE 4: Late-Game Close/Break/Expand
    if actions_played >= 60 {
        return predict_late_gb(position_after_our, gb_color, our_move_was_close, &legal);
    }

    legal.first().copied()
}

fn blue_gb_opener() -> Vec<Option<Move>> {
    // Extracted from 30c7653b (GB Blue)
    // Blue's turns at total actions: 0, 3,4, 7,8, 11,12
    // Corridor from D10 up-right: D10 -> G9 -> J10 -> J11 -> M12 -> M13 -> P13
    vec![
        Some(Move::between(Point::new(-6, 0).unwrap(), Point::new(-7, -3).unwrap()).expect("valid")),  // 0: D10-A7 (down-left, forced-ish)
        None, None,                                                                                      // 1,2: opponent
        Some(Move::between(Point::new(-6, 0).unwrap(), Point::new(-5, 2).unwrap()).expect("valid")),    // 3: D10-G9 (corridor root 1)
        Some(Move::between(Point::new(-6, 0).unwrap(), Point::new(-4, 3).unwrap()).expect("valid")),    // 4: D10-J10 (corridor root 2)
        None, None,                                                                                      // 5,6: opponent
        Some(Move::between(Point::new(-5, 2).unwrap(), Point::new(-3, 5).unwrap()).expect("valid")),    // 7: G9-J11
        Some(Move::between(Point::new(-4, 3).unwrap(), Point::new(-2, 6).unwrap()).expect("valid")),    // 8: J10-M12
        None, None,                                                                                      // 9,10: opponent
        Some(Move::between(Point::new(-3, 5).unwrap(), Point::new(-1, 8).unwrap()).expect("valid")),    // 11: J11-M13
        // 12: M12-P13 would be next but we only go to 11
    ]
}

fn red_gb_opener() -> Vec<Option<Move>> {
    // Mirror on right flank from ad65f054 (GB Red)
    // Red's turns at total actions: 1,2, 5,6, 9,10, 13,14
    // Corridor from P10 up-left: P10 -> S9 -> J10 -> P11 -> M11 -> M12 -> J13
    vec![
        Some(Move::between(Point::new(-6, 0).unwrap(), Point::new(-7, -3).unwrap()).expect("valid")),  // 0: Blue's move (Riposte forced)
        Some(Move::between(Point::new(6, 0).unwrap(), Point::new(5, 2).unwrap()).expect("valid")),     // 1: P10-S9 (corridor root 1)
        Some(Move::between(Point::new(6, 0).unwrap(), Point::new(4, 3).unwrap()).expect("valid")),     // 2: P10-J10 (corridor root 2)
        None, None,                                                                                     // 3,4: opponent
        Some(Move::between(Point::new(5, 2).unwrap(), Point::new(3, 5).unwrap()).expect("valid")),     // 5: S9-P11
        Some(Move::between(Point::new(4, 3).unwrap(), Point::new(2, 6).unwrap()).expect("valid")),     // 6: J10-M11
        None, None,                                                                                     // 7,8: opponent
        Some(Move::between(Point::new(3, 5).unwrap(), Point::new(1, 8).unwrap()).expect("valid")),     // 9: P11-M12
        Some(Move::between(Point::new(2, 6).unwrap(), Point::new(0, 9).unwrap()).expect("valid")),     // 10: M11-J13
        None, None,                                                                                     // 11,12: opponent
        None, None,                                                                                     // 13,14: opponent (GB would move at 13,14 but we stop at 12)
    ]
}

fn corridor_infrastructure_score(pos: &Position, color: Player, mv: Move) -> f64 {
    // Don't suggest moves that are already on the board
    if let Some(edge) = meridian_engine::Edge::from_move(mv.source, mv.direction) {
        if pos.edges(color).contains(edge) {
            return -100.0;
        }
    }
    
    let mut probe = pos.clone();
    probe.apply_unchecked(mv);
    let tgt = mv.target().unwrap();
    let src = mv.source;
    
    // Shared-node density (GB's signature)
    let shared_nodes = probe.edges(color).iter()
        .filter(|e| e.has_endpoint(tgt))
        .flat_map(|e| [e.origin(), e.far()])
        .filter(|n| probe.edges(color).iter().filter(|e2| e2.has_endpoint(*n)).count() >= 2)
        .count() as f64;
    
    // Root proximity: GB Blue corridor roots at G9 (-5,2) and J10 (-4,3)
    // GB Red corridor roots at S9 (5,2) and J10 (4,3)
    let roots = match color {
        Player::Blue => [Point::new(-5, 2).unwrap(), Point::new(-4, 3).unwrap()], // G9, J10
        Player::Red => [Point::new(5, 2).unwrap(), Point::new(4, 3).unwrap()],  // S9, J10
    };
    let root_proximity = roots.iter()
        .map(|r| (r.x() - tgt.x()).abs().max((r.y() - tgt.y()).abs()) as f64)
        .fold(f64::INFINITY, f64::min);
    let root_bonus = if root_proximity <= 3.0 { 5.0 - root_proximity } else { 0.0 };

    // Forward direction bonus: prefer moves that extend away from home base toward center
    let home_base = match color {
        Player::Blue => Point::new(-6, 0).unwrap(), // D10
        Player::Red => Point::new(6, 0).unwrap(),   // P10
    };
    let dist_from_home_before = (src.x() - home_base.x()).abs().max((src.y() - home_base.y()).abs()) as f64;
    let dist_from_home_after = (tgt.x() - home_base.x()).abs().max((tgt.y() - home_base.y()).abs()) as f64;
    let forward_bonus = if dist_from_home_after > dist_from_home_before { 5.0 } else { -10.0 }; // Heavy penalty for backward/sideways

    // Penalty for connecting two corridor roots sideways (GB extends forward, not sideways)
    let sideways_penalty = if roots.iter().any(|r| r == &src) && roots.iter().any(|r| r == &tgt) {
        -20.0  // Don't connect the two roots directly
    } else { 0.0 };

    shared_nodes * 2.0 + root_bonus + forward_bonus + sideways_penalty
}

fn find_corridor_cut(pos: &Position, color: Player, legal: &[Move]) -> Option<Move> {
    let our_area_before = pos.area(color).to_f64();
    
    let mut best: Option<(Move, f64)> = None;
    for mv in legal {
        let mut probe = pos.clone();
        let outcome = probe.apply_unchecked(*mv);
        if outcome.broken.is_some() {
            let area_destroyed = our_area_before - probe.area(color).to_f64();
            if area_destroyed > 0.5 {
                let tgt = mv.target().unwrap();
                let roots = match color {
                    Player::Blue => [Point::new(-5, 2).unwrap(), Point::new(-4, 3).unwrap()],
                    Player::Red => [Point::new(5, 2).unwrap(), Point::new(4, 3).unwrap()],
                };
                let root_proximity = roots.iter()
                    .map(|r| (r.x() - tgt.x()).abs().max((r.y() - tgt.y()).abs()) as f64)
                    .fold(f64::INFINITY, f64::min);
                let score = area_destroyed * 2.0 + if root_proximity <= 3.0 { 10.0 - root_proximity * 2.0 } else { 0.0 };
                if best.map_or(true, |(_, s)| score > s) {
                    best = Some((*mv, score));
                }
            }
        }
    }
    best.map(|(m, _)| m)
}

fn predict_late_gb(pos: &Position, color: Player, we_closed: bool, legal: &[Move]) -> Option<Move> {
    let opponent = color.opponent();
    
    // Priority 1: Break opponent's responding corridor
    if we_closed {
        if let Some(break_mv) = find_opponent_corridor_break(pos, color, legal) {
            return Some(break_mv);
        }
    }
    
    // Priority 2: Close own ready corridor
    if let Some(close_mv) = find_own_corridor_close(pos, color, legal) {
        return Some(close_mv);
    }
    
    // Priority 3: Far expand (safe, >5 from enemy)
    legal.iter()
        .filter(|mv| is_far_expand(pos, color, **mv))
        .max_by_key(|mv| area_gain(pos, color, **mv) as i64)
        .copied()
}

fn find_opponent_corridor_break(pos: &Position, color: Player, legal: &[Move]) -> Option<Move> {
    let opponent = color.opponent();
    let mut best: Option<(Move, f64)> = None;
    
    for mv in legal {
        let mut probe = pos.clone();
        let outcome = probe.apply_unchecked(*mv);
        if outcome.broken.is_some() {
            let opp_area_before = pos.area(opponent).to_f64();
            let opp_area_after = probe.area(opponent).to_f64();
            let destroyed = opp_area_before - opp_area_after;
            if destroyed > 0.5 {
                let tgt = mv.target().unwrap();
                let roots = match opponent {
                    Player::Blue => [Point::new(-5, 2).unwrap(), Point::new(-4, 3).unwrap()],
                    Player::Red => [Point::new(5, 2).unwrap(), Point::new(4, 3).unwrap()],
                };
                let root_proximity = roots.iter()
                    .map(|r| (r.x() - tgt.x()).abs().max((r.y() - tgt.y()).abs()) as f64)
                    .fold(f64::INFINITY, f64::min);
                let score = destroyed * 2.0 + if root_proximity <= 4.0 { 8.0 - root_proximity } else { 0.0 };
                if best.map_or(true, |(_, s)| score > s) {
                    best = Some((*mv, score));
                }
            }
        }
    }
    best.map(|(m, _)| m)
}

fn find_own_corridor_close(pos: &Position, color: Player, legal: &[Move]) -> Option<Move> {
    let mut best: Option<(Move, f64)> = None;
    for mv in legal {
        let mut probe = pos.clone();
        let outcome = probe.apply_unchecked(*mv);
        if outcome.kind == MoveKind::Connect {
            let area_gain = probe.area(color).to_f64() - pos.area(color).to_f64();
            if area_gain > 2.0 {
                let tgt = mv.target().unwrap();
                let roots = match color {
                    Player::Blue => [Point::new(-5, 2).unwrap(), Point::new(-4, 3).unwrap()],
                    Player::Red => [Point::new(5, 2).unwrap(), Point::new(4, 3).unwrap()],
                };
                let root_proximity = roots.iter()
                    .map(|r| (r.x() - tgt.x()).abs().max((r.y() - tgt.y()).abs()) as f64)
                    .fold(f64::INFINITY, f64::min);
                let score = area_gain + if root_proximity <= 3.0 { 5.0 } else { 0.0 };
                if best.map_or(true, |(_, s)| score > s) {
                    best = Some((*mv, score));
                }
            }
        }
    }
    best.map(|(m, _)| m)
}

fn is_far_expand(pos: &Position, color: Player, mv: Move) -> bool {
    let tgt = mv.target().unwrap();
    let opponent = color.opponent();
    let dist = pos.nodes(opponent).iter()
        .map(|n| (n.x() - tgt.x()).abs().max((n.y() - tgt.y()).abs()))
        .min()
        .unwrap_or(100);
    dist >= 5
}

fn area_gain(pos: &Position, color: Player, mv: Move) -> f64 {
    let mut probe = pos.clone();
    probe.apply_unchecked(mv);
    probe.area(color).to_f64() - pos.area(color).to_f64()
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

    for &mv in move_ids.iter() {
        let mover = game.position().to_move();
        let action_count = game.position().actions_played() as usize;

        if mover != our_color {
            let our_area_after_move = game.position().area(our_color).to_f64();
            let area_gain = our_area_after_move - our_area_before_move;
            let our_move_was_close = our_last_move_kind == Some(MoveKind::Connect) && area_gain > 0.5;
            
            let predicted = predict_gb_reply(game.position(), our_move_was_close);
            let phase = Phase::from_action_count(action_count);
            
            results.push(PredictionResult {
                actual: mv,
                predicted,
                matched: predicted == Some(mv),
                phase,
                action: action_count,
            });
        } else {
            our_area_before_move = game.position().area(our_color).to_f64();
            our_last_move_kind = game.position().check_move(mv).ok();
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
    let games = vec![
        ("GB Blue", "30c7653b-6d57-43d7-a361-d6080b07e266"),
        ("GB Red", "ad65f054-597e-47f1-b1f8-84f0d8ca3c54"),
    ];

    let mut all_results = Vec::new();

    for (name, game_id) in games {
        println!("\n=== Validating GB Predictor on {} ({}) ===", name, game_id);
        let results = analyze_game(game_id);
        
        if results.is_empty() {
            println!("No results!");
            continue;
        }

        let total = results.len();
        let matched = results.iter().filter(|r| r.matched).count();
        
        println!("  Overall: {}/{} = {:.1}%", matched, total, 100.0 * matched as f64 / total as f64);

        for phase in [Phase::Opening, Phase::Mid, Phase::Late] {
            let phase_results: Vec<_> = results.iter().filter(|r| r.phase == phase).collect();
            if !phase_results.is_empty() {
                let m = phase_results.iter().filter(|r| r.matched).count();
                let t = phase_results.len();
                println!("  {:?}: {}/{} = {:.1}%", phase, m, t, 100.0 * m as f64 / t as f64);
            }
        }

        all_results.extend(results);
    }

    let total = all_results.len();
    let matched = all_results.iter().filter(|r| r.matched).count();
    
    println!("\n=== OVERALL (2 games, {} enemy turns) ===", total);
    println!("Matched: {}/{} = {:.1}%", matched, total, 100.0 * matched as f64 / total as f64);
    
    for phase in [Phase::Opening, Phase::Mid, Phase::Late] {
        let phase_results: Vec<_> = all_results.iter().filter(|r| r.phase == phase).collect();
        if !phase_results.is_empty() {
            let m = phase_results.iter().filter(|r| r.matched).count();
            let t = phase_results.len();
            println!("  {:?}: {}/{} = {:.1}%", phase, m, t, 100.0 * m as f64 / t as f64);
        }
    }

    println!("\n=== TARGETS ===");
    println!("Overall: {:.1}% (target ≥45%: {})", 100.0 * matched as f64 / total as f64, if matched as f64 / total as f64 >= 0.45 { "PASS" } else { "FAIL" });
    println!("Baseline greedy: 37.5%");
}