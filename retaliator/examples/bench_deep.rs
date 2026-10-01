//! bench_deep: per-move time and nodes for search_deep at depths 2..max on
//! varied real positions, to calibrate depth against the clock (site gives
//! ~20s/move, hard kill ~23s). Also checks depth-2 agreement with the
//! 2-ply baseline on the same positions (implementation sanity check).
//! Usage: cargo run --release --example bench_deep [max_depth] [positions]

#[path = "../src/search_deep.rs"]
mod search_deep;

use meridian_engine::{Game, Move};

/// Greedy 1-ply: max own area after the move, ties by move id.
fn greedy(position: &meridian_engine::Position) -> Option<Move> {
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let max_depth: u8 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(5);
    let wanted: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(12);
    // Varied positions: greedy vs search self-play, one snapshot every 6 actions.
    let mut positions: Vec<Game> = Vec::new();
    let mut game = Game::new();
    let mut since = 0;
    while positions.len() < wanted {
        if game.is_over() {
            break;
        }
        let mv = if since % 2 == 0 {
            retaliator::search::best_move(game.position())
        } else {
            greedy(game.position())
        };
        game.play(mv.unwrap()).unwrap();
        since += 1;
        if since % 6 == 0 {
            positions.push(game.clone());
        }
    }
    println!("bench_deep: {} positions, depths 2..{max_depth}", positions.len());
    println!(
        "{:<10} {:>8} {:>10} {:>10} {:>8} {:>10}",
        "depth", "ms/move", "nodes", "max_nodes", "agree2", "eval"
    );
    for depth in 2..=max_depth {
        let (mut total, mut max_nodes, mut nodes, mut agree, mut n) =
            (0.0f64, 0u64, 0u64, 0usize, 0usize);
        for pos in &positions {
            let base_pick = retaliator::search::best_move(&pos.position());
            let t = std::time::Instant::now();
            let a = search_deep::analyze(&pos.position(), depth);
            total += t.elapsed().as_secs_f64() * 1000.0;
            nodes += a.nodes;
            max_nodes = max_nodes.max(a.nodes);
            n += 1;
            if a.candidates.first().map(|c| c.mv) == base_pick {
                agree += 1;
            }
        }
        println!(
            "{:<10} {:>8.0} {:>10} {:>10} {:>4}/{:<3}",
            depth,
            total / n as f64,
            (nodes as f64 / n as f64) as u64,
            max_nodes,
            agree,
            n
        );
    }
}
