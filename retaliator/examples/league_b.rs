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
use meridian_engine::{Edge, Game, Move, MoveKind, Player, Point};

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

#[derive(Default)]
struct MissedCloses {
    ret_pot_sum: u32,
    ret_turns: u32,
    ret_closes: u32,
    scout_pot_sum: u32,
    scout_turns: u32,
    scout_closes: u32,
}

fn play(
    mut game: Game,
    ret_blue: bool,
    mem: usize,
    scale: u8,
    cuts: &mut Vec<(u16, Player, Edge)>,
) -> (f64, f64, MissedCloses) {
    let mut mc = MissedCloses::default();
    while !game.is_over() {
        let to_move = game.position().to_move();
        let played = u16::from(game.position().actions_played());
        let ret_moves = (to_move == Player::Blue) == ret_blue;
        
        // Track one-move potential at start of turn
        let pot_me = eval_phases::one_move_potential(game.position(), to_move);
        let pot_opp = eval_phases::one_move_potential(game.position(), to_move.opponent());
        if ret_moves {
            mc.ret_pot_sum += pot_me;
            mc.ret_turns += 1;
        } else {
            mc.scout_pot_sum += pot_me;
            mc.scout_turns += 1;
        }
        
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
        let mv = mv.unwrap();
        let area_before = game.position().area(to_move).to_f64();
        let oc = game.play(mv).unwrap();
        let area_after = game.position().area(to_move).to_f64();
        let gain = area_after - area_before;
        
        // Count area-gaining Connect moves as "closes"
        if oc.kind == MoveKind::Connect && gain > 0.0 {
            if ret_moves {
                mc.ret_closes += 1;
            } else {
                mc.scout_closes += 1;
            }
        }
        
        if let Some(cut) = oc.broken {
            cuts.push((played, to_move.opponent(), cut));
        }
    }
    let rs = game.position().score(if ret_blue { Player::Blue } else { Player::Red }).to_f64();
    let ss = game.position().score(if ret_blue { Player::Red } else { Player::Blue }).to_f64();
    (rs, ss, mc)
}
fn main() {
    let mem: usize = std::env::var("B3_MEM").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let scale: u8 = std::env::var("B3_SCALE").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    println!("B3 config: mem={mem} scale={scale}");
    let mut sum = 0.0; let mut n = 0;
    let mut total_ret_missed = 0.0;
    let mut total_scout_missed = 0.0;
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
            let (rs, ss, mc) = play(game, ret_blue, mem, scale, &mut cuts);
            let m = (rs - ss) / rs * 100.0;
            let ret_avg_pot = if mc.ret_turns > 0 { mc.ret_pot_sum as f64 / mc.ret_turns as f64 } else { 0.0 };
            let scout_avg_pot = if mc.scout_turns > 0 { mc.scout_pot_sum as f64 / mc.scout_turns as f64 } else { 0.0 };
            let ret_missed = ret_avg_pot - mc.ret_closes as f64;
            let scout_missed = scout_avg_pot - mc.scout_closes as f64;
            total_ret_missed += ret_missed;
            total_scout_missed += scout_missed;
            println!("skip={skip} ret={} rs={rs:.0} ss={ss:.0} margin={m:+.1}% diff={:+.0} | missed: ret={ret_missed:.1} scout={scout_missed:.1} (pot: ret={ret_avg_pot:.1} scout={scout_avg_pot:.1}, closes: ret={} scout={}) ({:.0}s)", if ret_blue { "blue" } else { "red" }, rs - ss, mc.ret_closes, mc.scout_closes, t.elapsed().as_secs_f64());
            sum += m; n += 1;
        }
    }
    println!("AVG margin (ret perspective): {:.1}%", sum / n as f64);
    println!("==> missed-closes/game: ret={:.2} scout={:.2}", total_ret_missed / n as f64, total_scout_missed / n as f64);
}
