//! Lane 7 ANGEL-CIRCLE census (read-only): how often the OPPONENT encircles
//! our nodes — closes a loop whose interior contains our nodes — what shape,
//! and what breaks it.
//!
//! A **circle** = after an opponent close (a Connect gaining real area), the
//! opponent's enclosed territory contains at least one of our nodes. Trapped
//! nodes can only expand inside the enemy wall (crossing an enemy edge is
//! illegal), which is the encircle-and-bank shape the roadmap targets
//! (AngelWASM priority).
//!
//! Territories are the engine's own face computation (union of closed loops),
//! ported from the site JS engine (`engine.js` `nv`/`_0`/`A0`/`O0`): planar
//! graph per connected component, directed-edge outer walks with positive
//! signed area, components nested inside another's face dropped. The probe
//! validates the port against the engine's `area()` at every game end.
//!
//! Shape recorded per circle: action, gained area, our nodes inside (share
//! of ours), opponent edge count and 2+-degree node count (wall thickness),
//! phase. What breaks it: at the first our-turn after the circle — how many
//! circle-boundary edges a legal cut can break and the max destroyed; for
//! the rest of the game — whether we ever cut a circle edge, whether the
//! opponent's area ever fell back below the pre-circle level (bank lost),
//! and the game outcome.
//!
//! Usage: cargo run --release --example probe_angel -- <path-or-dir>...
//! Read-only: replays recorded games, changes no bot behavior.

#[path = "support/gamejson.rs"]
mod gamejson;

use gamejson::{game_files, RecordedGame};
use meridian_engine::{Game, MoveKind, Player, Point};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

type P2 = (f64, f64);

const EPS: f64 = 1e-9;

fn pt2(p: Point) -> P2 {
    (p.x() as f64, p.y() as f64)
}

fn key(p: P2) -> (i64, i64) {
    ((p.0 / EPS).round() as i64, (p.1 / EPS).round() as i64)
}

fn same(a: P2, b: P2) -> bool {
    (a.0 - b.0).abs() < EPS && (a.1 - b.1).abs() < EPS
}

fn cross2(a: P2, b: P2) -> f64 {
    a.0 * b.1 - a.1 * b.0
}

/// Segment intersection: Some(Some(point)) = touching/proper point,
/// Some(None) = collinear overlap, None = no intersection. (JS `Kc`.)
fn intersect(s: (P2, P2), f: (P2, P2)) -> Option<Option<P2>> {
    let b = s.0;
    let r = f.0;
    let v = (s.1 .0 - b.0, s.1 .1 - b.1);
    let w = (f.1 .0 - r.0, f.1 .1 - r.1);
    let m = cross2(v, w);
    let z = (r.0 - b.0, r.1 - b.1);
    if m.abs() < EPS {
        if cross2(z, v).abs() >= EPS {
            return None; // parallel, not collinear
        }
        let axis = usize::from(v.0.abs() < EPS); // parametrize on the changing axis
        let vb = if axis == 0 { v.0 } else { v.1 };
        if vb.abs() < EPS {
            return None;
        }
        let rb = if axis == 0 { r.0 - b.0 } else { r.1 - b.1 };
        let fb = if axis == 0 { f.1 .0 - b.0 } else { f.1 .1 - b.1 };
        let (g, h) = (rb / vb, fb / vb);
        let (lo, hi) = (g.min(h), g.max(h));
        let ce = lo.max(0.0);
        if ce > hi + EPS {
            None
        } else if (ce - hi).abs() < EPS {
            Some(Some((b.0 + ce * v.0, b.1 + ce * v.1)))
        } else {
            Some(None) // collinear overlap
        }
    } else {
        let hh = cross2(z, w) / m;
        let jj = cross2(z, v) / m;
        if (-EPS..=1.0 + EPS).contains(&hh) && (-EPS..=1.0 + EPS).contains(&jj) {
            Some(Some((b.0 + hh * v.0, b.1 + hh * v.1)))
        } else {
            None
        }
    }
}

fn intern(p: P2, nodes: &mut Vec<P2>, index: &mut HashMap<(i64, i64), usize>) -> usize {
    let k = key(p);
    if let Some(&i) = index.get(&k) {
        return i;
    }
    nodes.push(p);
    index.insert(k, nodes.len() - 1);
    nodes.len() - 1
}

