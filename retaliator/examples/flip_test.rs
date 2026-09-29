//! Flip test: on real mid-game positions, compare `analyze` (2-ply fixed)
//! best pick vs `analyze_timed` best pick. Report flip %.

use meridian_engine::{Game, Move, Player};
use retaliator::search::{analyze, analyze_timed, MOVE_BUDGET};

fn main() {
    // Use the VladNet game positions (mid-game: actions 12-60)
    let game_id = "3a414aad-5aeb-4c16-87c1-f89e87966b93";
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
    let our_color = if blue_bot.contains("Riposte") || blue_bot.contains("v7") || blue_bot.contains("v6") {
        Player::Blue
    } else if red_bot.contains("Riposte") || red_bot.contains("v7") || red_bot.contains("v6") {
        Player::Red
    } else {
        eprintln!("Could not determine our color for game {}", game_id);
        return;
    };

    println!("Our color: {:?}", our_color);

    // Replay to mid-game positions (actions 12-60)
    let mut game = Game::new();
    let mut avoid = Vec::new();
    let mut test_positions = Vec::new();

    for &mv in move_ids.iter() {
        let mover = game.position().to_move();
        let action_count = game.position().actions_played() as usize;

        // Collect OUR positions in mid-game (actions 12-60)
        if mover == our_color && action_count >= 12 && action_count < 60 {
            test_positions.push((game.position().clone(), avoid.clone(), action_count));
            if test_positions.len() >= 20 {
                break;
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

    println!("Collected {} test positions", test_positions.len());

    let mut flips = 0;
    let mut total = 0;

    for (pos, avoid, action) in test_positions {
        let fixed = analyze(&pos, MOVE_BUDGET);
        let timed = analyze_timed(&pos, &avoid);

        let fixed_best = fixed.candidates.first().map(|c| c.mv);
        let timed_best = timed.candidates.first().map(|c| c.mv);

        let flipped = fixed_best != timed_best;
        if flipped {
            flips += 1;
            println!("FLIP at action {}: fixed={:?}, timed={:?} (depth={})", 
                action, fixed_best, timed_best, timed.depth);
        } else {
            println!("SAME at action {}: {:?} (timed depth={})", action, fixed_best, timed.depth);
        }
        total += 1;
    }

    println!("\n=== FLIP TEST RESULTS ===");
    println!("Total positions: {}", total);
    println!("Flipped: {} ({:.1}%)", flips, 100.0 * flips as f64 / total as f64);
    println!("Same: {} ({:.1}%)", total - flips, 100.0 * (total - flips) as f64 / total as f64);
}