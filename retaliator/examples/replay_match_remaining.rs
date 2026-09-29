//! Replay remaining finished games.

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
    by_opponent: HashMap<String, (usize, usize)>,
    by_phase: HashMap<Phase, (usize, usize)>,
    by_opponent_phase: HashMap<(String, Phase), (usize, usize)>,
}

fn main() {
    let games = vec![
        // AngelWASM - 1 remaining
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
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    println!("\n=== OVERALL (remaining) ===");
    print_stats("OVERALL", &overall);
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