/// The planar graph of `segments`: split at intersection points, adjacency
/// sets. (JS `A0`.)
fn build_graph(segments: &[(P2, P2)]) -> (Vec<P2>, Vec<BTreeSet<usize>>) {
    let mut pts_lists: Vec<Vec<P2>> = segments.iter().map(|s| vec![s.0, s.1]).collect();
    for i in 0..segments.len() {
        for j in i + 1..segments.len() {
            if let Some(Some(pt)) = intersect(segments[i], segments[j]) {
                pts_lists[i].push(pt);
                pts_lists[j].push(pt);
            }
        }
    }
    let mut nodes: Vec<P2> = Vec::new();
    let mut index: HashMap<(i64, i64), usize> = HashMap::new();
    let mut adj: Vec<BTreeSet<usize>> = Vec::new();
    for (si, list) in pts_lists.iter().enumerate() {
        let (from, to) = segments[si];
        let (dx, dy) = (to.0 - from.0, to.1 - from.1);
        let mut sorted = list.clone();
        if dx.abs() >= EPS {
            sorted.sort_by(|a, b| ((a.0 - from.0) / dx).total_cmp(&((b.0 - from.0) / dx)));
        } else {
            sorted.sort_by(|a, b| ((a.1 - from.1) / dy).total_cmp(&((b.1 - from.1) / dy)));
        }
        let mut prev: Option<usize> = None;
        for p in sorted {
            let i = intern(p, &mut nodes, &mut index);
            if adj.len() < nodes.len() {
                adj.resize(nodes.len(), BTreeSet::new());
            }
            if let Some(pv) = prev {
                if pv != i {
                    adj[pv].insert(i);
                    adj[i].insert(pv);
                }
            }
            prev = Some(i);
        }
    }
    (nodes, adj)
}

/// Signed area of a simple polygon (JS `C0`).
fn poly_area2(poly: &[P2]) -> f64 {
    poly.iter()
        .enumerate()
        .map(|(i, p)| {
            let q = poly[(i + 1) % poly.len()];
            p.0 * q.1 - q.0 * p.1
        })
        .sum::<f64>()
        / 2.0
}

/// Faces of the planar graph via directed-edge outer walks with positive
/// signed area. (JS `_0`.)
fn faces(nodes: &[P2], adj: &[BTreeSet<usize>]) -> Vec<Vec<P2>> {
    let nbrs: Vec<Vec<usize>> = adj
        .iter()
        .enumerate()
        .map(|(i, set)| {
            let mut v: Vec<usize> = set.iter().copied().collect();
            let p = nodes[i];
            v.sort_by(|&a, &b| {
                let aa = (nodes[a].1 - p.1).atan2(nodes[a].0 - p.0);
                let bb = (nodes[b].1 - p.1).atan2(nodes[b].0 - p.0);
                aa.total_cmp(&bb)
            });
            v
        })
        .collect();
    let mut visited: HashSet<(usize, usize)> = HashSet::new();
    let mut out: Vec<Vec<P2>> = Vec::new();
    for a in 0..nodes.len() {
        for &z in &nbrs[a] {
            let h = (a, z);
            if visited.contains(&h) {
                continue;
            }
            let (mut j, mut y) = (a, z);
            let mut poly: Vec<P2> = Vec::new();
            while !visited.contains(&(j, y)) {
                visited.insert((j, y));
                poly.push(nodes[j]);
                let list = &nbrs[y];
                let Some(ce) = list.iter().position(|&x| x == j) else { break };
                let e = list[(ce + list.len() - 1) % list.len()];
                j = y;
                y = e;
            }
            if (j, y) == h && poly_area2(&poly) > EPS {
                out.push(poly);
            }
        }
    }
    out
}

/// Point in a simple face polygon: on the boundary = false, else even-odd
/// ray cast. (JS `O0`/`bi`.)
fn on_segment(pt: P2, a: P2, b: P2) -> bool {
    let r = (pt.0 - a.0, pt.1 - a.1);
    let s = (b.0 - a.0, b.1 - a.1);
    cross2(r, s).abs() < EPS
        && pt.0 >= a.0.min(b.0) - EPS
        && pt.0 <= a.0.max(b.0) + EPS
        && pt.1 >= a.1.min(b.1) - EPS
        && pt.1 <= a.1.max(b.1) + EPS
}

