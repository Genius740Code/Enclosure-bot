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

/// The candidate: real alpha-beta with move ordering, depth 3 (unbounded).
fn deep(position: &Position) -> Option<Move> {
    retaliator::search_deep::best_move(position)
}

/// The candidate: real alpha-beta with move ordering, depth 3, 2s budget (FULL_WIDTH).
fn deep_budget(position: &Position) -> Option<Move> {
    retaliator::search_deep::best_move_with_budget(
        position,
        retaliator::search_deep::DEFAULT_DEPTH,
        &[],
        &retaliator::search_deep::Budget::Ms(retaliator::search_deep::DEFAULT_BUD_MS),
    )
}

/// The candidate: real alpha-beta with move ordering, depth 3, 2s budget, DEFAULT_WIDTH (deployment config).
fn deep_capped_budget(position: &Position) -> Option<Move> {
    retaliator::search_deep::best_move_capped(
        position,
        retaliator::search_deep::DEFAULT_DEPTH,
        &[],
        &retaliator::search_deep::Budget::Ms(retaliator::search_deep::DEFAULT_BUD_MS),
        retaliator::search_deep::DEFAULT_WIDTH,
    )
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

fn h2h_budget(openings: usize) {
    println!("== h2h_budget: search_deep (depth 3, 2s budget) vs search baseline, {openings} openings x 2 colors ==");
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
            let (dsc, bsc) = play(game, deep_blue, deep_budget, base);
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
    println!("==> h2h_budget search_deep vs search: {w}/{n} (blue {w_blue}/{n_blue}, red {w_red}/{n_red})");
}

fn h2h_capped_budget(openings: usize) {
    println!("== h2h_capped_budget: search_deep (depth 3, 2s budget, DEFAULT_WIDTH) vs search baseline, {openings} openings x 2 colors ==");
    let opens = [None, Some(4864usize), Some(5589), Some(11723), Some(9199)];
    let (mut w, mut n) = (0, 0);
    let (mut w_blue, mut n_blue, mut w_red, mut n_red) = (0, 0, 0, 0);
    for open in opens.into_iter().take(openings.max(1)) {
        for deep_blue in [true, false] {
            let mut game = Game::new();
            if let Some(id) = open {
                game.play(Move::from_index(id).unwrap()).unwrap();
            }
            let t = std::time::Instant::now();
            let (dsc, bsc) = play(game, deep_blue, deep_capped_budget, base);
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
    println!("==> h2h_capped_budget search_deep vs search: {w}/{n} (blue {w_blue}/{n_blue}, red {w_red}/{n_red})");
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

fn league_capped_budget(skips: &[usize]) {
    println!("== league_capped_budget: search_deep (depth 3, 2s budget, DEFAULT_WIDTH) vs scoutbase, skips {skips:?}, both colors ==");
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
            let (rs, ss) = play(game, ret_blue, deep_capped_budget, scoutbase::best_move);
            let m = (rs - ss) / rs * 100.0;
            println!("skip={skip} ret={} rs={rs:.0} ss={ss:.0} margin={m:+.1}% ({:.0}s)", if ret_blue { "blue" } else { "red" }, t.elapsed().as_secs_f64());
            sum += m;
            n += 1;
        }
    }
    println!("==> league_capped_budget AVG margin (ret perspective): {:+.1}% over {n} games", sum / n.max(1) as f64);
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

fn gauge_budget(games: usize) {
    println!("== gauge_budget: search_deep (depth 3, 2s budget) vs greedy, {games} games ==");
    let mut wins_r = 0;
    for g in 0..games {
        let ret_blue = g % 2 == 0;
        let t = std::time::Instant::now();
        let (rs, gs) = play(Game::new(), ret_blue, deep_budget, greedy);
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

fn gauge_capped_budget(games: usize) {
    println!("== gauge_capped_budget: search_deep (depth 3, 2s budget, DEFAULT_WIDTH) vs greedy, {games} games ==");
    let mut wins_r = 0;
    for g in 0..games {
        let ret_blue = g % 2 == 0;
        let t = std::time::Instant::now();
        let (rs, gs) = play(Game::new(), ret_blue, deep_capped_budget, greedy);
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

/// CPU time (utime + stime) of this process, from /proc/self/stat. Clock ticks
/// are 100/s, so the resolution is 10 ms; load-independent, unlike wall time.
fn cpu_seconds() -> f64 {
    let stat = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let Some(rest) = stat.rsplit_once(')').map(|(_, rest)| rest) else { return 0.0 };
    let fields = rest.split_whitespace().collect::<Vec<_>>();
    // fields[0] is /proc's field 3 (state), so field 14 is fields[11].
    let utime: u64 = fields.get(11).and_then(|s| s.parse().ok()).unwrap_or(0);
    let stime: u64 = fields.get(12).and_then(|s| s.parse().ok()).unwrap_or(0);
    f64::from(u32::try_from(utime + stime).unwrap_or(0)) / 100.0
}

/// Baseline cost: one real `analyze(3)` per position (arg 2 = runs per
/// position, default 1). Positions come from baseline (`search::best_move`)
/// play, as in the gates. Reports nodes, wall, CPU and CPU nodes/sec.
fn bench(runs: usize) {
    println!("== bench: one analyze(3) per position (wall + cpu), {runs} run(s) ==");
    for open in [None, Some(4864usize)] {
        let mut game = Game::new();
        if let Some(id) = open {
            game.play(Move::from_index(id).unwrap()).unwrap();
        }
        for target in [12usize, 13, 24, 25, 48, 49, 72, 73, 96, 97] {
            while !game.is_over() && usize::from(game.position().actions_played()) < target {
                let mv = base(game.position()).unwrap();
                game.play(mv).unwrap();
            }
            if game.is_over() {
                break;
            }
            let pos = game.position();
            for _r in 0..runs.max(1) {
                let w0 = std::time::Instant::now();
                let c0 = cpu_seconds();
                let analysis = retaliator::search_deep::analyze(&pos, 3);
                let wall = w0.elapsed().as_secs_f64();
                let cpu = cpu_seconds() - c0;
                let ns = if cpu > 0.0 { analysis.nodes as f64 / cpu / 1000.0 } else { 0.0 };
                println!(
                    "  t={} open={open:?}: nodes={} wall={wall:.1}s cpu={cpu:.1}s ({ns:.1}k n/s cpu) best={} eval={:.2} tt={}/{} hits/stores",
                    pos.actions_played(),
                    analysis.nodes,
                    analysis
                        .candidates
                        .first()
                        .map(|c| c.mv.index())
                        .unwrap_or(usize::MAX),
                    analysis.evaluation,
                    analysis.tt_hits,
                    analysis.tt_stores
                );
                std::hint::black_box(&analysis);
            }
        }
    }
}

/// Which per-node components dominate: calls the pieces of the `negamax`
/// children loop in isolation, N iterations each (arg 2 = N, default 20000),
/// and reports CPU ns/call. Run at a t=24 baseline position.
fn profile(iterations: usize) {
    let mut game = Game::new();
    game.play(Move::from_index(4864usize).unwrap()).unwrap();
    while !game.is_over() && game.position().actions_played() < 24 {
        let mv = base(game.position()).unwrap();
        game.play(mv).unwrap();
    }
    let pos = game.position();
    let legal: Vec<Move> = pos
        .legal_moves()
        .iter()
        .filter(|&mv| !retaliator::search_deep::repeats_a_connection(pos, mv))
        .collect();
    let n = iterations.max(1);
    println!(
        "== profile at t={} (nodes {}/{} per side, {n} iterations each) ==",
        pos.actions_played(),
        pos.nodes(Player::Blue).len(),
        pos.nodes(Player::Red).len()
    );
    let timed = |label: &str, iterations: usize, run: &dyn Fn()| {
        let c0 = cpu_seconds();
        for _ in 0..iterations {
            run();
        }
        let per = (cpu_seconds() - c0) * 1e9 / iterations.max(1) as f64;
        println!("  {label}: {per:.0} ns/call");
    };
    let per_child = (n / legal.len().max(1)).max(1);
    let b = legal.len();
    let apply_all = move || {
        for &mv in &legal {
            let mut after = pos.clone();
            after.apply_unchecked(mv);
            std::hint::black_box(&after);
        }
    };
    timed("legal_moves()", n, &|| {
        std::hint::black_box(pos.legal_moves().len());
    });
    timed("evaluate()", n, &|| {
        std::hint::black_box(retaliator::search_deep::evaluate(&pos));
    });
    timed("has_legal_move()", n, &|| {
        std::hint::black_box(retaliator::search_deep::has_legal_move(&pos));
    });
    timed("room() both", n, &|| {
        std::hint::black_box(retaliator::search_deep::room(&pos, Player::Blue));
        std::hint::black_box(retaliator::search_deep::room(&pos, Player::Red));
    });
    timed("one_edge_nodes() both", n, &|| {
        std::hint::black_box(retaliator::search_deep::one_edge_nodes(&pos, Player::Blue));
        std::hint::black_box(retaliator::search_deep::one_edge_nodes(&pos, Player::Red));
    });
    timed("clone+apply, per child", per_child, &apply_all);
    timed("cheap eval (score+area)", n, &|| {
        std::hint::black_box(retaliator::search_deep::cheap_value(&pos));
    });
    println!("  branching b = {b}");
}

/// Fast-path exactness (no args): walks a baseline self-play game and, at
/// every position, compares every legal move's fast [`ChildFacts`] against a
/// real clone+apply. The fast path claims the engine would change no area;
/// any move where it would show up here as an f64 bit mismatch.
fn exact() {
    let mut game = Game::new();
    let (mut fast, mut slow, mut mismatches) = (0usize, 0usize, 0usize);
    loop {
        let pos = game.position().clone();
        let mover = pos.to_move();
        for mv in pos.legal_moves().iter() {
            let danger = retaliator::search_deep::source_danger(&pos, mv.source);
            match retaliator::search_deep::child_facts(&pos, mv, danger) {
                Some(facts) => {
                    fast += 1;
                    let mut after = pos.clone();
                    let outcome = after.apply_unchecked(mv);
                    let gained = (after.area(mover).to_f64() - pos.area(mover).to_f64()).max(0.0);
                    let destroyed =
                        (pos.area(mover.opponent()).to_f64() - after.area(mover.opponent()).to_f64()).max(0.0);
                    let banked = pos.actions_played() % 2 == 0 || after.is_finished();
                    let ok = retaliator::search_deep::cheap_value(&after).to_bits() == facts.val.to_bits()
                        && (gained + destroyed).to_bits() == facts.swing.to_bits()
                        && after.is_finished() == facts.finished
                        && after.scoring_events_left() == facts.events
                        && outcome.scored.is_some() == banked;
                    if !ok {
                        mismatches += 1;
                        println!("MISMATCH t={} mv={mv:?} fast={facts:?} after val={}", pos.actions_played(), retaliator::search_deep::cheap_value(&after));
                    }
                }
                None => slow += 1,
            }
        }
        if game.is_over() {
            break;
        }
        let mv = base(game.position()).unwrap();
        game.play(mv).unwrap();
    }
    println!(
        "== exact: {fast} fast children bit-identical to clone+apply, {slow} materialized, {mismatches} mismatches =="
    );
}

/// Budgeted cost: one `analyze_with_budget(…, 3, Budget::Ms(ms))` per
/// position (arg 2 = ms, default 2000): the deployment line. Reports CPU
/// per move, the depth reached, and how many root children were searched
/// (fewer than the legal count = the cap truncated the root).
fn benchbudget(ms: u64) {
    println!("== benchbudget: ID depth<=3 under Budget::Ms({ms}) per position ==");
    for open in [None, Some(4864usize)] {
        let mut game = Game::new();
        if let Some(id) = open {
            game.play(Move::from_index(id).unwrap()).unwrap();
        }
        for target in [12usize, 13, 24, 25, 48, 49, 72, 73, 96, 97] {
            while !game.is_over() && usize::from(game.position().actions_played()) < target {
                let mv = base(game.position()).unwrap();
                game.play(mv).unwrap();
            }
            if game.is_over() {
                break;
            }
            let pos = game.position();
            let legal = pos.legal_moves().len();
            let c0 = cpu_seconds();
            let analysis = retaliator::search_deep::analyze_with_budget(
                &pos,
                3,
                &[],
                &retaliator::search_deep::Budget::Ms(ms),
            );
            let cpu = cpu_seconds() - c0;
            println!(
                "  t={} open={open:?}: depth={} nodes={} cpu={cpu:.2}s root={}/{} best={} eval={:.2}",
                pos.actions_played(),
                analysis.depth,
                analysis.nodes,
                analysis.candidates.len(),
                legal,
                analysis.candidates.first().map(|c| c.mv.index()).unwrap_or(usize::MAX),
                analysis.evaluation
            );
            std::hint::black_box(&analysis);
        }
    }
}

/// Capped cost: `analyze_capped(…, 3, Budget::Unbounded, DEFAULT_WIDTH)` per
/// position — the gate configuration (8 root / 6 inner, deterministic).
/// The affordability table for the width-capped depth-3 verdict.
fn benchcap() {
    println!("== benchcap: depth-3, Budget::Unbounded, DEFAULT_WIDTH (8 root / 6 inner) ==");
    for open in [None, Some(4864usize)] {
        let mut game = Game::new();
        if let Some(id) = open {
            game.play(Move::from_index(id).unwrap()).unwrap();
        }
        for target in [12usize, 13, 24, 25, 48, 49, 72, 73, 96, 97] {
            while !game.is_over() && usize::from(game.position().actions_played()) < target {
                let mv = base(game.position()).unwrap();
                game.play(mv).unwrap();
            }
            if game.is_over() {
                break;
            }
            let pos = game.position();
            let legal = pos.legal_moves().len();
            let c0 = cpu_seconds();
            let analysis = retaliator::search_deep::analyze_capped(
                &pos,
                3,
                &[],
                &retaliator::search_deep::Budget::Unbounded,
                retaliator::search_deep::DEFAULT_WIDTH,
            );
            let cpu = cpu_seconds() - c0;
            println!(
                "  t={} open={open:?}: nodes={} wall cpu={cpu:.2}s root={}/{} best={} eval={:.2} tt={}/{}",
                pos.actions_played(),
                analysis.nodes,
                analysis.candidates.len(),
                legal,
                analysis.candidates.first().map(|c| c.mv.index()).unwrap_or(usize::MAX),
                analysis.evaluation,
                analysis.tt_hits,
                analysis.tt_stores
            );
            std::hint::black_box(&analysis);
        }
    }
}

/// Q14 ablation: killer/history ordering ON vs OFF (XBot alone), exact search
/// (`Budget::Unbounded`, no aspiration) at capped width and at full width.
/// Prints per-position nodes/best/eval both ways plus totals: the gate is
/// nodes/pos down >20% with zero best-move/eval mismatches (value-exact).
fn benchorder() {
    use retaliator::search_deep::{Budget, FULL_WIDTH, DEFAULT_WIDTH};
    println!("== benchorder: KH on vs off, Unbounded depth-3, capped + full width ==");
    let (mut cap_off, mut cap_on, mut full_off, mut full_on) = (0u64, 0u64, 0u64, 0u64);
    let (mut mism, mut n) = (0u32, 0u32);
    for open in [None, Some(4864usize)] {
        let mut game = Game::new();
        if let Some(id) = open {
            game.play(Move::from_index(id).unwrap()).unwrap();
        }
        for target in [12usize, 13, 24, 25, 48, 49] {
            while !game.is_over() && usize::from(game.position().actions_played()) < target {
                let mv = base(game.position()).unwrap();
                game.play(mv).unwrap();
            }
            if game.is_over() {
                break;
            }
            let pos = game.position().clone();
            let run = |width, kh: bool| {
                retaliator::search_deep::analyze_capped_kh(&pos, 3, &[], &Budget::Unbounded, width, kh)
            };
            let c0 = run(DEFAULT_WIDTH, false);
            let c1 = run(DEFAULT_WIDTH, true);
            // Full width is orders of magnitude pricier: only the two cheapest
            // (turn-start) positions per game carry it; the capped width (the
            // production width) covers all six.
            let (f0, f1, fb0, fb1, full_ok) = if target == 12 || target == 13 {
                let f0 = run(FULL_WIDTH, false);
                let f1 = run(FULL_WIDTH, true);
                let fb0 = f0.candidates.first().map(|c| c.mv.index()).unwrap_or(usize::MAX);
                let fb1 = f1.candidates.first().map(|c| c.mv.index()).unwrap_or(usize::MAX);
                full_off += f0.nodes;
                full_on += f1.nodes;
                let ok = fb0 == fb1 && f0.evaluation.to_bits() == f1.evaluation.to_bits();
                (f0.nodes, f1.nodes, fb0, fb1, ok)
            } else {
                (0, 0, usize::MAX, usize::MAX, true)
            };
            let b0 = c0.candidates.first().map(|c| c.mv.index()).unwrap_or(usize::MAX);
            let b1 = c1.candidates.first().map(|c| c.mv.index()).unwrap_or(usize::MAX);
            let ok = b0 == b1
                && c0.evaluation.to_bits() == c1.evaluation.to_bits()
                && full_ok;
            if !ok {
                mism += 1;
            }
            n += 1;
            cap_off += c0.nodes;
            cap_on += c1.nodes;
            println!(
                "  t={} open={open:?}: cap off={} on={} best={b0}/{b1} | full off={f0} on={f1} best={fb0}/{fb1} {}",
                pos.actions_played(),
                c0.nodes,
                c1.nodes,
                if ok { "EXACT" } else { "MISMATCH" }
            );
            std::hint::black_box((&c0, &c1));
        }
    }
    let pct = |off: u64, on: u64| (off as f64 - on as f64) / off.max(1) as f64 * 100.0;
    println!(
        "==> benchorder over {n} positions, {mism} mismatches: capped {cap_off}->{cap_on} ({:+.1}%), full {full_off}->{full_on} ({:+.1}%)",
        pct(cap_off, cap_on),
        pct(full_off, full_on)
    );
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
        "verify" => verify(),
        "exact" => exact(),
        "bench" => bench(n1.unwrap_or(1)),
        "benchbudget" => benchbudget(n1.unwrap_or(2000) as u64),
        "benchcap" => benchcap(),
        "gauge_budget" => gauge_budget(n1.unwrap_or(6)),
        "h2h_budget" => h2h_budget(n1.unwrap_or(5)),
        "gauge_capped_budget" => gauge_capped_budget(n1.unwrap_or(6)),
        "h2h_capped_budget" => h2h_capped_budget(n1.unwrap_or(5)),
        "benchorder" => benchorder(),
        "league_capped_budget" => {
            let skips: Vec<usize> = args
                .get(2)
                .map(|s| s.split(',').filter_map(|p| p.parse().ok()).collect())
                .unwrap_or_else(|| vec![0, 10, 20, 30]);
            league_capped_budget(&skips);
        }
        "profile" => profile(n1.unwrap_or(20000)),
        _ => {
            h2h(4);
            league(&[0, 10, 20, 30]);
            gauge(6);
        }
    }
}

/// The deep search's draw check must agree with the engine's own movegen at
/// every position: `has_legal_move` == `!legal_moves().is_empty()`. Walks
/// diverse games (five openings, then stretches of base / greedy / scout
/// play) and checks every position along the way.
fn verify() {
    let mut checked = 0u64;
    let mut bad = 0u64;
    for open in [None, Some(4864usize), Some(5589), Some(9199), Some(11723)] {
        let mut game = Game::new();
        if let Some(id) = open {
            game.play(Move::from_index(id).unwrap()).unwrap();
        }
        for stretch in 0..3usize {
            let player_fn = match stretch {
                0 => base as fn(&Position) -> Option<Move>,
                1 => greedy as fn(&Position) -> Option<Move>,
                _ => scoutbase::best_move as fn(&Position) -> Option<Move>,
            };
            for _ in 0..40 {
                if game.is_over() { break; }
                let pos = game.position();
                let engine_says_none = pos.legal_moves().is_empty();
                let deep_says_none = !retaliator::search_deep::has_legal_move(pos);
                checked += 1;
                if engine_says_none != deep_says_none {
                    bad += 1;
                    println!("MISMATCH at action {}: engine_empty={engine_says_none} deep_empty={deep_says_none}", pos.actions_played());
                }
                let Some(mv) = player_fn(pos) else { break };
                game.play(mv).unwrap();
            }
        }
    }
    println!("verify: {checked} positions checked, {bad} mismatches");
}
