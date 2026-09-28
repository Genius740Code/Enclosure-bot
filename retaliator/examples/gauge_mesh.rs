//! gauge_mesh: routed bot (mesh8 + search) vs greedy max-area 1-ply,
//! alternating colors, full games. The clean-gauge leg of the v6 gate.
//! Usage: cargo run --release --example gauge_mesh [games]
use meridian_engine::{Game, Move, Player, Position};

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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let games: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(6);
    let mut wins = 0;
    for g in 0..games {
        let ours = if g % 2 == 0 { Player::Blue } else { Player::Red };
        let mut game = Game::new();
        let t = std::time::Instant::now();
        while !game.is_over() {
            let mv = if game.position().to_move() == ours {
                retaliator::search::best_move_routed(game.position(), game.moves(), &[])
            } else {
                greedy(game.position())
            };
            let Some(mv) = mv else { break };
            game.play(mv).expect("legal");
            if game.moves().len() > 800 {
                break;
            }
        }
        let (rs, gs) = if ours == Player::Blue {
            (
                game.position().score(Player::Blue).to_f64(),
                game.position().score(Player::Red).to_f64(),
            )
        } else {
            (
                game.position().score(Player::Red).to_f64(),
                game.position().score(Player::Blue).to_f64(),
            )
        };
        let m = (rs - gs) / rs * 100.0;
        if rs > gs {
            wins += 1;
        }
        println!(
            "game {} as {}: {rs:.0}-{gs:.0} margin={m:+.1}% ({:.0}s)",
            g + 1,
            if ours == Player::Blue { "blue" } else { "red" },
            t.elapsed().as_secs_f64()
        );
    }
    println!("routed wins: {wins}/{games}");
}
