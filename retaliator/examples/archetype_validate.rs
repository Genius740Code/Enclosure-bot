//! Archetype Weighted Reply Prediction Validator
//! 
//! Tests archetype-specific predictors on held-out games and measures
//! weighted ensemble vs greedy baseline match rates per archetype.

use meridian_engine::{Game, Move, MoveKind, Player, Point};
use retaliator::search::MOVE_BUDGET;
use std::collections::HashMap;

mod archetype_profiles {
    include!("archetype_profiles/mod.rs");
}
use archetype_profiles::{Archetype, ArchetypeProfile, predict_archetype_reply, predict_weighted_reply};

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

#[derive(Clone)]
struct PredictionResult {
    actual: Move,
    predicted_greedy: Option<Move>,
    predicted_archetype: Option<Move>,
    predicted_weighted: Option<Move>,
    matched_greedy: bool,
    matched_archetype: bool,
    matched_weighted: bool,
    phase: Phase,
    action: usize,
    our_move_was_close: bool,
    archetype: Archetype,
}

fn analyze_game(game_id: &str, archetype: Archetype) -> Vec<PredictionResult> {
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
        println!("  Game {} not finished, skipping", game_id);
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
    
    // Determine which player matches the archetype
    let archetype_player = match archetype {
        Archetype::WallBuilder => {
            if blue_bot.to_lowercase().contains("capybara") { Player::Blue }
            else if red_bot.to_lowercase().contains("capybara") { Player::Red }
            else { 
                eprintln!("No capybara player found for WallBuilder in game {}", game_id);
                return vec![];
            }
        }
        Archetype::PopTimer => {
            if blue_bot.to_lowercase().contains("vlad") { Player::Blue }
            else if red_bot.to_lowercase().contains("vlad") { Player::Red }
            else { 
                eprintln!("No VladNet player found for PopTimer in game {}", game_id);
                return vec![];
            }
        }
        Archetype::SpaceGrabber => {
            if blue_bot.to_lowercase().contains("capybara") { Player::Blue }
            else if red_bot.to_lowercase().contains("capybara") { Player::Red }
            else { 
                eprintln!("No capybara player found for SpaceGrabber in game {}", game_id);
                return vec![];
            }
        }
        Archetype::Mirror => {
            if blue_bot.contains("YshanV1") { Player::Blue }
            else if red_bot.contains("YshanV1") { Player::Red }
            else { 
                eprintln!("No YshanV1 player found for Mirror in game {}", game_id);
                return vec![];
            }
        }
        Archetype::Unknown => {
            eprintln!("Unknown archetype");
            return vec![];
        }
    };

    let mut game = Game::new();
    let mut avoid = Vec::new();
    let mut results = Vec::new();
    let mut area_before_move: HashMap<Player, f64> = HashMap::new();
    let mut last_move_kind: HashMap<Player, MoveKind> = HashMap::new();
    let mut last_move_target: HashMap<Player, Point> = HashMap::new();

    for &mv in move_ids.iter() {
        let mover = game.position().to_move();
        let action_count = game.position().actions_played() as usize;

        if mover == archetype_player {
            // It's the archetype player's turn. Predict their move.
            let our_area_after_move = game.position().area(archetype_player).to_f64();
            let area_before = area_before_move.get(&archetype_player).copied().unwrap_or(0.0);
            let area_gain = our_area_after_move - area_before;
            let was_close = last_move_kind.get(&archetype_player) == Some(&MoveKind::Connect) && area_gain > 0.5;
            let last_target = last_move_target.get(&archetype_player).copied();
            
            // Greedy prediction: ranked().first() from our search
            let greedy_pred = retaliator::search::analyze_with_avoid(game.position(), 512, &avoid)
                .candidates.first().map(|c| c.mv);
            
            // Archetype-specific prediction
            let archetype_pred = predict_archetype_reply(
                archetype,
                game.position(),
                was_close,
                last_target,
                action_count,
            );
            
            // Weighted ensemble: equal weights for now
            let weights = vec![
                (Archetype::WallBuilder, 0.25),
                (Archetype::PopTimer, 0.25),
                (Archetype::SpaceGrabber, 0.25),
                (Archetype::Mirror, 0.25),
            ];
            let weighted_pred = predict_weighted_reply(
                &weights,
                game.position(),
                was_close,
                last_target,
                action_count,
            );

            let phase = Phase::from_action_count(action_count);
            
            results.push(PredictionResult {
                actual: mv,
                predicted_greedy: greedy_pred,
                predicted_archetype: archetype_pred,
                predicted_weighted: weighted_pred,
                matched_greedy: greedy_pred == Some(mv),
                matched_archetype: archetype_pred == Some(mv),
                matched_weighted: weighted_pred == Some(mv),
                phase,
                action: action_count,
                our_move_was_close: was_close,
                archetype,
            });
        }

        // Track area before move for both players
        area_before_move.insert(mover, game.position().area(mover).to_f64());
        
        // Play the move
        let outcome = match game.play(mv) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Illegal move at action {}: {}", action_count, e);
                break;
            }
        };

        // Track move kind and target
        let moved_pos = game.position(); // position after move
        // We can't easily get MoveKind from outcome, so approximate
        let is_connect = outcome.broken.is_none() && mv.target().is_some(); // rough heuristic
        last_move_kind.insert(mover, if is_connect { MoveKind::Connect } else { MoveKind::Extend });
        last_move_target.insert(mover, mv.target().unwrap());

        if let Some(cut) = outcome.broken {
            // Track cuts for avoid list
            avoid.push(cut.origin());
            avoid.push(cut.far());
        }
        if avoid.len() > 12 {
            avoid.drain(0..avoid.len() - 12);
        }
    }

    results
}

