//! Lane E v7 Q12 thick-line probe: variant (E_THICK dose) vs shipped over
//! the h2h set (5 openings x 2 colors = 10 games). Per side per game:
//! - thick_share: mean over actions of (own nodes with degree >= 2) / (own
//!   nodes) — the "thick-line lifetime" proxy: fraction of the game our
//!   lines spend shared-node thick (structural, engine state only).
//! - completions: own moves after which the own shared-node count ROSE —
//!   foe near-thick pairs closed into thickness (the prevent gate's denial
//!   target, counted for both sides so dose-vs-control compares foe
//!   completions too).
//! Gate (dose-vs-control): variant thick_share UP and foe (shipped-side)
//! completions DOWN. Structural read-only accounting; no eval terms here.
//! Usage: E_THICK=1,0 cargo run --release --example probe_b_thick

#[path = "../src/eval_phases.rs"]
mod eval_phases;
use meridian_engine::{Game, Move, Player, Position};

fn shared_count(position: &Position, player: Player) -> usize {
    position
        .nodes(player)
        .iter()
        .filter(|n| {
            position.edges(player).iter().filter(|e| e.has_endpoint(*n)).count() >= 2
        })
        .count()
}

fn share(position: &Position, player: Player) -> f64 {
    let nodes = position.nodes(player).len();
    if nodes == 0 {
        0.0
    } else {
        shared_count(position, player) as f64 / nodes as f64
    }
}

fn variant_pick(position: &Position) -> Option<Move> {
    let t = eval_phases::Thick::from_env();
    if t.on() {
        eval_phases::thick_best_move_with_avoid(position, &[], t)
    } else {
        eval_phases::best_move(position)
    }
}

fn main() {
    let t = eval_phases::Thick::from_env();
    println!("thick config: E_THICK r={} p={}", t.r, t.p);
    // Aggregates: (share_sum, actions, completions) per side class.
    let (mut v_share, mut v_act, mut v_comp) = (0.0, 0u32, 0u32);
    let (mut s_share, mut s_act, mut s_comp) = (0.0, 0u32, 0u32);
    let (mut w, mut n) = (0, 0);
    for open in [None, Some(4864), Some(5589), Some(11723), Some(9199)] {
        for a_blue in [true, false] {
            let mut game = Game::new();
            if let Some(id) = open {
                game.play(Move::from_index(id).unwrap()).unwrap();
            }
            let (mut vsh, mut vah, mut vco) = (0.0, 0u32, 0u32);
            let (mut ssh, mut sah, mut sco) = (0.0, 0u32, 0u32);
            while !game.is_over() {
                let a_moves = (game.position().to_move() == Player::Blue) == a_blue;
                let mover = game.position().to_move();
                let before = shared_count(game.position(), mover);
                let mv = if a_moves {
                    variant_pick(game.position())
                } else {
                    retaliator::search::best_move(game.position())
                };
                game.play(mv.unwrap()).unwrap();
                if shared_count(game.position(), mover) > before {
                    if a_moves { vco += 1; } else { sco += 1; }
                }
                let (vb, sb) = (share(game.position(), Player::Blue), share(game.position(), Player::Red));
                // Attribute blue/red shares to variant(a)/shipped(b).
                let (vs, ss) = if a_blue { (vb, sb) } else { (sb, vb) };
                vsh += vs; vah += 1;
                ssh += ss; sah += 1;
            }
            let (asc, bsc) = (
                game.position().score(if a_blue { Player::Blue } else { Player::Red }).to_f64(),
                game.position().score(if a_blue { Player::Red } else { Player::Blue }).to_f64(),
            );
            println!(
                "open={open:?} a={} {asc:.0}-{bsc:.0} {} | variant share={:.3} comp={vco} | shipped share={:.3} comp={sco}",
                if a_blue { "blue" } else { "red" },
                if asc > bsc { "A WINS" } else { "b wins" },
                vsh / vah as f64,
                ssh / sah as f64,
            );
            v_share += vsh; v_act += vah; v_comp += vco;
            s_share += ssh; s_act += sah; s_comp += sco;
            if asc > bsc {
                w += 1;
            }
            n += 1;
        }
    }
    println!(
        "==> {w}/{n} | variant share={:.3} comp={v_comp} | shipped share={:.3} comp={s_comp}",
        v_share / v_act as f64,
        s_share / s_act as f64,
    );
}
