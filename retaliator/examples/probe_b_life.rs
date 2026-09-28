//! Lane E v7 area-lifetime probe: variant (E_UNBREAK dose) vs shipped over
//! the h2h set (5 openings x 2 colors = 10 games). Per side per game:
//! - built: sum of positive own-area increments (new enclosures + extensions)
//! - lost: sum of own-area drops (enemy cuts), breaks: drop-event count
//! - lifetime: sum(own area after each action) / built = mean actions each
//!   built area-unit survives (the Q2 metric: control ~30.7 -> rival ~49.8)
//! - banked-per-built: final score / built (the Q1 metric)
//! Structural read-only accounting over engine state; no eval terms here.
//! Usage: E_UNBREAK=1,1 cargo run --release --example probe_b_life

#[path = "../src/eval_phases.rs"]
mod eval_phases;
use meridian_engine::{Game, Move, Player, Position};

#[derive(Default)]
struct Side {
    area_sum: f64,
    built: f64,
    lost: f64,
    breaks: u32,
}

fn variant_pick(position: &Position, dose: (f64, f64)) -> Option<Move> {
    eval_phases::unbreak_best_move_with_avoid(
        position,
        &[],
        eval_phases::Unbreak { extend_w: dose.0, create_w: dose.1 },
    )
}

fn main() {
    let u = eval_phases::Unbreak::from_env();
    let dose = (u.extend_w, u.create_w);
    println!("life config: E_UNBREAK extend={} create={}", dose.0, dose.1);
    // Aggregate (variant, shipped) across games.
    let (mut va, mut sa) = (Side::default(), Side::default());
    let (mut vb, mut sb) = (0.0, 0.0);
    let (mut w, mut n) = (0, 0);
    for open in [None, Some(4864), Some(5589), Some(11723), Some(9199)] {
        for a_blue in [true, false] {
            let mut game = Game::new();
            if let Some(id) = open {
                game.play(Move::from_index(id).unwrap()).unwrap();
            }
            let (mut cb, mut cr) = (Side::default(), Side::default());
            let (mut pb, mut pr) = (0.0, 0.0);
            while !game.is_over() {
                let a_moves = (game.position().to_move() == Player::Blue) == a_blue;
                let mv = if a_moves {
                    variant_pick(game.position(), dose)
                } else {
                    retaliator::search::best_move(game.position())
                };
                game.play(mv.unwrap()).unwrap();
                let (nb, nr) = (
                    game.position().area(Player::Blue).to_f64(),
                    game.position().area(Player::Red).to_f64(),
                );
                for (s, now, prev) in [(&mut cb, nb, pb), (&mut cr, nr, pr)] {
                    let d = now - prev;
                    if d > 0.0 {
                        s.built += d;
                    } else if d < -0.5 {
                        s.lost -= d;
                        s.breaks += 1;
                    }
                    s.area_sum += now;
                }
                (pb, pr) = (nb, nr);
            }
            let (asc, bsc) = (
                game.position().score(if a_blue { Player::Blue } else { Player::Red }).to_f64(),
                game.position().score(if a_blue { Player::Red } else { Player::Blue }).to_f64(),
            );
            // Attribute blue/red sides to variant(a)/shipped(b).
            let (v, s) = if a_blue { (&cb, &cr) } else { (&cr, &cb) };
            let (vs, ss) = if a_blue { (asc, bsc) } else { (bsc, asc) };
            let vl = v.area_sum / v.built.max(1e-9);
            let sl = s.area_sum / s.built.max(1e-9);
            println!(
                "open={open:?} a={} {asc:.0}-{bsc:.0} {} | variant life={vl:.1} built={:.0} lost={:.0} brk={} bpb={:.2} | shipped life={sl:.1} built={:.0} lost={:.0} brk={} bpb={:.2}",
                if a_blue { "blue" } else { "red" },
                if asc > bsc { "A WINS" } else { "b wins" },
                v.built, v.lost, v.breaks, vs / v.built.max(1e-9),
                s.built, s.lost, s.breaks, ss / s.built.max(1e-9),
            );
            for (agg, bnk, x) in [(&mut va, &mut vb, (v, vs)), (&mut sa, &mut sb, (s, ss))] {
                agg.area_sum += x.0.area_sum;
                agg.built += x.0.built;
                agg.lost += x.0.lost;
                agg.breaks += x.0.breaks;
                *bnk += x.1;
            }
            if asc > bsc {
                w += 1;
            }
            n += 1;
        }
    }
    println!(
        "==> {w}/{n} | variant life={:.1} bpb={:.2} breaks={} | shipped life={:.1} bpb={:.2} breaks={}",
        va.area_sum / va.built.max(1e-9),
        vb / va.built.max(1e-9),
        va.breaks,
        sa.area_sum / sa.built.max(1e-9),
        sb / sa.built.max(1e-9),
        sa.breaks,
    );
}
