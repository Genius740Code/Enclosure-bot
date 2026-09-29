//! Replay finished games and measure reply-match rate: how often does the
//! enemy's ACTUAL move equal our `ranked()` predicted reply (via analyze_with_avoid).

use std::collections::HashMap;
use meridian_engine::{Game, Move, Player};
use retaliator::search::{analyze_with_avoid, MOVE_BUDGET};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Default)]
struct Stats {
    total: usize,
    matched: usize,
    by_opponent: HashMap<String, (usize, usize)>, // (matched, total)
    by_phase: HashMap<Phase, (usize, usize)>,
    by_opponent_phase: HashMap<(String, Phase), (usize, usize)>,
}

fn main() {
    // Game IDs to analyze: (opponent_archetype, game_id)
    let games = vec![
        // Mirror (v6-vs-v7) - 10 finished games
        ("Mirror", "b4462808-e94a-4269-9ecf-80447d4972b1"),
        ("Mirror", "434027fd-0ff1-434e-bd68-a0f57f22e8f7"),
        ("Mirror", "c3c8cf5b-858d-40b4-a721-e7aca788d8bf"),
        ("Mirror", "21fc2945-7a56-4fe5-ba19-3604e7357a6f"),
        ("Mirror", "16840108-93ce-4c18-934d-3590e4cc693f"),
        ("Mirror", "c20c750b-3c06-49af-8da8-6eb0e4a76dea"),
        ("Mirror", "f81e3b7e-e198-4a1f-8625-53605874bd1a"),
        ("Mirror", "2f6343a3-a167-4ccb-bb37-9a1ccbf78d9a"),
        ("Mirror", "0608aa58-c364-4d0f-a1bc-b0585efe5e50"),
        ("Mirror", "3a8cb06a-cefe-4b45-86f9-d67b8694c82b"),
        // VladNet - 2 games
        ("VladNet", "3a414aad-5aeb-4c16-87c1-f89e87966b93"),
        ("VladNet", "1c69056c-dd8a-4dc4-b62c-4846d08e92c8"),
        // GB - 2 games
        ("GB", "30c7653b-6d57-43d7-a361-d6080b07e266"),
        ("GB", "ad65f054-597e-47f1-b1f8-84f0d8ca3c54"),
        // GB2.0 - 2 games
        ("GB2.0", "b85bd6df-67ea-453f-95e5-db1890547d1c"),
        ("GB2.0", "d2d4b4fd-9713-4fdc-82d9-26232a31784a"),
        // AngelWASM - 2 games
        ("AngelWASM", "aa0193b0-1e9e-4d72-ac02-e7b0e367dff9"),
        ("AngelWASM", "17a73c5b-ec53-4f69-b593-4d202d07c8fc"),
        // Stompy - 2 games
        ("Stompy", "9b50e36f-6515-4050-9e59-d263d2e844f8"),
        ("Stompy", "b0ac4141-0c5f-407c-b1d2-9e35be9c5a58"),
        // v3 - 2 games
        ("v3", "6f0198c7-7c3f-4f48-bfa9-608565c3881b"),
        ("v3", "e02faf40-e395-4e07-a0de-cd58b4cbe017"),
    ];

    let mut overall = Stats::default();

    for (opponent_name, game_id) in games {
        println!("\n=== Analyzing {} (game {}) ===", opponent_name, game_id);
        let mut stats = Stats::default();
        analyze_game(game_id, opponent_name, &mut stats);
        print_stats(opponent_name, &stats);
        merge_stats(&mut overall, &stats);
        
        // Rate limiting: small delay between requests
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    println!("\n=== OVERALL ===");
    print_stats("OVERALL", &overall);
    
    println!("\n=== BY OPPONENT & PHASE ===");
    for phase in [Phase::Opening, Phase::Mid, Phase::Late] {
        println!("  {:?}:", phase);
        let mut opps: Vec<_> = overall.by_opponent_phase.keys()
            .filter(|(_, p)| *p == phase)
            .map(|(o, _)| o.clone())
            .collect();
        opps.sort();
        opps.dedup();
        for opp in opps {
            if let Some((m, t)) = overall.by_opponent_phase.get(&(opp.clone(), phase)) {
                if *t > 0 {
                    println!("    {}: {}/{} = {:.1}%", opp, m, t, 100.0 * *m as f64 / *t as f64);
                }
            }
        }
    }
}

fn analyze_game(game_id: &str, opponent_name: &str, stats: &mut Stats) {
    let url = format!("https://constellation.blueshrimp.uk/api/games/{}", game_id);
    let mut response = match ureq::get(&url).call() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to fetch game {}: {}", game_id, e);
            return;
        }
    };

    let game_json: serde_json::Value = match response.body_mut().read_json() {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Failed to parse JSON for {}: {}", game_id, e);
            return;
        }
    };

    if game_json["status"].as_str() != Some("finished") {
        println!("  Game not finished, skipping");
        return;
    }

    let moves_array = match game_json["moves"].as_array() {
        Some(arr) => arr,
        None => {
            eprintln!("No moves array in game {}", game_id);
            return;
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
        return;
    };

    // println!("  Our color: {:?}, Opponent: {}", our_color, opponent_name);

    let mut game = Game::new();
    let mut avoid = Vec::new();

    for &mv in move_ids.iter() {
        let mover = game.position().to_move();
        let action_count = game.position().actions_played() as usize;

        if mover != our_color {
            let analysis = analyze_with_avoid(game.position(), MOVE_BUDGET, &avoid);
            let predicted = analysis.candidates.first().map(|c| c.mv);
            let actual = Some(mv);

            let phase = Phase::from_action_count(action_count);
            let matched = predicted == actual;

            *stats.by_opponent.entry(opponent_name.to_string()).or_default() = {
                let (m, t) = stats.by_opponent.get(opponent_name).copied().unwrap_or((0, 0));
                (m + matched as usize, t + 1)
            };
            *stats.by_phase.entry(phase).or_default() = {
                let (m, t) = stats.by_phase.get(&phase).copied().unwrap_or((0, 0));
                (m + matched as usize, t + 1)
            };
            *stats.by_opponent_phase.entry((opponent_name.to_string(), phase)).or_default() = {
                let (m, t) = stats.by_opponent_phase.get(&(opponent_name.to_string(), phase)).copied().unwrap_or((0, 0));
                (m + matched as usize, t + 1)
            };
            stats.total += 1;
            if matched {
                stats.matched += 1;
            }
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

    // println!("  Total enemy turns analyzed: {}", stats.total);
    // println!("  Matched: {} ({:.1}%)", stats.matched, 100.0 * stats.matched as f64 / stats.total.max(1) as f64);
}

fn print_stats(label: &str, stats: &Stats) {
    if stats.total == 0 {
        println!("{} - No data", label);
        return;
    }
    println!("{} - Overall: {}/{} = {:.1}%", label, stats.matched, stats.total, 100.0 * stats.matched as f64 / stats.total as f64);
    println!("  By opponent:");
    for (opp, (matched, total)) in &stats.by_opponent {
        println!("    {}: {}/{} = {:.1}%", opp, matched, total, 100.0 * *matched as f64 / *total as f64);
    }
    println!("  By phase:");
    for phase in [Phase::Opening, Phase::Mid, Phase::Late] {
        if let Some((matched, total)) = stats.by_phase.get(&phase) {
            if *total > 0 {
                println!("    {:?}: {}/{} = {:.1}%", phase, matched, total, 100.0 * *matched as f64 / *total as f64);
            }
        }
    }
}

fn merge_stats(overall: &mut Stats, new: &Stats) {
    overall.total += new.total;
    overall.matched += new.matched;
    for (opp, (m, t)) in &new.by_opponent {
        let (om, ot) = overall.by_opponent.entry(opp.clone()).or_default();
        *om += m;
        *ot += t;
    }
    for (phase, (m, t)) in &new.by_phase {
        let (om, ot) = overall.by_phase.entry(*phase).or_default();
        *om += m;
        *ot += t;
    }
    for ((opp, phase), (m, t)) in &new.by_opponent_phase {
        let (om, ot) = overall.by_opponent_phase.entry((opp.clone(), *phase)).or_default();
        *om += m;
        *ot += t;
    }
}