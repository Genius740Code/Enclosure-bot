//! Lane B v1 head-to-head: `eval_phases` vs the frozen v1 baseline
//! (`support/v1base.rs`), over the site's forced openings + empty start
//! (same 10-game set as `probe_v3all`). v1 is the live site bot and the
//! Blue-chair record vs v1 is problem #1, so this is the gate that counts.
//!
//! B-3 farm-cycle routing (2026-09-28): OUR side's anti-rebuild avoid set
//! is fed live from the game (the `lib.rs` `replay()` wiring, memory
//! generalized). `B3_MEM` = the cut-memory window in actions — 0/absent =
//! no avoid = the published doom-OFF control (byte-identical); 6 = the
//! shipped site wiring; the B-3 sweep runs 12/20/40. `B3_SCALE` = 0 flat
//! routing only, 1 = + gain-scaled penalty keeping the counter-cut
//! exemption, 2 = + gain-scaled penalty on all first actions on cut
//! ground. Usage: `B3_MEM=20 B3_SCALE=2 cargo run --release --example probe_b_v1`

#[path = "../src/eval_phases.rs"]
mod eval_phases;
#[path = "support/v1base.rs"]
mod v1base;
use meridian_engine::{Edge, Game, Move, Player, Point};

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
    /// Total one-move potential at start of each turn for laneB (our bot)
    laneb_pot_sum: u32,
    laneb_turns: u32,
    /// Actual area-gaining Connect moves by laneB
    laneb_closes: u32,
    /// Total one-move potential at start of each turn for v1 (opponent)
    v1_pot_sum: u32,
    v1_turns: u32,
    /// Actual area-gaining Connect moves by v1
    v1_closes: u32,
}

fn play(
    mut game: Game,
    a_blue: bool,
    mem: usize,
    scale: u8,
    cuts: &mut Vec<(u16, Player, Edge)>,
) -> (f64, f64, MissedCloses) {
    let mut mc = MissedCloses::default();
    while !game.is_over() {
        let to_move = game.position().to_move();
        let played = u16::from(game.position().actions_played());
        let a_moves = (to_move == Player::Blue) == a_blue;
        
        // Track one-move potential at start of turn
        let pot_me = eval_phases::one_move_potential(game.position(), to_move);
        let pot_opp = eval_phases::one_move_potential(game.position(), to_move.opponent());
        if a_moves {
            mc.laneb_pot_sum += pot_me;
            mc.laneb_turns += 1;
        } else {
            mc.v1_pot_sum += pot_me;
            mc.v1_turns += 1;
        }
        
        let mv = if a_moves {
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
            v1base::best_move(game.position())
        };
        let mv = mv.unwrap();
        let area_before = game.position().area(to_move).to_f64();
        let oc = game.play(mv).unwrap();
        let area_after = game.position().area(to_move).to_f64();
        let gain = area_after - area_before;
        
        // Count area-gaining Connect moves as "closes"
        if oc.kind == meridian_engine::MoveKind::Connect && gain > 0.0 {
            if a_moves {
                mc.laneb_closes += 1;
            } else {
                mc.v1_closes += 1;
            }
        }
        
        if let Some(cut) = oc.broken {
            cuts.push((played, to_move.opponent(), cut));
        }
    }
    let asc = game.position().score(if a_blue { Player::Blue } else { Player::Red }).to_f64();
    let bsc = game.position().score(if a_blue { Player::Red } else { Player::Blue }).to_f64();
    (asc, bsc, mc)
}

fn main() {
    let mem: usize = std::env::var("B3_MEM").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let scale: u8 = std::env::var("B3_SCALE").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    println!("B3 config: mem={mem} scale={scale}");
    let (mut w, mut n) = (0, 0);
    let (mut wb, mut nb, mut wr, mut nr) = (0, 0, 0, 0);
    let (mut sum_b, mut sum_r) = (0.0, 0.0);
    let (mut total_laneb_missed, mut total_v1_missed) = (0.0, 0.0);
    for open in [None, Some(4864), Some(5589), Some(11723), Some(9199)] {
        for a_blue in [true, false] {
            let mut game = Game::new();
            let mut cuts: Vec<(u16, Player, Edge)> = Vec::new();
            if let Some(id) = open {
                let oc = game.play(Move::from_index(id).unwrap()).unwrap();
                if let Some(cut) = oc.broken {
                    cuts.push((0, Player::Blue.opponent(), cut));
                }
            }
            let t = std::time::Instant::now();
            let (asc, bsc, mc) = play(game, a_blue, mem, scale, &mut cuts);
            n += 1;
            let m = (asc - bsc) / asc * 100.0;
            if a_blue {
                nb += 1;
                sum_b += m;
                if asc > bsc { w += 1; wb += 1; }
            } else {
                nr += 1;
                sum_r += m;
                if asc > bsc { w += 1; wr += 1; }
            }
            let laneb_avg_pot = if mc.laneb_turns > 0 { mc.laneb_pot_sum as f64 / mc.laneb_turns as f64 } else { 0.0 };
            let v1_avg_pot = if mc.v1_turns > 0 { mc.v1_pot_sum as f64 / mc.v1_turns as f64 } else { 0.0 };
            let laneb_missed = laneb_avg_pot - mc.laneb_closes as f64;
            let v1_missed = v1_avg_pot - mc.v1_closes as f64;
            total_laneb_missed += laneb_missed;
            total_v1_missed += v1_missed;
            println!(
                "laneB vs v1 open={open:?} a={} {asc:.0}-{bsc:.0} margin={m:+.1}% diff={:+.0} {} | missed: laneB={laneb_missed:.1} v1={v1_missed:.1} (pot: laneB={laneb_avg_pot:.1} v1={v1_avg_pot:.1}, closes: laneB={} v1={}) ({:.0}s)",
                if a_blue { "blue" } else { "red" },
                asc - bsc,
                if asc > bsc { "A WINS" } else { "b wins" },
                mc.laneb_closes,
                mc.v1_closes,
                t.elapsed().as_secs_f64()
            );
        }
    }
    println!(
        "==> laneB vs v1: {w}/{n} | as blue {wb}/{nb} avg margin {:+.1}% | as red {wr}/{nr} avg margin {:+.1}%",
        sum_b / nb as f64,
        sum_r / nr as f64
    );
    println!(
        "==> missed-closes/game: laneB={:.2} v1={:.2}",
        total_laneb_missed / n as f64,
        total_v1_missed / n as f64
    );
}
