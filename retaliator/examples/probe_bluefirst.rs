//! Lane H probe -- force Blue's first action to candidate moves, then play out full games with
//! the shipped `search::best_move` for both sides. PROBE-ONLY: no `retaliator/src/*` changes.
//!
//! Why (c-blunder-catalog §0 + c-site-losses-2 §4.3 H6): the site chooses Blue's engine action 1
//! in rated play (our observed first moves 7030/4503/5228 are never D10-F7), so `blue_opener`
//! never fires on site and its measured +20-41% advantage is unexercised. The prior
//! opener-response experiment (`probe_bluechair`) was a NO-OP: it checked D10-F7 on Red's turn,
//! so the move was always illegal. This is the fixed version (catalog §0's one-line fix):
//!
//! * forced opening: the site's choice is played as engine action 1, Red's reply via the
//!   opponent, then the candidate is forced as Blue's first own move (engine action 4, where
//!   `to_move() == Blue`);
//! * empty start: the candidate replaces `blue_opener` outright (engine action 1).
//!
//! Numbering: engine actions 1..120; B[1] solo, R[2,3], B[4,5]... (`mover_after`). The eval
//! trace is the shipped `search::evaluate` (the V5-1 brain), Blue-perspective. `flip` = last OUR
//! turn-end engine action with eval >= 0 (the c2 instrument; 119 is garbage-time).
//!
//! Matchups: search-vs-search (v3 vs v3) and search-vs-v1, our side Blue for the candidate rows
//! (the candidates are Blue first actions) plus one no-force baseline row per color per opening.
//! Optional CLI arg `v3` / `v1` runs only that matchup (for parallel runs).

#[path = "support/v1base.rs"]
mod v1base;
use meridian_engine::{Game, Move, MoveKind, Player, Point, Position};

/// Record the eval trace for engine actions up to this: covers the act-5..10 window the task
/// asks about, plus our second turn-end (9) and Red's (11).
const TRACE_TO: u8 = 12;

fn pt(x: i8, y: i8) -> Point {
    Point::new(x, y).expect("on the board")
}

fn mv(a: (i8, i8), b: (i8, i8)) -> Move {
    Move::between(pt(a.0, a.1), pt(b.0, b.1)).expect("1-3 king steps")
}

struct Row {
    ours: f64,
    theirs: f64,
    our_area: f64,
    their_area: f64,
    our_first_close: Option<(u8, f64)>,
    evals: Vec<(u8, f64)>,
    flip: Option<u8>,
}

/// Blue's first own action: engine action 1 on the empty start (where `blue_opener` would fire),
/// engine action 4 on a forced opening (after the site's action 1 and Red's [2,3]).
fn first_choice(pos: &Position, had_opening: bool) -> bool {
    pos.actions_played() == if had_opening { 3 } else { 0 }
}

fn play_game(
    opening: Option<usize>,
    force: Option<Move>,
    us_blue: bool,
    us: fn(&Position) -> Option<Move>,
    them: fn(&Position) -> Option<Move>,
) -> Row {
    let mut game = Game::new();
    if let Some(id) = opening {
        game.play(Move::from_index(id).expect("valid opening id")).expect("legal opening");
    }
    let had_opening = opening.is_some();
    let our = if us_blue { Player::Blue } else { Player::Red };
    let (mut evals, mut flip, mut first_close) = (Vec::new(), None, None);
    while !game.is_over() {
        let pos = game.position();
        let mover = pos.to_move();
        let our_turn = (mover == Player::Blue) == us_blue;
        let mv = if our_turn {
            match force {
                // The catalog §0 fix: inject only when to_move() == Blue, and only at Blue's
                // first own action. Legality-checked first (defensive, per that lesson).
                Some(f) if mover == Player::Blue
                    && first_choice(pos, had_opening)
                    && pos.check_move(f).is_ok() =>
                {
                    Some(f)
                }
                _ => us(pos),
            }
        } else {
            them(pos)
        }
        .expect("both searches return a move");
        let area_before = pos.area(our).to_f64();
        let kind = pos.check_move(mv).ok();
        game.play(mv).expect("legal move");
        let after = game.position();
        let ap = after.actions_played();
        let e = retaliator::search::evaluate(after);
        if ap <= TRACE_TO {
            evals.push((ap, e));
        }
        // flip = last OUR turn-end with eval >= 0 (our 2-action turn ends when the mover flips).
        if our_turn && after.to_move() != mover && e >= 0.0 {
            flip = Some(ap);
        }
        if our_turn && kind == Some(MoveKind::Connect) && first_close.is_none() {
            first_close = Some((ap, after.area(our).to_f64() - area_before));
        }
    }
    Row {
        ours: game.position().score(our).to_f64(),
        theirs: game.position().score(our.opponent()).to_f64(),
        our_area: game.position().area(our).to_f64(),
        their_area: game.position().area(our.opponent()).to_f64(),
        our_first_close: first_close,
        evals,
        flip,
    }
}

