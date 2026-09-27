//! Lane A probe: `search_deep` (negamax alpha-beta, depth 3) against the gates.
//!
//! Modes (arg 1, default `all`):
//! - `h2h` — search_deep::best_move vs search::best_move (the v3 baseline),
//!   openings x 2 colors (arg 2 = how many of the 4 openings, default 4).
//!   Both engines are deterministic, so the openings are what makes the games
//!   differ (same device as probe_v3all).
//! - `league` — probe_league.rs structure with search_deep substituted for
//!   the baseline: scoutbase opponent, skip prelude (played by
//!   search::best_move, as in the baseline), both colors. Arg 2 = comma
//!   separated skips (default 0,10,20,30).
//! - `gauge` — gauge.rs structure with search_deep substituted: greedy
//!   max-area 1-ply opponent (arg 2 = games, default 6), alternating colors.
//!
//! The deep search keeps the baseline's evaluation, opener and root priority
//! verbatim, so any difference against `search::best_move` isolates the
//! search itself. Games print progress every 12 actions: the depth-3 search
//! costs tens of seconds CPU per move, so the pace has to be visible.

#[path = "support/scoutbase.rs"]
mod scoutbase;

use meridian_engine::{Game, Move, Player, Position};

/// The baseline strategy (v3 + v4 terms, 2-ply width-8).
fn base(position: &Position) -> Option<Move> {
    retaliator::search::best_move(position)
}

/// The candidate: real alpha-beta with move ordering, depth 3.
fn deep(position: &Position) -> Option<Move> {
    retaliator::search_deep::best_move(position)
}

/// 1-ply greedy: maximize own enclosed area after the move, ties by move id.
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

/// Plays `a` vs `b` from `game`, returning a's and b's scores.
fn play(mut game: Game, a_blue: bool, a: fn(&Position) -> Option<Move>, b: fn(&Position) -> Option<Move>) -> (f64, f64) {
    let t = std::time::Instant::now();
    let mut last = 0u8;
    while !game.is_over() {
        let a_moves = (game.position().to_move() == Player::Blue) == a_blue;
        game.play(if a_moves { a(game.position()) } else { b(game.position()) }.unwrap()).unwrap();
        let played = game.position().actions_played();
        if played >= last + 12 {
            last = played;
            println!("  t={played} elapsed={:.0}s", t.elapsed().as_secs_f64());
        }
    }
    let asc = game.position().score(if a_blue { Player::Blue } else { Player::Red }).to_f64();
    let bsc = game.position().score(if a_blue { Player::Red } else { Player::Blue }).to_f64();
    (asc, bsc)
}

fn h2h(openings: usize) {
    println!("== h2h: search_deep (depth 3) vs search baseline, {openings} openings x 2 colors ==");
    let opens = [None, Some(4864usize), Some(5589), Some(9199)];
    let (mut w, mut n) = (0, 0);
    let (mut w_blue, mut n_blue, mut w_red, mut n_red) = (0, 0, 0, 0);
    for open in opens.into_iter().take(openings.max(1)) {
        for deep_blue in [true, false] {
            let mut game = Game::new();
            if let Some(id) = open {
                game.play(Move::from_index(id).unwrap()).unwrap();
            }
            let t = std::time::Instant::now();
            let (dsc, bsc) = play(game, deep_blue, deep, base);
            n += 1;
            let won = dsc > bsc;
            if won { w += 1; }
            if deep_blue { n_blue += 1; if won { w_blue += 1; } } else { n_red += 1; if won { w_red += 1; } }
            println!(
                "open={open:?} deep={} {dsc:.0}-{bsc:.0} {} ({:.0}s)",
                if deep_blue { "blue" } else { "red" },
                if won { "DEEP WINS" } else { "base wins" },
                t.elapsed().as_secs_f64()
            );
        }
    }
    println!("==> h2h search_deep vs search: {w}/{n} (blue {w_blue}/{n_blue}, red {w_red}/{n_red})");
}

fn league(skips: &[usize]) {
    println!("== league: search_deep (depth 3) vs scoutbase, skips {skips:?}, both colors ==");
    let mut sum = 0.0;
    let mut n = 0;
    for skip in skips {
        for ret_blue in [true, false] {
            let mut game = Game::new();
            for _ in 0..*skip {
                if game.is_over() { break; }
                let mv = base(game.position()).unwrap();
                game.play(mv).unwrap();
            }
            if game.is_over() { continue; }
            let t = std::time::Instant::now();
            let (rs, ss) = play(game, ret_blue, deep, scoutbase::best_move);
            let m = (rs - ss) / rs * 100.0;
            println!("skip={skip} ret={} rs={rs:.0} ss={ss:.0} margin={m:+.1}% ({:.0}s)", if ret_blue { "blue" } else { "red" }, t.elapsed().as_secs_f64());
            sum += m;
            n += 1;
        }
    }
    println!("==> league AVG margin (ret perspective): {:+.1}% over {n} games", sum / n.max(1) as f64);
}

fn gauge(games: usize) {
    println!("== gauge: search_deep (depth 3) vs greedy, {games} games ==");
    let mut wins_r = 0;
    for g in 0..games {
        let ret_blue = g % 2 == 0;
        let t = std::time::Instant::now();
        let (rs, gs) = play(Game::new(), ret_blue, deep, greedy);
        let margin = (rs - gs) / rs * 100.0;
        if rs > gs {
            wins_r += 1;
        }
        println!(
            "game {}: deep={} blue={:.1} red={:.1} margin={:+.1}% ({:.0}s)",
            g + 1,
            if ret_blue { "blue" } else { "red" },
            if ret_blue { rs } else { gs },
            if ret_blue { gs } else { rs },
            margin,
            t.elapsed().as_secs_f64()
        );
    }
    println!("==> deep wins: {wins_r}/{games}");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).cloned().unwrap_or_else(|| "all".into());
    let n1 = args.get(2).and_then(|s| s.parse::<usize>().ok());
    match mode.as_str() {
        "h2h" => h2h(n1.unwrap_or(4)),
        "league" => {
            let skips: Vec<usize> = args
                .get(2)
                .map(|s| s.split(',').filter_map(|p| p.parse().ok()).collect())
                .unwrap_or_else(|| vec![0, 10, 20, 30]);
            league(&skips);
        }
        "gauge" => gauge(n1.unwrap_or(6)),
        _ => {
            h2h(4);
            league(&[0, 10, 20, 30]);
            gauge(6);
        }
    }
}