fn point_in_face(pt: P2, poly: &[P2]) -> bool {
    let mut inside = false;
    for i in 0..poly.len() {
        let v = poly[i];
        let a = poly[(i + 1) % poly.len()];
        if on_segment(pt, v, a) {
            return false;
        }
        if (v.1 > pt.1) != (a.1 > pt.1) {
            let x = v.0 + (pt.1 - v.1) * (a.0 - v.0) / (a.1 - v.1);
            if x > pt.0 {
                inside = !inside;
            }
        }
    }
    inside
}

/// Territories: the union region of `segments` — faces per connected
/// component, dropping components nested inside another component's face.
/// (JS `nv`.) This is exactly what the engine's `area()` sums.
fn territories(segments: &[(P2, P2)]) -> Vec<Vec<P2>> {
    let n = segments.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut Vec<usize>, mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    for i in 0..n {
        for j in i + 1..n {
            if intersect(segments[i], segments[j]).is_some() {
                let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
                if ri != rj {
                    parent[rj] = ri;
                }
            }
        }
    }
    let mut comps: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n {
        comps.entry(find(&mut parent, i)).or_default().push(i);
    }
    let mut roots: Vec<usize> = comps.keys().copied().collect();
    roots.sort();
    let mut entries: Vec<(P2, Vec<Vec<P2>>)> = Vec::new();
    for root in roots {
        let segs = &comps[&root];
        let probe = segments[segs[0]].0;
        let sub: Vec<(P2, P2)> = segs.iter().map(|&i| segments[i]).collect();
        let (nodes, adj) = build_graph(&sub);
        let f = faces(&nodes, &adj);
        if !f.is_empty() {
            entries.push((probe, f));
        }
    }
    let mut out: Vec<Vec<P2>> = Vec::new();
    for (zi, (probe, f)) in entries.iter().enumerate() {
        let nested = entries.iter().enumerate().any(|(zj, (_, g))| {
            zj != zi && g.iter().any(|face| point_in_face(*probe, face))
        });
        if !nested {
            out.extend(f.iter().cloned());
        }
    }
    out
}

fn territory_area(territories: &[Vec<P2>]) -> f64 {
    territories.iter().map(|f| poly_area2(f).abs()).sum()
}

fn phase(n: usize) -> &'static str {
    if n <= 30 {
        "early"
    } else if n <= 80 {
        "mid"
    } else {
        "late"
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    assert!(!args.is_empty(), "usage: probe_angel <path-or-dir>...");
    let files = game_files(&args);

    let mut totals = Totals::default();
    let mut per_opp: BTreeMap<String, OpponentCircles> = BTreeMap::new();

    for file in &files {
        let Some(recorded) = gamejson::load(file) else {
            println!("skip (neither format): {}", file.display());
            continue;
        };
        let lines = census_game(&recorded, &mut totals, per_opp.entry(recorded.opponent.clone()).or_default());
        for line in &lines {
            println!("{line}");
        }
    }

    println!("\n=== ANGEL-CIRCLE census ===");
    println!(
        "games {}, actions {} | opponent closes {} (gaining {}) | circles {} in {} games | circles/closing-gaining {:.1}%",
        totals.games,
        totals.actions,
        totals.closes_opp,
        totals.closes_gaining,
        totals.circles,
        totals.games_with_circle,
        pct(totals.circles, totals.closes_gaining)
    );
    println!(
        "shape by gained area: small(<8) {} mid(8-30) {} big(>=30) {} | by phase: early {} mid {} late {} | our nodes inside: 1 {} 2-4 {} 5+ {}",
        totals.shape_small, totals.shape_mid, totals.shape_big,
        totals.early, totals.mid, totals.late,
        totals.inside_1, totals.inside_2_4, totals.inside_5plus
    );
    println!(
        "wall thickness at circle: avg deg>=2 (shared-node) count {:.1}, avg opponent edges {:.1}",
        totals.avg_thickness(),
        avg(totals.edges_sum, totals.circles)
    );
    println!(
        "what breaks it: circle edges cuttable at first our-turn {}/{} circles (avg {:.1} cuttable edges, max destroyed {:.1}) | we cut a circle edge in {} circles | bank lost (opp area fell back) in {} circles",
        totals.cuttable_circles,
        totals.checked_circles,
        avg(totals.cuttable_edges, totals.checked_circles),
        totals.max_destroyed_seen,
        totals.we_cut_circle,
        totals.bank_lost
    );
    println!(
        "outcomes: our win rate in circled games {}/{} = {:.1}% vs uncircled {}/{} = {:.1}%",
        totals.wins_circled,
        totals.games_with_circle,
        pct(totals.wins_circled, totals.games_with_circle),
        totals.wins_uncircled,
        totals.games - totals.games_with_circle,
        pct(totals.wins_uncircled, totals.games - totals.games_with_circle),
    );
    println!(
        "port validation (territory port vs engine area): {} checks, {} mismatches",
        totals.port_checks, totals.port_fail
    );
    println!("\n-- per opponent (games, circles, circles/game, cuttable-at-turn, we-cut, bank-lost) --");
    for (name, row) in &per_opp {
        println!(
            "{name:14} games {:2} circles {:3} ({:.2}/game) | cuttable {}/{} | we-cut {} | bank-lost {}",
            row.games, row.circles, row.circles as f64 / row.games as f64,
            row.cuttable, row.checked, row.we_cut, row.bank_lost
        );
    }
    println!("\nNOTE: no AngelWASM games available yet (v8 evals queued, not landed) —");
    println!("run this probe on them when they land for the AngelWASM-specific census.");
}