fn report(open: &str, cand: &str, vs: &str, side: &str, row: &Row, summary: &mut Vec<(String, String, String, f64, bool)>) {
    let margin = (row.ours - row.theirs) / row.ours.max(1.0) * 100.0;
    let evals = row
        .evals
        .iter()
        .filter(|(a, _)| *a >= 5)
        .map(|(a, e)| format!("{a}={e:+.0}"))
        .collect::<Vec<_>>()
        .join(" ");
    let fc = row
        .our_first_close
        .map_or_else(|| "-".to_string(), |(a, g)| format!("act{a}({g:+.1})"));
    println!(
        "OPEN={} CAND={} VS={} SIDE={} FINAL={:.0}-{:.0} MARGIN={:+.1}% AREA={:.1}/{:.1} 1stCls={} FLIP={} E[5-12] {}",
        open, cand, vs, side, row.ours, row.theirs, margin, row.our_area, row.their_area, fc,
        row.flip.map_or_else(|| "-".to_string(), |a| a.to_string()),
        evals
    );
    summary.push((open.to_string(), cand.to_string(), vs.to_string(), margin, row.ours > row.theirs));
}

fn main() {
    let v3: fn(&Position) -> Option<Move> = retaliator::search::best_move;
    let v1: fn(&Position) -> Option<Move> = v1base::best_move;

    // Candidate Blue first actions, with site ids printed for verification:
    // D10-F7 = the shipped `blue_opener` (catalog §0: never fires on site);
    // D10-A7 = the walk-backward the search picks on direction-index ties (search.rs);
    // D10-G7/G13/G10 = longest-reach moves from the center-facing endpoint toward the middle
    //   (H6's area-first book: fast openers reach 19-33 area by act 11 while we sit at 9.0);
    // A10-C9/C8 = the site's observed rated action-1s (catalog §0), short develops from the back.
    let candidates: Vec<(&str, Move)> = vec![
        ("D10-F7", mv((-6, 0), (-4, -3))),
        ("D10-A7", mv((-6, 0), (-9, -3))),
        ("D10-G7", mv((-6, 0), (-3, -3))),
        ("D10-G13", mv((-6, 0), (-3, 3))),
        ("D10-G10", mv((-6, 0), (-3, 0))),
        ("A10-C9", mv((-9, 0), (-7, -1))),
        ("A10-C8", mv((-9, 0), (-7, -2))),
    ];
    for (name, c) in &candidates {
        println!("CAND {} idx={} {:?}", name, c.index(), c);
    }

    // The 5 openings of probe_v3all (none = empty start, `blue_opener`'s own ground).
    let openings: Vec<(&str, Option<usize>)> = vec![
        ("none", None),
        ("4864", Some(4864)),
        ("5589", Some(5589)),
        ("11723", Some(11723)),
        ("9199", Some(9199)),
    ];

    let filter = std::env::args().nth(1);
    let mut summary: Vec<(String, String, String, f64, bool)> = Vec::new();
    for (vs_name, them) in [("v3", v3), ("v1", v1)] {
        if let Some(f) = &filter {
            if f != vs_name {
                continue;
            }
        }
        for (oname, open) in &openings {
            // No-force baselines, both colors (must reproduce probe_v3all byte-for-byte).
            let b = play_game(*open, None, true, v3, them);
            report(oname, "base", vs_name, "blue", &b, &mut summary);
            let r = play_game(*open, None, false, v3, them);
            report(oname, "base", vs_name, "red", &r, &mut summary);
            // Candidate rows: force Blue's first action, our side Blue.
            for (cname, c) in &candidates {
                let b = play_game(*open, Some(*c), true, v3, them);
                report(oname, cname, vs_name, "blue", &b, &mut summary);
            }
        }
    }

    // Ranked action-1 tables: per opening and matchup, best margin first (our side Blue only;
    // the base row is the no-force baseline for reference).
    println!("== RANKED ACTION-1 TABLES (side=blue, base = no-force baseline) ==");
    for vs_name in ["v3", "v1"] {
        if let Some(f) = &filter {
            if f != vs_name {
                continue;
            }
        }
        for (oname, _) in &openings {
            let mut rows: Vec<(f64, bool, &String)> = summary
                .iter()
                .filter(|(o, c, v, _, _)| o == oname && v == vs_name && c != "base")
                .map(|(_, c, _, m, w)| (*m, *w, c))
                .collect();
            rows.sort_by(|a, b| b.0.total_cmp(&a.0));
            let list = rows
                .iter()
                .map(|(m, w, c)| format!("{c} {:+.1}%{}", m, if *w { " W" } else { " L" }))
                .collect::<Vec<_>>()
                .join(" | ");
            let base = summary
                .iter()
                .find(|(o, c, v, _, _)| o == oname && v == vs_name && c == "base")
                .map(|(_, _, _, m, w)| format!("{:+.1}%{}", m, if *w { " W" } else { " L" }))
                .unwrap_or_default();
            println!("VS={vs_name} OPEN={oname}: {list} || base {base}");
        }
    }

    // Per-candidate totals across the 5 openings (n=10 per candidate per matchup).
    println!("== PER-CANDIDATE TOTALS (5 openings, side=blue) ==");
    for vs_name in ["v3", "v1"] {
        if let Some(f) = &filter {
            if f != vs_name {
                continue;
            }
        }
        for (cname, _) in &candidates {
            let rows: Vec<&(String, String, String, f64, bool)> = summary
                .iter()
                .filter(|(o, c, v, _, _)| c == cname && v == vs_name)
                .collect();
            let n = rows.len();
            let wins = rows.iter().filter(|(_, _, _, _, w)| *w).count();
            let mean = rows.iter().map(|(_, _, _, m, _)| *m).sum::<f64>() / n.max(1) as f64;
            let worst = rows.iter().map(|(_, _, _, m, _)| *m).fold(f64::INFINITY, f64::min);
            println!("VS={vs_name} CAND={cname}: {wins}/{n} mean={mean:+.1}% worst={worst:+.1}%");
        }
    }
}