fn print_results(archetype: Archetype, results: &[PredictionResult]) {
    if results.is_empty() {
        println!("No results for {:?}!", archetype);
        return;
    }

    let total = results.len();
    let greedy_matched = results.iter().filter(|r| r.matched_greedy).count();
    let archetype_matched = results.iter().filter(|r| r.matched_archetype).count();
    let weighted_matched = results.iter().filter(|r| r.matched_weighted).count();

    println!("\n=== {:?} ({} games, {} enemy turns) ===", archetype, 
        results.iter().map(|r| r.action).max().unwrap_or(0) / 120 + 1, total);
    
    println!("Greedy baseline:     {}/{} = {:.1}%", greedy_matched, total, 100.0 * greedy_matched as f64 / total as f64);
    println!("Archetype-specific:  {}/{} = {:.1}%", archetype_matched, total, 100.0 * archetype_matched as f64 / total as f64);
    println!("Weighted ensemble:   {}/{} = {:.1}%", weighted_matched, total, 100.0 * weighted_matched as f64 / total as f64);

    // By phase
    println!("\n--- BY PHASE ---");
    for phase in [Phase::Opening, Phase::Mid, Phase::Late] {
        let phase_results: Vec<_> = results.iter().filter(|r| r.phase == phase).collect();
        if !phase_results.is_empty() {
            let g = phase_results.iter().filter(|r| r.matched_greedy).count();
            let a = phase_results.iter().filter(|r| r.matched_archetype).count();
            let w = phase_results.iter().filter(|r| r.matched_weighted).count();
            let t = phase_results.len();
            println!("  {:?}: Greedy {}/{}={:.1}% | Arch {}/{}={:.1}% | Wgt {}/{}={:.1}%", 
                phase, g, t, 100.0*g as f64/t as f64, a, t, 100.0*a as f64/t as f64, w, t, 100.0*w as f64/t as f64);
        }
    }

    // On close-turns
    println!("\n--- ON CLOSE TURNS (Connect + area gain > 0.5) ---");
    let close_results: Vec<_> = results.iter().filter(|r| r.our_move_was_close).collect();
    if !close_results.is_empty() {
        let g = close_results.iter().filter(|r| r.matched_greedy).count();
        let a = close_results.iter().filter(|r| r.matched_archetype).count();
        let w = close_results.iter().filter(|r| r.matched_weighted).count();
        let t = close_results.len();
        println!("  Close: Greedy {}/{}={:.1}% | Arch {}/{}={:.1}% | Wgt {}/{}={:.1}%", 
            g, t, 100.0*g as f64/t as f64, a, t, 100.0*a as f64/t as f64, w, t, 100.0*w as f64/t as f64);
    }

    // On non-close turns
    println!("\n--- ON NON-CLOSE TURNS ---");
    let non_close_results: Vec<_> = results.iter().filter(|r| !r.our_move_was_close).collect();
    if !non_close_results.is_empty() {
        let g = non_close_results.iter().filter(|r| r.matched_greedy).count();
        let a = non_close_results.iter().filter(|r| r.matched_archetype).count();
        let w = non_close_results.iter().filter(|r| r.matched_weighted).count();
        let t = non_close_results.len();
        println!("  Non-close: Greedy {}/{}={:.1}% | Arch {}/{}={:.1}% | Wgt {}/{}={:.1}%", 
            g, t, 100.0*g as f64/t as f64, a, t, 100.0*a as f64/t as f64, w, t, 100.0*w as f64/t as f64);
    }
}

