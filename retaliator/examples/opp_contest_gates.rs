//! Lane O gate run: contest-gen as a probe-side dose through main-session gates.
//! Dose D1 (BLUE-ONLY, BONUS=0): when mover is Blue, generate the explicit
//! contest set (CUT_SITE + OCCUPY within 2 of foe fresh/shielded wall) and
//! force-pick on full-horizon merit vs the baseline pick; when mover is Red,
//! play unmodified `retaliator::search::best_move` (dose == control).
//! Gates: H = h2h vs v1base (5 openings x 2 colors = 10, pass >=6/10),
//! L = scoutbase league (skip 0/10/20/30 x 2 colors = 8, pass AVG > control,
//! no row < -300), G = greedy gauge (6 games, pass 6/6). Control arms (plain
//! search::best_move) run in the same binary for apples-to-apples numbers.

#[path = "support/v1base.rs"]
mod v1base;
#[path = "support/scoutbase.rs"]
mod scoutbase;

use meridian_engine::{Game, Move, Player, Point, Position};

const SITE_DIST: i8 = 2;

fn near(a: Point, b: Point, dist: i8) -> bool {
    (a.x() - b.x()).abs() <= dist && (a.y() - b.y()).abs() <= dist
}

fn site_points(pos: &Position) -> Vec<Point> {
    let mut pts = Vec::new();
    for edge in pos.shielded_edges().chain(pos.fresh_edges()) {
        pts.push(edge.origin());
        pts.push(edge.far());
    }
    pts
}

fn full_value(before: &Position, after: &Position, mover: Player) -> f64 {
    let ev = f64::from(after.scoring_events_left());
    let lead = |p: &Position| {
        (p.score(Player::Blue).to_f64() - p.score(Player::Red).to_f64())
            + (p.area(Player::Blue).to_f64() - p.area(Player::Red).to_f64()) * ev
    };
    let s = if mover == Player::Blue { 1.0 } else { -1.0 };
    s * (lead(after) - lead(before))
}

/// Contest-augmented pick (BONUS=0 full-horizon merit). Caller guarantees
/// this is only used for Blue-to-move (the dose); Red plays baseline.
fn contest_best_move(pos: &Position) -> Option<Move> {
    let mover = pos.to_move();
    let baseline = retaliator::search::best_move(pos);
    let mut base_val = f64::NEG_INFINITY;
    if let Some(mv) = baseline {
        let mut after = pos.clone();
        after.apply_unchecked(mv);
        base_val = full_value(pos, &after, mover);
    }
    let sites = site_points(pos);
    let mut best: Option<(Move, f64)> = None;
    for mv in pos.legal_moves().iter() {
        let target = match mv.target() {
            Some(t) => t,
            None => continue,
        };
        if !sites.iter().any(|&s| near(s, target, SITE_DIST)) {
            continue;
        }
        let mut after = pos.clone();
        after.apply_unchecked(mv);
        let v = full_value(pos, &after, mover);
        if v >= base_val && best.map_or(true, |(_, bv)| v > bv) {
            best = Some((mv, v));
        }
    }
    match best {
        Some((mv, _)) => Some(mv),
        None => baseline,
    }
}

/// Dose D1: contest logic only as Blue; Red (and everything else) = baseline.
fn dose(pos: &Position) -> Option<Move> {
    if pos.to_move() == Player::Blue {
        contest_best_move(pos)
    } else {
        retaliator::search::best_move(pos)
    }
}

fn base(pos: &Position) -> Option<Move> {
    retaliator::search::best_move(pos)
}

fn play(mut game: Game, a_blue: bool, a: fn(&Position) -> Option<Move>, b: fn(&Position) -> Option<Move>) -> (f64, f64) {
    while !game.is_over() {
        let a_moves = (game.position().to_move() == Player::Blue) == a_blue;
        game.play(if a_moves { a(game.position()) } else { b(game.position()) }.unwrap()).unwrap();
    }
    let asc = game.position().score(if a_blue { Player::Blue } else { Player::Red }).to_f64();
    let bsc = game.position().score(if a_blue { Player::Red } else { Player::Blue }).to_f64();
    (asc, bsc)
}

