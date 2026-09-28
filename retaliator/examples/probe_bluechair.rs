//! Lane C read-only probe (autopsy): Blue chair — v3 as Blue vs v1 with the
//! site's forced openings. Replays games and prints per-action diagnostics:
//! first-close timing/area for both sides, every our-move with the eval before
//! it, every break with the exact action, and analyze() top candidates at big
//! events. Does not change bot behavior.
//!
//! Also runs the opener-response test: forced opening as Blue action 0, then
//! our first action FORCED to the D10-F7 opener move, rest free — does placing
//! the measured opener as the response turn the loss around?

#[path = "support/v1base.rs"]
mod v1base;

use meridian_engine::{Game, Move, MoveKind, Player, Point};
use retaliator::search;

fn side(p: Player) -> &'static str {
    if p == Player::Blue { "B" } else { "R" }
}

fn area_of(game: &Game, p: Player) -> f64 {
    game.position().area(p).to_f64()
}

fn score_of(game: &Game, p: Player) -> f64 {
    game.position().score(p).to_f64()
}

/// Plays v3 (as `ret_blue`) vs v1 and prints the full per-action log.
fn play_logged(mut game: Game, ret_blue: bool, label: &str) {
    let mut n = 0usize;
    let mut first_close = [None::<(usize, f64)>; 2]; // [Blue, Red]: (action, area gained)
    let mut first_gain = [None::<(usize, f64)>; 2]; // any area-gaining event
    let mut closes = [0usize; 2];
    let mut cuts = [0usize; 2];
    let mut analyze_budget = 3usize; // analyze() top-5 printed at most this many big events
    println!("== {label} (v3 as {} vs v1)", if ret_blue { "BLUE" } else { "RED" });
    println!("move kind areas-after | annotations (eval = search::evaluate before our move)");
    while !game.is_over() {
        let pos = game.position().clone();
        let mover = pos.to_move();
        let ours = (mover == Player::Blue) == ret_blue;
        let mv = if ours {
            search::best_move(&pos)
        } else {
            v1base::best_move(&pos)
        }
        .unwrap();
        let b0 = area_of(&game, Player::Blue);
        let r0 = area_of(&game, Player::Red);
        let eval_before = if ours { Some(search::evaluate(&pos)) } else { None };
        let oc = game.play(mv).unwrap();
        n += 1;
        let (b1, r1) = (area_of(&game, Player::Blue), area_of(&game, Player::Red));
        let me = if mover == Player::Blue { b1 - b0 } else { r1 - r0 };
        let mi = if mover == Player::Blue { b0 - b1 } else { r0 - r1 };
        let who = side(if (mover == Player::Blue) == ret_blue { Player::Blue } else { Player::Red });
        let close_gain = if oc.kind == MoveKind::Connect { Some(me) } else { None };
        // Any area gain (self-crossing extends close loops too), for first-gain timing.
        if me > 0.5 && oc.kind != MoveKind::Capture {
            let idx = usize::from(mover == Player::Blue);
            if first_gain[idx].is_none() {
                first_gain[idx] = Some((n, me));
            }
        }
        if let Some(g) = close_gain {
            let idx = usize::from(mover == Player::Blue);
            closes[idx] += 1;
            if first_close[idx].is_none() {
                first_close[idx] = Some((n, g));
            }
        }
        if oc.broken.is_some() {
            let idx = usize::from(mover == Player::Blue);
            cuts[idx] += 1;
        }
        let mut ann = String::new();
        if let Some(ev) = eval_before {
            ann.push_str(&format!(" eval={ev:+.0}"));
        }
        if oc.kind == MoveKind::Connect {
            ann.push_str(&format!(" CLOSE(+{me:.1})"));
        }
        if let Some(broken) = oc.broken {
            ann.push_str(&format!(
                " BREAK {}->{} victim{}-{mi:+.1}",
                meridian_engine::notation::move_text(broken.origin(), broken.far()),
                meridian_engine::notation::move_text(oc.placed.origin(), oc.placed.far()),
                who
            ));
        }
        if me.abs() > 3.0 && oc.kind != MoveKind::Connect && oc.broken.is_none() {
            ann.push_str(&format!(" BIGSWING{me:+.1}"));
        }
        let interesting = ours
            || close_gain.is_some()
            || oc.broken.is_some()
            || me.abs() > 3.0
            || n % 10 == 0;
        let txt = meridian_engine::notation::move_text(mv.source, mv.target().unwrap());
        if interesting {
            println!(
                "[{n}] {} {} {who} {txt} {b1:.1}/{r1:.1}{ann}",
                mv.index(),
                match oc.kind {
                    MoveKind::Extend => "ext",
                    MoveKind::Connect => "con",
                    MoveKind::Capture => "cap",
                }
            );
        }
        // analyze() top-5 at the first close and at any big enemy pop (capped).
        if analyze_budget > 0
            && ((close_gain.is_some() && me.abs() > 2.0) || mi.abs() > 5.0)
        {
            analyze_budget -= 1;
            let a = search::analyze(&pos, 2048);
            let tops: Vec<String> = a
                .candidates
                .iter()
                .take(5)
                .map(|c| format!("{}({:+.0})", c.mv.index(), c.evaluation))
                .collect();
            println!(
                "    analyze@{n} eval={:+.0} depth={} nodes={} top: {}",
                a.evaluation,
                a.depth,
                a.nodes,
                tops.join(" ")
            );
        }
        if n % 10 == 0 || game.is_over() {
            println!(
                "= after {n}: b={b1:.1}/{:.0} r={r1:.1}/{:.0}",
                score_of(&game, Player::Blue),
                score_of(&game, Player::Red)
            );
        }
    }
    let (bs, rs) = (score_of(&game, Player::Blue), score_of(&game, Player::Red));
    let (ba, ra) = (area_of(&game, Player::Blue), area_of(&game, Player::Red));
    let fc = |x: Option<(usize, f64)>| match x {
        Some((t, g)) => format!("act {t} (+{g:.1})"),
        None => "never".into(),
    };
    println!(
        "==> FINAL {bs:.0}-{rs:.0} {} | areas B {ba:.1} R {ra:.1} | B: 1st gain {} 1st close {} / {} closes / {} cuts | R: 1st gain {} 1st close {} / {} closes / {} cuts",
        if bs > rs { "BLUE WINS" } else { "RED WINS" },
        fc(first_gain[1]),
        fc(first_close[1]),
        closes[1],
        cuts[1],
        fc(first_gain[0]),
        fc(first_close[0]),
        closes[0],
        cuts[0]
    );
}