fn main() {
    // Use available finished games from API
    // Archetype mapping:
    // - WallBuilder: GB-like (use capybara as proxy, similar wall-building style)
    // - PopTimer: VladNet
    // - SpaceGrabber: capybara vs YshanV1 (space-grab / expansion style)
    // - Mirror: YshanV1 family (our search family)
    let test_games = vec![
        // WallBuilder proxy (capybara wall-building games vs YshanV1-Plus-Lite) - 1 game each color
        ("d8cfb9b1-373e-4bf4-a274-17a14ec97e4a", Archetype::WallBuilder), // capybara vs YshanV1-Plus-Lite
        ("99a02dad-6edb-40d8-9678-34af03f2ad89", Archetype::WallBuilder), // YshanV1-Plus-Lite vs capybara
        
        // PopTimer (VladNet) - 1 game each color
        ("49352c59-892c-4a85-b022-605b3d6753f1", Archetype::PopTimer),   // VladNet vs capybara
        ("2cd5f9db-9392-4a02-984d-8aea605554b1", Archetype::PopTimer),   // capybara vs VladNet
        
        // SpaceGrabber (capybara vs YshanV1-Lite-PUCT - expansion style) - 1 game each color
        ("19e27551-d22a-41f9-b36a-2dc3575331a6", Archetype::SpaceGrabber), // capybara vs YshanV1-Lite-PUCT
        ("80a192d6-23f2-4d0c-919b-adaccce2b0cc", Archetype::SpaceGrabber), // YshanV1-Lite-PUCT vs capybara
        
        // Mirror (YshanV1 family) - 1 game each
        ("5ccf21e0-411a-4102-9503-4351f7f090d1", Archetype::Mirror),     // YshanV1-Plus-Lite (solo)
        ("f3fe101d-5bb3-4c1b-93c3-8621263a4c51", Archetype::Mirror),     // YshanV1-Lite-PUCT vs YshanV1-Plus-Lite
    ];

    let mut all_results: Vec<PredictionResult> = Vec::new();
    let mut by_archetype: HashMap<Archetype, Vec<PredictionResult>> = HashMap::new();

    for (game_id, archetype) in test_games {
        println!("\n========================================");
        println!("Analyzing game {} ({:?})", game_id, archetype);
        println!("========================================");
        
        let results = analyze_game(game_id, archetype);
        if results.is_empty() {
            println!("  No results (game may not be finished or fetch failed)");
            continue;
        }
        
        print_results(archetype, &results);
        
        all_results.extend(results.clone());
        by_archetype.entry(archetype).or_default().extend(results);
    }

    // Overall summary
    println!("\n========================================");
    println!("OVERALL SUMMARY");
    println!("========================================");
    
    let total = all_results.len();
    let greedy = all_results.iter().filter(|r| r.matched_greedy).count();
    let archetype = all_results.iter().filter(|r| r.matched_archetype).count();
    let weighted = all_results.iter().filter(|r| r.matched_weighted).count();
    
    println!("Total enemy turns analyzed: {}", total);
    println!("Greedy baseline:            {:.1}%", 100.0 * greedy as f64 / total as f64);
    println!("Archetype-specific:         {:.1}%", 100.0 * archetype as f64 / total as f64);
    println!("Weighted ensemble:          {:.1}%", 100.0 * weighted as f64 / total as f64);
    
    // Per-archetype summary
    println!("\n--- PER ARCHETYPE ---");
    for arch in [Archetype::WallBuilder, Archetype::PopTimer, Archetype::SpaceGrabber, Archetype::Mirror] {
        if let Some(results) = by_archetype.get(&arch) {
            let t = results.len();
            let g = results.iter().filter(|r| r.matched_greedy).count();
            let a = results.iter().filter(|r| r.matched_archetype).count();
            let w = results.iter().filter(|r| r.matched_weighted).count();
            println!("  {:?}: Greedy {:.1}% | Arch {:.1}% | Wgt {:.1}% (n={})", 
                arch, 100.0*g as f64/t as f64, 100.0*a as f64/t as f64, 100.0*w as f64/t as f64, t);
        }
    }
    
    // Verdict
    println!("\n=== VERDICT ===");
    let overall_weighted = 100.0 * weighted as f64 / total as f64;
    println!("Overall weighted match: {:.1}% (target >55%: {})", 
        overall_weighted, if overall_weighted > 55.0 { "PASS" } else { "FAIL" });
    
    let mut all_arch_pass = true;
    for arch in [Archetype::WallBuilder, Archetype::PopTimer, Archetype::SpaceGrabber, Archetype::Mirror] {
        if let Some(results) = by_archetype.get(&arch) {
            let t = results.len();
            let a = results.iter().filter(|r| r.matched_archetype).count();
            let w = results.iter().filter(|r| r.matched_weighted).count();
            let arch_pct = 100.0 * a as f64 / t as f64;
            let wgt_pct = 100.0 * w as f64 / t as f64;
            let baseline = ArchetypeProfile::all().into_iter()
                .find(|p| p.archetype == arch)
                .map(|p| p.overall_match * 100.0)
                .unwrap_or(0.0);
            let beats_baseline = wgt_pct > baseline;
            println!("  {:?}: weighted {:.1}% vs baseline {:.1}% -> {}", 
                arch, wgt_pct, baseline, if beats_baseline { "UP" } else { "DOWN" });
            if !beats_baseline {
                all_arch_pass = false;
            }
        }
    }
    println!("All archetypes beat baseline: {}", if all_arch_pass { "YES" } else { "NO" });
    
    if overall_weighted > 55.0 && all_arch_pass {
        println!("\n✅ GATE PASSED: Ready to wire dose behind toggle (default OFF)");
    } else {
        println!("\n❌ GATE FAILED: Need better predictors or more training data");
    }
}