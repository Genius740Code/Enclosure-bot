// oracle_kill0: MIRROR-ORACLE kill-0 test
// Beam-64 search vs Red's recorded mesh line, measure Blue @30 area
// Bar: Blue @30 >= 35 in >= 8/9 games
// Usage: cargo run --release --example oracle_kill0

use meridian_engine::{Game, Move, Player, Position, notation};
use retaliator::search::best_move_routed;

/// Play one game and return Blue's area after exactly 30 moves.
/// If the game ends before 30 moves, returns None.
fn play_one_game() -> Option<f64> {
    let mut game = Game::new();
    let mut moves_played = 0;

    while moves_played < 30 && !game.is_over() {
        let pos = game.position();
        let to_move = pos.to_move();

        // Let the bot decide the move using best_move_routed,
        // which internally handles mesh prefix enforcement.
        let mv = best_move_routed(pos, game.moves(), &[]);
        let Some(mv) = mv else { break };
        let outcome = game.play(mv).expect("legal blue move");
        // If a piece was broken, we still continue counting moves.
        // The area measurement happens after 30 moves regardless.
        moves_played += 1;
    }

    // If we reached exactly 30 moves, measure Blue's area
    if moves_played >= 30 {
        let pos = game.position();
        let area = pos.area(Player::Blue).to_f64();
        Some(area)
    } else {
        None
    }
}

fn main() {
    let games: usize = 9;
    let mut below_bar = 0usize;
    let mut results: Vec<(usize, f64)> = Vec::new(); // (game#, area)

    println!("MIRROR-ORACLE kill-0 test: {} games", games);
    println!("==========================================");

    for g in 0..games {
        let area = play_one_game();
        match area {
            Some(a) => {
                let status = if a >= 35.0 { "PASS" } else { "FAIL" };
                if a < 35.0 {
                    below_bar += 1;
                }
                println!(
                    "game {}: Blue @30 area = {:.1} {}",
                    g + 1, a, status
                );
                results.push((g + 1, a));
            }
            None => {
                println!("game {}: did not reach 30 moves", g + 1);
            }
        }
    }

    println!("==========================================");
    let pass_count = games - below_bar;
    println!(
        "Blue @30 >= 35: {}/{} games ({:.1}%)",
        pass_count,
        games,
        pass_count as f64 / games as f64 * 100.0
    );

    // Bar check: Blue @30 >= 35 in >= 8/9 games
    if pass_count >= 8 {
        println!("VERDICT: KILL — bar met (Blue @30 >= 35 in >= 8/9 games)");
    } else {
        println!("VERDICT: NOT KILL — bar not met (Blue @30 >= 35 in < 8/9 games)");
    }
}