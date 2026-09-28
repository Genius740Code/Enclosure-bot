//! Lane B2 close-weight diagnostic (gain-scaled close priority, H-B-VLAD1).
//! (1) Pick flips: `eval_phases::best_move` (the config under test) vs
//! `eval_phases::ablation_best_move` (doom OFF + close OFF — the same doom
//! setting), position for position over two control trajectories (empty start
//! + the 5589 Blue chair, scoutbase self-play). Isolates the close term alone.
//! (2) Close sizes: the 8 league games (skip 0/10/20/30 x 2 colors) with the
//! gain of every close recorded per side — the H-B-VLAD1 disease is our
//! closes banking ~0.2-0.9 while the greedy close banks ~2-4.
//! Usage: cargo run --release --example probe_b_close

#[path = "../src/eval_phases.rs"]
mod eval_phases;
#[path = "support/scoutbase.rs"]
mod scoutbase;
use meridian_engine::{Game, Move, Player, Position};

/// Gained area of `mv` for the player to move in `position`.
fn gain_of(position: &Position, mv: Move) -> f64 {
    let mover = position.to_move();
    let mut after = position.clone();
    after.apply_unchecked(mv);
    after.area(mover).to_f64() - position.area(mover).to_f64()
}

fn main() {
    // (1) Pick flips vs the ablation control, on the control trajectory.
    let mut checked = 0usize;
    let mut flips = 0usize;
    for seed_open in [None, Some(5589)] {
        let mut game = Game::new();
        if let Some(id) = seed_open {
            game.play(Move::from_index(id).unwrap()).unwrap();
        }
        while !game.is_over() {
            let control = eval_phases::ablation_best_move(game.position());
            let tested = eval_phases::best_move(game.position());
            checked += 1;
            if control != tested {
                flips += 1;
                if flips <= 12 {
                    let ctrl = control.map(|mv| mv.index());
                    let tst = tested.map(|mv| mv.index());
                    let cg = control.map(|mv| gain_of(game.position(), mv));
                    let tg = tested.map(|mv| gain_of(game.position(), mv));
                    println!(
                        "flip at actions_played={}: control={ctrl:?} gain={cg:.2?} | tested={tst:?} gain={tg:.2?}",
                        game.position().actions_played()
                    );
                }
            }
            game.play(control.unwrap()).unwrap();
        }
    }
    println!("pick flips vs ablation control (doom OFF + close OFF): {flips} in {checked} positions");

    // (2) Close sizes in the 8 league games (skip 0/10/20/30 x 2 colors).
    let mut sum = 0.0;
    let mut n = 0;
    for skip in [0usize, 10, 20, 30] {
        for ret_blue in [true, false] {
            let mut game = Game::new();
            for _ in 0..skip {
                if game.is_over() { break; }
                let mv = retaliator::search::best_move(game.position()).unwrap();
                game.play(mv).unwrap();
            }
            if game.is_over() { continue; }
            let mut stats = [(0u32, 0.0f64, f64::INFINITY, f64::NEG_INFINITY, 0u32, 0u32); 2];
            while !game.is_over() {
                let to_move = game.position().to_move();
                let ret_moves = (to_move == Player::Blue) == ret_blue;
                let mv = if ret_moves {
                    eval_phases::best_move(game.position())
                } else {
                    scoutbase::best_move(game.position())
                }
                .unwrap();
                let side = if ret_moves { 0 } else { 1 };
                let g = gain_of(game.position(), mv);
                if g > 0.0 {
                    let s = &mut stats[side];
                    s.0 += 1;
                    s.1 += g;
                    s.2 = s.2.min(g);
                    s.3 = s.3.max(g);
                    if g < 1.0 { s.4 += 1; }
                    if g >= 2.0 { s.5 += 1; }
                }
                game.play(mv).unwrap();
            }
            let rs = game.position().score(if ret_blue { Player::Blue } else { Player::Red }).to_f64();
            let ss = game.position().score(if ret_blue { Player::Red } else { Player::Blue }).to_f64();
            let m = (rs - ss) / rs * 100.0;
            let (rc, rg, rmin, rmax, rtiny, rbig) = (stats[0].0, stats[0].1, stats[0].2, stats[0].3, stats[0].4, stats[0].5);
            let (sc, sg, smin, smax, stiny, sbig) = (stats[1].0, stats[1].1, stats[1].2, stats[1].3, stats[1].4, stats[1].5);
            println!(
                "skip={skip} ret={} rs={rs:.0} ss={ss:.0} margin={m:+.1}% | ret closes {rc} avg {}/c [{rtiny} tiny, {rbig} big] min {rmin:.2} max {rmax:.2} | scout closes {sc} avg {}/c [{stiny} tiny, {sbig} big] min {smin:.2} max {smax:.2}",
                if ret_blue { "blue" } else { "red" },
                if rc > 0 { rg / rc as f64 } else { 0.0 },
                if sc > 0 { sg / sc as f64 } else { 0.0 },
            );
            sum += m;
            n += 1;
        }
    }
    println!("AVG margin (ret perspective): {:.1}%", sum / n as f64);
}
