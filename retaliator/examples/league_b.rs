//! League (Lane B): `eval_phases` vs scoutbase, 8 games (skip 0/10/20/30 x
//! 2 colors), same structure as `probe_league.rs` with the in-game variant
//! wired in. The skip prelude stays the shipped search's self-play
//! (unchanged openings, so the collapse-line rows stay comparable with the
//! scoreboard baseline); the controlled comparison is `probe_b_h2h`.
//!
//! B-3 farm-cycle routing (2026-09-28): OUR side's anti-rebuild avoid set
//! is fed live from the game (the `lib.rs` `replay()` wiring, memory
//! generalized). `B3_MEM` = the cut-memory window in actions — 0/absent =
//! no avoid = the published doom-OFF control (byte-identical); 6 = the
//! shipped site wiring; the B-3 sweep runs 12/20/40 (C2: farm cycles run
//! 20-40 actions, far beyond the shipped 6). `B3_SCALE` = 0 flat routing
//! only (the shipped form), 1 = + gain-scaled penalty keeping the
//! counter-cut exemption, 2 = + gain-scaled penalty on all first actions
//! on cut ground (the farm-cycle form; the C2 farmed re-closes are
//! close+cut combis). Usage:
//! `B3_MEM=20 B3_SCALE=2 cargo run --release --example league_b`

#[path = "../src/eval_phases.rs"]
mod eval_phases;
#[path = "support/scoutbase.rs"]
mod scoutbase;
use meridian_engine::{Edge, Game, Player, Point};

/// Endpoints of `me`'s edges cut within the last `mem` actions: the shipped
/// anti-rebuild wiring (`lib.rs` `replay()`), memory generalized (`B3_MEM`).
/// `mem == 0` = no avoid set (the control).
fn avoid_points(cuts: &[(u16, Player, Edge)], me: Player, played: u16, mem: usize) -> Vec<Point> {
    let mut avoid = Vec::new();
    if mem == 0 {
        return avoid;
    }
    for &(action, owner, cut) in cuts.iter().rev() {
        if played - action > mem as u16 {
            break;
        }
        if owner == me {
            avoid.push(cut.origin());
            avoid.push(cut.far());
        }
    }
    avoid
}

fn play(
    mut game: Game,
    ret_blue: bool,
    mem: usize,
    scale: u8,
    cuts: &mut Vec<(u16, Player, Edge)>,
) -> (f64, f64) {
    while !game.is_over() {
        let to_move = game.position().to_move();
        let played = u16::from(game.position().actions_played());
        let ret_moves = (to_move == Player::Blue) == ret_blue;
        let mv = if ret_moves {
            if mem == 0 {
                eval_phases::best_move(game.position())
            } else {
                let avoid = avoid_points(cuts, to_move, played, mem);
                match scale {
                    0 => eval_phases::best_move_with_avoid(game.position(), &avoid),
                    1 => eval_phases::farm_best_move_with_avoid(
                        game.position(),
                        &avoid,
                        eval_phases::Rebuild::ScaledExempt,
                    ),
                    _ => eval_phases::farm_best_move_with_avoid(
                        game.position(),
                        &avoid,
                        eval_phases::Rebuild::ScaledAll,
                    ),
                }
            }
        } else {
            scoutbase::best_move(game.position())
        };
        let oc = game.play(mv.unwrap()).unwrap();
        if let Some(cut) = oc.broken {
            cuts.push((played, to_move.opponent(), cut));
        }
    }
    let rs = game.position().score(if ret_blue { Player::Blue } else { Player::Red }).to_f64();
    let ss = game.position().score(if ret_blue { Player::Red } else { Player::Blue }).to_f64();
    (rs, ss)
}
fn main() {
    let mem: usize = std::env::var("B3_MEM").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let scale: u8 = std::env::var("B3_SCALE").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    println!("B3 config: mem={mem} scale={scale}");
    let mut sum = 0.0; let mut n = 0;
    for skip in [0usize, 10, 20, 30] {
        for ret_blue in [true, false] {
            let mut game = Game::new();
            let mut cuts: Vec<(u16, Player, Edge)> = Vec::new();
            for _ in 0..skip {
                if game.is_over() { break; }
                let played = u16::from(game.position().actions_played());
                let mover = game.position().to_move();
                let mv = retaliator::search::best_move(game.position()).unwrap();
                let oc = game.play(mv).unwrap();
                if let Some(cut) = oc.broken {
                    cuts.push((played, mover.opponent(), cut));
                }
            }
            if game.is_over() { continue; }
            let t = std::time::Instant::now();
            let (rs, ss) = play(game, ret_blue, mem, scale, &mut cuts);
            let m = (rs - ss) / rs * 100.0;
            println!("skip={skip} ret={} rs={rs:.0} ss={ss:.0} margin={m:+.1}% diff={:+.0} ({:.0}s)", if ret_blue { "blue" } else { "red" }, rs - ss, t.elapsed().as_secs_f64());
            sum += m; n += 1;
        }
    }
    println!("AVG margin (ret perspective): {:.1}%", sum / n as f64);
}
