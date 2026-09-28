//! site_check: end-to-end deployed path (answer() with start+moves JSON).
//! 1. Empty start -> must reply D10-D13 (16058, prefix fires on site path).
//! 2. Mid-game (start0 + move list) -> legal move replies, both colors' turns.
//! 3. Analysis request -> well-formed.
//! Usage: cargo run --release --example site_check
use meridian_engine::{Game, Move, notation};
use serde_json::json;

fn ask(start: &str, moves: &[usize]) -> serde_json::Value {
    let req = json!({"type": "move", "start": start, "moves": moves});
    retaliator::answer(&req)
}

fn main() {
    // 1. Site opening: empty board, no history.
    let g0 = Game::new();
    let start0 = notation::setup_text(g0.position());
    let r = ask(&start0, &[]);
    let id = r["move"].as_u64().expect("move reply") as usize;
    println!("opening reply: {id}");
    assert_eq!(id, 16058, "prefix slot 0 must fire on the site path");

    // 2. Play a line through answer(): us routed both colors (alternate by
    // to_move), scout-ish replies via best_move for the other side.
    let mut game = Game::new();
    let mut ids: Vec<usize> = Vec::new();
    for _ in 0..12 {
        let r = ask(&start0, &ids);
        let id = r["move"].as_u64().expect("legal reply") as usize;
        let mv = Move::from_index(id).expect("id decodes");
        game.play(mv).unwrap_or_else(|e| panic!("site reply illegal: {id} {e}"));
        ids.push(id);
    }
    println!("12 site-path replies, all legal. moves={ids:?}");

    // 3. Analysis request on the mid-game position.
    let req = json!({"type": "analysis", "start": start0, "moves": ids, "limits": {"visits": 240}});
    let a = retaliator::answer(&req);
    assert!(a["analysis"]["candidates"].as_array().is_some(), "analysis shape");
    println!("analysis OK, depth={}", a["analysis"]["depth"]);
    println!("site_check OK");
}
