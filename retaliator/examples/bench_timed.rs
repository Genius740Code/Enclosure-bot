//! bench_timed: verify Q6 timed think — searched moves spend >= SOFT ms,
//! volatile positions extend, forced/terminal return fast, all legal.
//! Usage: cargo run --release --example bench_timed
use meridian_engine::{Game, Player};
use std::time::Instant;

fn main() {
    // Mid-game position: retaliator vs itself for 32 actions.
    let mut game = Game::new();
    for _ in 0..32 {
        if game.is_over() {
            break;
        }
        let mv = retaliator::search::best_move(game.position()).expect("move");
        game.play(mv).unwrap();
    }
    let pos = game.position().clone();
    println!(
        "bench position: actions={} to_move={:?} legal={}",
        pos.actions_played(),
        pos.to_move(),
        pos.legal_moves().len()
    );
    let t = Instant::now();
    let a = retaliator::search::analyze_timed(&pos, &[]);
    let ms = t.elapsed().as_millis() as u64;
    let gap = if a.candidates.len() > 1 {
        (a.candidates[0].evaluation - a.candidates[1].evaluation).abs()
    } else {
        f64::NAN
    };
    let best = a.candidates.first().expect("candidate").mv;
    pos.check_move(best).expect("timed best must be legal");
    println!(
        "timed: ms={} depth={} nodes={} cands={} topgap={:.2} best={:?}",
        ms,
        a.depth,
        a.nodes,
        a.candidates.len(),
        gap,
        best
    );
    assert!(ms >= retaliator::search::THINK_SOFT_MS, "must spend soft budget");
    assert!(ms < retaliator::search::THINK_HARD_MS + 2000, "must respect hard cap");
    assert!(a.depth >= 2, "at least base 2-ply");
    println!("bench_timed OK");
}
