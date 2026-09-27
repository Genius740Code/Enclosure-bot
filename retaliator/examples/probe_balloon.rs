//! Lane C read-only probe (autopsy): the red collapse line — ret (current
//! search) as RED vs Scout, skip=10/20/30 pre-moves like probe_league.
//! Characterizes the doomed shape around actions 40-90: per red turn the
//! enemy's worst one-action pop available (max_pop), our hull/room, our loop
//! count (independent cycles in our edge graph), and every big break with the
//! exact action, cut edge, and location. Does not change bot behavior.

#[path = "support/scoutbase.rs"]
mod scoutbase;

use meridian_engine::{Game, Edge, Player, Point};
use std::collections::{HashMap, HashSet};

/// Worst our-area loss across the enemy's legal replies from `pos` (enemy to move).
fn max_pop(pos: &Game, victim: Player) -> f64 {
    let held = pos.position().area(victim).to_f64();
    let mut worst: f64 = 0.0;
    for mv in pos.position().legal_moves().iter() {
        let mut next = pos.position().clone();
        next.apply_unchecked(mv);
        worst = worst.max(held - next.area(victim).to_f64());
    }
    worst.max(0.0)
}

/// Independent cycles in `player`'s edge graph (E - V + C summed per component):
/// an approximation of how many separate enclosed loops the shape holds.
fn loop_count(pos: &Game, player: Player) -> (usize, usize) {
    let edges: Vec<Edge> = pos.position().edges(player).iter().collect();
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for e in &edges {
        let (a, b) = (e.origin().index(), e.far().index());
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }
    let mut seen: HashSet<usize> = HashSet::new();
    let (mut comp, mut cycles) = (0usize, 0usize);
    for start in adj.keys().copied().collect::<Vec<_>>() {
        if seen.contains(&start) {
            continue;
        }
        comp += 1;
        let (mut v, mut e) = (0usize, 0usize);
        let mut stack = vec![start];
        seen.insert(start);
        while let Some(u) = stack.pop() {
            v += 1;
            for &w in &adj[&u] {
                e += 1;
                if !seen.contains(&w) {
                    seen.insert(w);
                    stack.push(w);
                }
            }
        }
        e /= 2;
        cycles += e.saturating_sub(v.saturating_sub(1)); // E-(V-1) per component = independent cycles
    }
    (comp, cycles)
}

fn hull_side(points: impl Iterator<Item = Point>) -> Vec<Point> {
    let cross = |a: Point, b: Point, c: Point| -> i32 {
        i32::from(b.x() - a.x()) * i32::from(c.y() - a.y())
            - i32::from(b.y() - a.y()) * i32::from(c.x() - a.x())
    };
    let mut chain: Vec<Point> = Vec::new();
    for p in points {
        while chain.len() >= 2 && cross(chain[chain.len() - 2], chain[chain.len() - 1], p) <= 0 {
            chain.pop();
        }
        chain.push(p);
    }
    chain.pop();
    chain
}

fn room_of(pos: &Game, player: Player) -> f64 {
    let mut points: Vec<Point> = pos.position().nodes(player).iter().collect();
    points.sort_by_key(|p| (p.x(), p.y()));
    if points.len() < 3 {
        return 0.0;
    }
    let cross = |a: Point, b: Point, c: Point| -> i32 {
        i32::from(b.x() - a.x()) * i32::from(c.y() - a.y())
            - i32::from(b.y() - a.y()) * i32::from(c.x() - a.x())
    };
    let mut hull = hull_side(points.iter().copied());
    hull.extend(hull_side(points.iter().rev().copied()));
    let first = hull[0];
    hull.windows(2).map(|pair| cross(first, pair[0], pair[1]) as f64).sum::<f64>().abs() * 0.5
}

fn bbox(pos: &Game, player: Player) -> (i8, i8, i8, i8) {
    let pts: Vec<Point> = pos.position().nodes(player).iter().collect();
    let (mut x0, mut x1, mut y0, mut y1) = (i8::MAX, i8::MIN, i8::MAX, i8::MIN);
    for p in pts {
        x0 = x0.min(p.x());
        x1 = x1.max(p.x());
        y0 = y0.min(p.y());
        y1 = y1.max(p.y());
    }
    (x0, x1, y0, y1)
}