#[derive(Default)]
struct Totals {
    games: usize,
    actions: usize,
    closes_opp: usize,
    closes_gaining: usize,
    circles: usize,
    games_with_circle: usize,
    shape_small: usize,
    shape_mid: usize,
    shape_big: usize,
    early: usize,
    mid: usize,
    late: usize,
    inside_1: usize,
    inside_2_4: usize,
    inside_5plus: usize,
    thickness_sum: usize,
    edges_sum: usize,
    checked_circles: usize,
    cuttable_circles: usize,
    cuttable_edges: usize,
    max_destroyed_seen: f64,
    we_cut_circle: usize,
    bank_lost: usize,
    wins_circled: usize,
    wins_uncircled: usize,
    port_checks: usize,
    port_fail: usize,
}

impl Totals {
    fn avg_thickness(&self) -> f64 {
        if self.circles == 0 { 0.0 } else { self.thickness_sum as f64 / self.circles as f64 }
    }
}

#[derive(Default)]
struct OpponentCircles {
    games: usize,
    circles: usize,
    checked: usize,
    cuttable: usize,
    we_cut: usize,
    bank_lost: usize,
}

struct PendingCircle {
    /// Opponent edges at circle time that lie on the containing face's boundary.
    circle_edges: Vec<(P2, P2)>,
    pre_area: f64,
    /// Whether the first-our-turn cuttability check has run.
    checked: bool,
    cuttable_edges: usize,
    max_destroyed: f64,
    we_cut: bool,
    bank_lost: bool,
}

/// The edge is part of the circle's wall: some face-boundary stretch (v, a)
/// lies along the edge. T-junction walls are made of edge parts — an edge
/// crossing the boundary at an interior point and extending beyond still
/// carries the wall stretch, so "both endpoints on the boundary" is too
/// strict (measured: circle@44, R-chall-neck-botIsred — face of 5 pts, 5
/// edges with an endpoint on it, 0 with both).
fn edge_on_boundary(e: (P2, P2), face: &[P2]) -> bool {
    (0..face.len()).any(|i| {
        let v = face[i];
        let a = face[(i + 1) % face.len()];
        on_segment(v, e.0, e.1) && on_segment(a, e.0, e.1)
    })
}

