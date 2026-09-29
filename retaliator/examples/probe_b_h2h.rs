//! Lane B head-to-head: `eval_phases` (legal-cuts-only vulnerability) vs the
//! shipped `search::best_move`, over the site's forced openings + empty start
//! (same set as `probe_v3all`, 5 openings x 2 colors = 10 games). First a
//! faithfulness check: the term-OFF control must pick exactly what the shipped
//! search picks, position for position — any mismatch means the skeleton copy
//! drifted and the games below measure drift, not the term.
//! Usage: cargo run --release --example probe_b_h2h

#[path = "../src/eval_phases.rs"]
mod eval_phases;
#[path = "support/scoutbase.rs"]
mod scoutbase;
use meridian_engine::{Game, Move, Player, Position};

fn play(mut game: Game, a_blue: bool, a: fn(&Position) -> Option<Move>, b: fn(&Position) -> Option<Move>) -> (f64, f64) {
    while !game.is_over() {
        let a_moves = (game.position().to_move() == Player::Blue) == a_blue;
        game.play(if a_moves { a(game.position()) } else { b(game.position()) }.unwrap()).unwrap();
    }
    let asc = game.position().score(if a_blue { Player::Blue } else { Player::Red }).to_f64();
    let bsc = game.position().score(if a_blue { Player::Red } else { Player::Blue }).to_f64();
    (asc, bsc)
}
fn wra(f: fn(&Position) -> Option<Move>) -> fn(&Position) -> Option<Move> { f }

std::thread_local! {
    static UNBREAK_DOSE: std::cell::RefCell<(f64, f64)> = std::cell::RefCell::new((0.0, 0.0));
    static MISSED_DOSE: std::cell::RefCell<(f64, f64)> = std::cell::RefCell::new((0.0, 0.0));
    static DELAY_DOSE: std::cell::RefCell<f64> = std::cell::RefCell::new(0.0);
}

/// Variant picker for `E_DELAY=w`: the Q5 dose via a thread-local.
fn q5_pick(position: &Position) -> Option<Move> {
    let w = DELAY_DOSE.with(|d| *d.borrow());
    eval_phases::delay_best_move_with_avoid(position, &[], eval_phases::Delay { w })
}

/// Variant picker for `E_MISSED=m[,f]`: the Q1 dose via a thread-local.
fn q1_pick(position: &Position) -> Option<Move> {
    let (m, f) = MISSED_DOSE.with(|d| *d.borrow());
    eval_phases::breakfix_best_move_with_avoid(position, &[], eval_phases::Breakfix { missed: m, floor: f })
}

/// Variant picker for `E_UNBREAK=e,c`: the Q2 dose via a thread-local
/// (fn pointers can't close over the parsed dose). Deterministic: the
/// dose is fixed before the games start, tiebreaks still by move index.
fn unbreak_pick(position: &Position) -> Option<Move> {
    let (e, c) = UNBREAK_DOSE.with(|d| *d.borrow());
    eval_phases::unbreak_best_move_with_avoid(
        position,
        &[],
        eval_phases::Unbreak { extend_w: e, create_w: c },
    )
}

fn main() {
    // Faithfulness: control (term OFF) vs shipped search, position for
    // position, over a scoutbase self-play game from the empty start and one
    // from a forced opening (5589, the worst Blue chair).
    let mut checked = 0usize;
    let mut bad = 0usize;
    for seed_open in [None, Some(5589)] {
        let mut game = Game::new();
        if let Some(id) = seed_open {
            game.play(Move::from_index(id).unwrap()).unwrap();
        }
        while !game.is_over() {
            let shipped = retaliator::search::best_move(game.position());
            let control = eval_phases::baseline_best_move(game.position());
            checked += 1;
            if control != shipped {
                bad += 1;
                println!(
                    "MISMATCH at actions_played={}: control={:?} shipped={:?}",
                    game.position().actions_played(),
                    control.map(|mv| mv.index()),
                    shipped.map(|mv| mv.index())
                );
            }
            game.play(shipped.unwrap()).unwrap();
        }
    }
    println!("faithfulness: {bad} mismatches in {checked} positions (control vs shipped)");
    if bad > 0 {
        println!("COPY DRIFTED — games below would measure drift, not the term. Fix first.");
        return;
    }

    let laneb: fn(&Position) -> Option<Move> = match std::env::var("E_DELAY") {
        Ok(v) => {
            let w: f64 = v.parse().unwrap_or(0.0);
            println!("laneE Q5 config: E_DELAY w={w}");
            DELAY_DOSE.with(|d| *d.borrow_mut() = w);
            q5_pick
        }
        Err(_) => match std::env::var("E_MISSED") {
        Ok(v) => {
            let mut it = v.split(',');
            let m: f64 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let f: f64 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
            println!("laneE Q1 config: E_MISSED missed={m} floor={f}");
            MISSED_DOSE.with(|d| *d.borrow_mut() = (m, f));
            q1_pick
        }
        Err(_) => match std::env::var("E_UNBREAK") {
            Ok(v) => {
                let mut it = v.split(',');
                let e: f64 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let c: f64 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                println!("laneE Q2 config: E_UNBREAK extend={e} create={c}");
                // fn pointer can't close over dose; route through a thread-local.
                UNBREAK_DOSE.with(|d| *d.borrow_mut() = (e, c));
                unbreak_pick
            }
            Err(_) => {
                println!("laneE config: control (doom-OFF best_move)");
                wra(eval_phases::best_move)
            }
        },
        },
    };
    let shipped = wra(retaliator::search::best_move);
    let (mut w, mut n) = (0, 0);
    let (mut wb, mut nb, mut wr, mut nr) = (0, 0, 0, 0);
    let (mut sum_b, mut sum_r) = (0.0, 0.0);
    for open in [None, Some(4864), Some(5589), Some(11723), Some(9199)] {
        for a_blue in [true, false] {
            let mut game = Game::new();
            if let Some(id) = open {
                game.play(Move::from_index(id).unwrap()).unwrap();
            }
            let t = std::time::Instant::now();
            let (asc, bsc) = play(game, a_blue, laneb, shipped);
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
            println!(
                "laneB vs shipped open={open:?} a={} {asc:.0}-{bsc:.0} margin={m:+.1}% {} ({:.0}s)",
                if a_blue { "blue" } else { "red" },
                if asc > bsc { "A WINS" } else { "b wins" },
                t.elapsed().as_secs_f64()
            );
        }
    }
    println!(
        "==> laneB vs shipped: {w}/{n} | as blue {wb}/{nb} avg margin {:+.1}% | as red {wr}/{nr} avg margin {:+.1}%",
        sum_b / nb as f64,
        sum_r / nr as f64
    );
}