fn min_enemy_dist(pos: &Game, player: Player) -> i32 {
    let own: Vec<Point> = pos.position().nodes(player).iter().collect();
    let foe: Vec<Point> = pos.position().nodes(player.opponent()).iter().collect();
    let mut best = i32::MAX;
    for a in &own {
        for b in &foe {
            best = best.min(i32::from((a.x() - b.x()).abs().max((a.y() - b.y()).abs())));
        }
    }
    best
}

fn play(mut game: Game, ret_blue: bool, skip_label: &str) {
    let mut t = 0usize;
    let (mut cut_us, mut cut_them) = (0usize, 0usize);
    let mut big = 0usize; // big-break diagnostics printed (capped)
    while !game.is_over() {
        let red_to_move = game.position().to_move() == Player::Red;
        let _ = red_to_move;
        let ours = (game.position().to_move() == Player::Blue) == ret_blue;
        let mv = if ours {
            retaliator::search::best_move(game.position())
        } else {
            scoutbase::best_move(game.position())
        }
        .unwrap();
        let b0 = game.position().area(Player::Blue).to_f64();
        let r0 = game.position().area(Player::Red).to_f64();
        // At the position BEFORE the move: what the enemy could pop next.
        let pop_available = if ours { max_pop(&game, if ret_blue { Player::Blue } else { Player::Red }) } else { 0.0 };
        let (hull, lc) = if ours {
            (room_of(&game, if ret_blue { Player::Blue } else { Player::Red }),
             loop_count(&game, if ret_blue { Player::Blue } else { Player::Red }))
        } else {
            (0.0, (0, 0))
        };
        let oc = game.play(mv).unwrap();
        t += 1;
        let (b1, r1) = (game.position().area(Player::Blue).to_f64(), game.position().area(Player::Red).to_f64());
        let my_drop = if ret_blue { b0 - b1 } else { r0 - r1 };
        if oc.broken.is_some() {
            if ours { cut_them += 1; } else { cut_us += 1; }
        }
        // Per-turn doomed-shape read, t=40..90, on OUR turns.
        if t >= 40 && t <= 90 && ours && t % 2 == 1 {
            println!(
                "t={t} us red_area {:.1}->{:.1} hull={hull:.0} comps={} loops={} maxEnemyPop={pop_available:.1} minDist={}",
                r0, r1, lc.0, lc.1, min_enemy_dist(&game, if ret_blue { Player::Blue } else { Player::Red })
            );
        }
        // Every big enemy break or our big area drop: exact t, move, cut edge.
        if big < 12 && t >= 35 && (oc.broken.is_some() && my_drop > 4.0 || (!oc.broken.is_some() && my_drop > 8.0)) {
            big += 1;
            let broken = oc.broken.map(|e| format!(
                " cut {}->{}",
                meridian_engine::notation::move_text(e.origin(), e.far()),
                meridian_engine::notation::move_text(oc.placed.origin(), oc.placed.far())
            )).unwrap_or_default();
            println!(
                "t={t} {} myArea-{my_drop:.1} mv={} kind={:?}{broken}",
                if ours { "US-EVENT " } else { "OPP-BREAK" },
                mv.index(),
                oc.kind
            );
        }
        if t % 10 == 0 || game.is_over() {
            println!(
                "= after {t}: b={b1:.1}/{:.0} r={r1:.1}/{:.0}",
                game.position().score(Player::Blue).to_f64(),
                game.position().score(Player::Red).to_f64()
            );
        }
    }
    let (bs, rs) = (game.position().score(Player::Blue).to_f64(), game.position().score(Player::Red).to_f64());
    println!(
        "==> {skip_label} ret={} FINAL {bs:.0}-{rs:.0} {} | cuts by us={} suffered={}",
        if ret_blue { "blue" } else { "red" },
        if (if ret_blue { bs } else { rs }) > (if ret_blue { rs } else { bs }) { "RET WINS" } else { "ret loses" },
        cut_them, cut_us
    );
}

fn main() {
    for skip in [10usize, 20, 30] {
        let mut game = Game::new();
        for _ in 0..skip {
            if game.is_over() { break; }
            let mv = retaliator::search::best_move(game.position()).unwrap();
            game.play(mv).unwrap();
        }
        play(game, false, &format!("skip={skip}"));
    }
}