/// The biggest enemy-area destruction among legal breaks, and which circle
/// edges a legal cut can break. `None` when the mover has no legal moves.
fn cuttable_check(
    pos: &meridian_engine::Position,
    mover: Player,
    circle_edges: &[(P2, P2)],
) -> Option<(f64, usize)> {
    let opp = mover.opponent();
    let opp_area = pos.area(opp).to_f64();
    let mut best = 0.0f64;
    let mut cuttable: BTreeSet<usize> = BTreeSet::new();
    for mv in pos.legal_moves().iter() {
        let mut after = pos.clone();
        let oc = after.apply_unchecked(mv);
        if let Some(broken) = oc.broken {
            let (a, b) = broken.endpoints();
            let (pa, pb) = (pt2(a), pt2(b));
            for (i, ce) in circle_edges.iter().enumerate() {
                let (cs, cf) = *ce;
                let touches = same(cs, pa) && same(cf, pb) || same(cs, pb) && same(cf, pa);
                if touches {
                    cuttable.insert(i);
                }
            }
            best = best.max(opp_area - after.area(opp).to_f64());
        }
    }
    Some((best, cuttable.len()))
}

fn census_game(recorded: &RecordedGame, totals: &mut Totals, row: &mut OpponentCircles) -> Vec<String> {
    let me = recorded.me;
    let opp = me.opponent();
    let mut game = Game::new();
    let mut lines: Vec<String> = Vec::new();
    let mut pending: Vec<PendingCircle> = Vec::new();
    let mut circles_this_game = 0usize;

    for (k, &(mv, mover)) in recorded.actions.iter().enumerate() {
        let n = k + 1;
        let before = game.position().clone();
        let opp_pre = before.area(opp).to_f64();
        let oc = game.play(mv).expect("recorded move is legal");
        let after = game.position();

        // Circle detection: opponent close that gains real area.
        if mover == opp && oc.kind == MoveKind::Connect {
            totals.closes_opp += 1;
            let gained = after.area(opp).to_f64() - opp_pre;
            if gained > 0.5 {
                totals.closes_gaining += 1;
                let opp_segs: Vec<(P2, P2)> =
                    after.edges(opp).iter().map(|e| e.endpoints()).map(|(a, b)| (pt2(a), pt2(b))).collect();
                let terr = territories(&opp_segs);
                let our_nodes: Vec<P2> = after.nodes(me).iter().map(pt2).collect();
                for face in &terr {
                    let inside: Vec<P2> =
                        our_nodes.iter().filter(|p| point_in_face(**p, face)).copied().collect();
                    if inside.is_empty() {
                        continue;
                    }
                    // CIRCLE event.
                    circles_this_game += 1;
                    totals.circles += 1;
                    let thickness = {
                        let mut degree: HashMap<(i64, i64), u32> = HashMap::new();
                        for seg in &opp_segs {
                            for p in [seg.0, seg.1] {
                                *degree.entry(key(p)).or_insert(0) += 1;
                            }
                        }
                        degree.values().filter(|d| **d >= 2).count()
                    };
                    totals.thickness_sum += thickness;
                    totals.edges_sum += opp_segs.len();
                    match inside.len() {
                        1 => totals.inside_1 += 1,
                        2..=4 => totals.inside_2_4 += 1,
                        _ => totals.inside_5plus += 1,
                    }
                    match gained {
                        g if g < 8.0 => totals.shape_small += 1,
                        g if g < 30.0 => totals.shape_mid += 1,
                        _ => totals.shape_big += 1,
                    }
                    match phase(n) {
                        "early" => totals.early += 1,
                        "mid" => totals.mid += 1,
                        _ => totals.late += 1,
                    }
                    pending.push(PendingCircle {
                        circle_edges: opp_segs
                            .iter()
                            .filter(|e| edge_on_boundary(**e, face))
                            .copied()
                            .collect(),
                        pre_area: opp_pre,
                        checked: false,
                        cuttable_edges: 0,
                        max_destroyed: 0.0,
                        we_cut: false,
                        bank_lost: false,
                    });
                    if pending.last().unwrap().circle_edges.is_empty() {
                        // Diagnostic: the boundary-edge mapping found nothing.
                        let on = |p: P2| {
                            (0..face.len()).any(|i| {
                                let v = face[i];
                                let a = face[(i + 1) % face.len()];
                                on_segment(p, v, a)
                            })
                        };
                        lines.push(format!(
                            "{}:   circle@{n} DIAG face poly {} pts, ends on boundary: {} of {} edges",
                            recorded.name,
                            face.len(),
                            opp_segs.iter().filter(|e| on(e.0) || on(e.1)).count(),
                            opp_segs.len()
                        ));
                    }
                    lines.push(format!(
                        "{}: CIRCLE@{:3} gained {gained:.1} | our nodes inside {}/{} | opp edges {} (deg>=2: {}) | phase {}",
                        recorded.name,
                        n,
                        inside.len(),
                        our_nodes.len(),
                        opp_segs.len(),
                        thickness,
                        phase(n)
                    ));
                }
            }
        }

        // Our cuts against circle edges + bank-lost tracking.
        if mover == me {
            if let Some(broken) = oc.broken {
                let (a, b) = broken.endpoints();
                let (pa, pb) = (pt2(a), pt2(b));
                for p in pending.iter_mut() {
                    if p.circle_edges.iter().any(|ce| {
                        let (cs, cf) = *ce;
                        same(cs, pa) && same(cf, pb) || same(cs, pb) && same(cf, pa)
                    }) {
                        p.we_cut = true;
                    }
                }
            }
        } else {
            let opp_now = after.area(opp).to_f64();
            for p in pending.iter_mut() {
                if opp_now < p.pre_area - 0.5 {
                    p.bank_lost = true;
                }
            }
        }

        // First-our-turn cuttability check for each pending circle.
        if mover == me {
            for p in pending.iter_mut() {
                if !p.checked {
                    if let Some((max_d, cuttable)) = cuttable_check(&after, me, &p.circle_edges) {
                        p.checked = true;
                        p.cuttable_edges = cuttable;
                        p.max_destroyed = max_d;
                        totals.checked_circles += 1;
                        if cuttable > 0 {
                            totals.cuttable_circles += 1;
                            totals.cuttable_edges += cuttable;
                        }
                        totals.max_destroyed_seen = totals.max_destroyed_seen.max(max_d);
                        if cuttable == 0 {
                            lines.push(format!(
                                "{}:   circle@{n} UNBREAKABLE at first our-turn (0 of {} boundary edges cuttable; max legal break destroyed {max_d:.1})",
                                recorded.name,
                                p.circle_edges.len()
                            ));
                        }
                    }
                }
            }
        }
    }

    // Game outcome + port validation.
    let final_pos = game.position();
    let (ours, theirs) = (final_pos.score(me).to_f64(), final_pos.score(opp).to_f64());
    let we_won = ours > theirs;
    totals.games += 1;
    totals.actions += recorded.actions.len();
    if circles_this_game > 0 {
        totals.games_with_circle += 1;
        if we_won {
            totals.wins_circled += 1;
        }
    } else if we_won {
        totals.wins_uncircled += 1;
    }
    for p in &pending {
        if p.we_cut {
            totals.we_cut_circle += 1;
        }
        if p.bank_lost {
            totals.bank_lost += 1;
        }
    }
    row.games += 1;
    row.circles += circles_this_game;
    for p in &pending {
        if p.checked {
            row.checked += 1;
            if p.cuttable_edges > 0 {
                row.cuttable += 1;
            }
        }
        if p.we_cut {
            row.we_cut += 1;
        }
        if p.bank_lost {
            row.bank_lost += 1;
        }
    }

    // Port validation: the territory port must reproduce the engine's area().
    for player in [Player::Blue, Player::Red] {
        let segs: Vec<(P2, P2)> =
            final_pos.edges(player).iter().map(|e| e.endpoints()).map(|(a, b)| (pt2(a), pt2(b))).collect();
        if segs.is_empty() {
            continue;
        }
        let engine_area = final_pos.area(player).to_f64();
        let port_area = territory_area(&territories(&segs));
        totals.port_checks += 1;
        if (engine_area - port_area).abs() > 0.05 {
            totals.port_fail += 1;
            lines.push(format!(
                "{}:   PORT MISMATCH {} engine {engine_area:.2} vs port {port_area:.2}",
                recorded.name,
                if player == Player::Blue { "blue" } else { "red" }
            ));
        }
    }
    lines.push(format!(
        "{}: {} circles | final {ours:.0}-{theirs:.0} {}",
        recorded.name,
        circles_this_game,
        if we_won { "WE WIN" } else { "OPP WINS" }
    ));
    lines
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 { 0.0 } else { n as f64 / d as f64 * 100.0 }
}

fn avg(n: usize, d: usize) -> f64 {
    if d == 0 { 0.0 } else { n as f64 / d as f64 }
}