fn gate_h2h(aname: &str, a: fn(&Position) -> Option<Move>) {
    let v1 = v1base::best_move as fn(&Position) -> Option<Move>;
    let (mut w, mut n) = (0, 0);
    let (mut wb, mut wr) = (0, 0);
    for open in [None, Some(4864), Some(5589), Some(11723), Some(9199)] {
        for a_blue in [true, false] {
            let mut game = Game::new();
            if let Some(id) = open {
                game.play(Move::from_index(id).unwrap()).unwrap();
            }
            let (asc, bsc) = play(game, a_blue, a, v1);
            n += 1;
            if asc > bsc {
                w += 1;
                if a_blue { wb += 1; } else { wr += 1; }
            }
            println!("H {aname} vs v1 open={open:?} a={} {asc:.0}-{bsc:.0} {}",
                if a_blue { "blue" } else { "red" }, if asc > bsc { "A WINS" } else { "b wins" });
        }
    }
    println!("==> H {aname} vs v1: {w}/{n} (blue {wb}/5, red {wr}/5)");
}

fn gate_league(aname: &str, a: fn(&Position) -> Option<Move>) {
    let mut sum = 0.0;
    let mut n = 0;
    let (mut sb, mut sr) = (0.0, 0.0);
    for skip in [0usize, 10, 20, 30] {
        for ret_blue in [true, false] {
            let mut game = Game::new();
            for _ in 0..skip {
                if game.is_over() { break; }
                let mv = retaliator::search::best_move(game.position()).unwrap();
                game.play(mv).unwrap();
            }
            if game.is_over() { continue; }
            let (rs, ss) = play(game, ret_blue, a, scoutbase::best_move);
            let m = (rs - ss) / rs * 100.0;
            println!("L {aname} skip={skip} ret={} rs={rs:.0} ss={ss:.0} margin={m:+.1}%",
                if ret_blue { "blue" } else { "red" });
            sum += m; n += 1;
            if ret_blue { sb += m; } else { sr += m; }
        }
    }
    println!("==> L {aname}: AVG {:.1}% over {n} (blue-avg {:.1}%, red-avg {:.1}%)",
        sum / n as f64, sb / 4.0, sr / 4.0);
}

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

fn gate_gauge(aname: &str, a: fn(&Position) -> Option<Move>) {
    let mut w = 0;
    for g in 0..6 {
        let a_blue = g % 2 == 0;
        let mut game = Game::new();
        while !game.is_over() {
            let a_moves = (game.position().to_move() == Player::Blue) == a_blue;
            game.play(if a_moves { a(game.position()) } else { greedy(game.position()) }.unwrap()).unwrap();
        }
        let (rs, gs) = if a_blue {
            (game.position().score(Player::Blue).to_f64(), game.position().score(Player::Red).to_f64())
        } else {
            (game.position().score(Player::Red).to_f64(), game.position().score(Player::Blue).to_f64())
        };
        let m = (rs - gs) / rs * 100.0;
        if rs > gs { w += 1; }
        println!("G {aname} game{} a={} {rs:.0}-{gs:.0} margin={m:+.1}% {}",
            g + 1, if a_blue { "blue" } else { "red" }, if rs > gs { "A WINS" } else { "b wins" });
    }
    println!("==> G {aname} vs greedy: {w}/6");
}

fn main() {
    println!("=== CONTROL (search::best_move) ===");
    gate_h2h("CONTROL", base);
    gate_league("CONTROL", base);
    gate_gauge("CONTROL", base);
    println!("=== DOSE D1 (blue-only contest, BONUS=0) ===");
    gate_h2h("DOSE-D1", dose);
    gate_league("DOSE-D1", dose);
    gate_gauge("DOSE-D1", dose);
}
