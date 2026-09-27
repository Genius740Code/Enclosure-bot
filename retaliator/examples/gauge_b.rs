//! Gauge (Lane B): `eval_phases` (legal-cuts-only vulnerability) vs greedy
//! max-area 1-ply (the blob archetype), alternating colors. Reports scores,
//! margins, breaks. Same structure as `gauge.rs`, with the variant wired in.
//! Usage: cargo run --release --example gauge_b [games]

#[path = "../src/eval_phases.rs"]
mod eval_phases;
use meridian_engine::{Game, Move, Player, Position};

/// 1-ply greedy: maximize own enclosed area after the move, ties by move id.
fn greedy(position: &Position) -> Option<Move> {
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

fn play_game(laneb_blue: bool) -> (f64, f64, u32, u32) {
    let mut game = Game::new();
    let (mut breaks_r, mut breaks_g) = (0u32, 0u32);
    while !game.is_over() {
        let to_move = game.position().to_move();
        let laneb_moves =
            to_move == Player::Blue && laneb_blue || to_move == Player::Red && !laneb_blue;
        let mv = if laneb_moves {
            eval_phases::best_move(game.position())
        } else {
            greedy(game.position())
        };
        let Some(mv) = mv else { break };
        let outcome = game.play(mv).expect("bot moves are legal");
        if outcome.broken.is_some() {
            if laneb_moves {
                breaks_r += 1;
            } else {
                breaks_g += 1;
            }
        }
    }
    let b = game.position().score(Player::Blue).to_f64();
    let r = game.position().score(Player::Red).to_f64();
    (b, r, breaks_r, breaks_g)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let games: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(2);
    let mut wins_r = 0;
    for g in 0..games {
        let laneb_blue = g % 2 == 0;
        let t = std::time::Instant::now();
        let (b, r, breaks_r, breaks_g) = play_game(laneb_blue);
        let (rs, gs) = if laneb_blue { (b, r) } else { (r, b) };
        let margin = (rs - gs) / rs * 100.0;
        if rs > gs {
            wins_r += 1;
        }
        println!(
            "game {}: laneb={} blue={:.1} red={:.1} margin={:+.1}% breaks R={} G={} ({:.0}s)",
            g + 1,
            if laneb_blue { "blue" } else { "red" },
            b,
            r,
            margin,
            breaks_r,
            breaks_g,
            t.elapsed().as_secs_f64()
        );
    }
    println!("laneb wins: {wins_r}/{games}");
}