fn main() {
    // The site's forced openings. None = empty board (v3's D10-F7 opener fires).
    for open in [None, Some(4864), Some(5589), Some(11723), Some(9199)] {
        let mut game = Game::new();
        if let Some(id) = open {
            let mv = Move::from_index(id).unwrap();
            println!(
                "-- forced opening {} = {}",
                id,
                meridian_engine::notation::move_text(mv.source, mv.target().unwrap())
            );
            game.play(mv).unwrap();
        } else {
            println!("-- no forced opening (D10-F7 opener fires at action 0)");
        }
        play_logged(game, true, &format!("open={open:?}"));
    }
    // Opener-response test: forced opening as Blue action 0, Red's first turn
    // free (v1), then D10-F7 FORCED as our first action of our first turn
    // (the earliest it can legally be played — after Red's 2-action turn),
    // rest free search for both sides. V1 of this test was a NO-OP: it
    // checked legality on Red's turn, where D10-F7 is always illegal.
    println!("== opener-response test: Red turn 1 free, then forced D10-F7 as our action 3");
    for open in [None, Some(4864), Some(5589), Some(11723), Some(9199)] {
        let mut game = Game::new();
        if let Some(id) = open {
            game.play(Move::from_index(id).unwrap()).unwrap();
        }
        // Red's full first turn (and, in the None game, Blue's solo — v1's
        // search picks D10-A7 there), free: play v1 until it's OUR turn.
        while game.position().to_move() == Player::Red {
            let mv = v1base::best_move(game.position()).unwrap();
            println!(
                "  red@{} {} {}",
                game.position().actions_played() + 1,
                mv.index(),
                meridian_engine::notation::move_text(mv.source, mv.target().unwrap())
            );
            game.play(mv).unwrap();
        }
        // Force the opener as our first action of our first turn.
        let opener = Move::between(Point::new(-6, 0).unwrap(), Point::new(-4, -3).unwrap()).unwrap();
        let forced = if game.position().check_move(opener).is_ok() {
            game.play(opener).unwrap();
            true
        } else {
            false
        };
        // rest free
        while !game.is_over() {
            let mv = if (game.position().to_move() == Player::Blue) {
                search::best_move(game.position())
            } else {
                v1base::best_move(game.position())
            }
            .unwrap();
            game.play(mv).unwrap();
        }
        let (bs, rs) = (score_of(&game, Player::Blue), score_of(&game, Player::Red));
        println!(
            "open={open:?} +D10-F7@our-act-3 ({}): {bs:.0}-{rs:.0} {}",
            if forced { "forced" } else { "ILLEGAL, free instead" },
            if bs > rs { "BLUE WINS" } else { "RED WINS" }
        );
    }
}
