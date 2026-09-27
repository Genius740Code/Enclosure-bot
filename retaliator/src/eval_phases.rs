//! Lane B (eval/phases): legal-cuts-only vulnerability pricing.
//!
//! This file is the shipped `search.rs` skeleton (2-ply WIDTH-8, full-horizon
//! valuation, every V4/V5 term) with ONE added term, kept in a separate file
//! because eval lanes add here and the main session merges gated winners into
//! `search.rs` themselves. Copied verbatim, not imported: the skeleton's items
//! are private, and a shared copy would let one drift from the other.
//!
//! **Legal-cuts-only vulnerability (V4d2 field rule).** The static eval counts
//! our area x `HORIZON` events as if it all banks, and the doom discount
//! subtracts the single worst one-action pop. Both miss erosion: several of
//! our edges legally cuttable over consecutive actions drain area that no max
//! ever prices (the 41-cut farming line in `f61a06ec`). The term widens the
//! doom discount's "worst pop" to the total area the enemy can LEGALLY cut
//! from the evaluated position, counted once per distinct edge, in one pass
//! over their legal move set. Legal cuts only: a dense cluster — any edge
//! through it touches 2+ opposing edges, which is illegal — prices zero, and
//! shielded edges are struck out by the movegen itself. No fantasy cuts, no
//! geometric touches, same full-horizon units as the discount it widens.
//!
//! The term is inert while our enclosed area is 0 (nothing to cut), so it
//! cannot distort the opening where the Blue chair lives; it bites from the
//! first closed loop on. `baseline_best_move` is the identical search with
//! the term OFF: the ablation control, which must pick exactly what
//! `search::best_move` picks — probe `probe_b_h2h` checks that position for
//! position before trusting any game.

use std::cmp::Reverse;

use meridian_engine::{Edge, Move, MoveKind, Outcome, Player, Point, Position};

/// Positions searched for a move in a game. Analysis asks for its own number, up to this.
pub const MOVE_BUDGET: usize = 4096;
const SMALLEST_BUDGET: usize = 16;
/// How many first actions get their replies searched.
const WIDTH: usize = 8;
/// Enclosed area counts as if held for at most this many more scoring events.
const HORIZON: f64 = 12.0;
/// What the room a player's nodes span is worth, per unit of area, against area enclosed.
const ROOM_WEIGHT: f64 = 0.4;
/// How much of the beyond-horizon area value counts, in both ranking and final
/// selection. Applies symmetrically to claims and denial (see below).
const HORIZON_WEIGHT: f64 = 0.5;
/// First-action penalty for ending next to an enemy node without cutting anything.
const CONTACT_PENALTY: f64 = 1.0;
/// Penalty for a no-area reinforcement between own nodes far from the enemy.
const DEADWOOD_PENALTY: f64 = 2.0;
/// Beyond this Chebyshev distance to the nearest enemy node, a no-area
/// reinforcement counts as dead wood in the back.
const DEADWOOD_ENEMY_DIST: i8 = 3;
/// Anti-rebuild routing (rival-analysis steal #1): our edges were cut near
/// these points within the last few actions, so re-closing there is farmed.
/// Penalty for a non-breaking first action landing within this radius.
const CUT_RADIUS: i8 = 3;
const REBUILD_PENALTY: f64 = 3.0;
/// Blue's forced first action: D10-F7. Measured over 12 games vs three
/// opponents (v1, v2, scout-class): +20% to +41% margins, 12/12 positive,
/// vs -4% to +14% unforced. The search walks backward (D10-A7) on
/// direction-index ties; the forward diagonal banks early and develops
/// toward the center. Zero risk: at action 1 nothing can make it illegal
/// except a non-standard start, which falls through to search via the
/// legality check below.
fn blue_opener(position: &Position) -> Option<Move> {
    if position.actions_played() != 0 {
        return None;
    }
    let mv = Move::between(
        Point::new(-6, 0).expect("D10 on board"),
        Point::new(-4, -3).expect("F7 on board"),
    )
    .expect("D10-F7 is a king step");
    position.check_move(mv).ok().map(|_| mv)
}

// (CUT_MEMORY is lib.rs's replay-side constant; it is not part of the search
// and is not copied here.)
/// Fresh/shielded-wall avoidance: enemy edges placed last turn can't be
/// cut this turn, so contesting them burns tempo. Penalty for a
/// non-breaking first action landing near them.
const FRESH_RADIUS: i8 = 2;
const FRESH_PENALTY: f64 = 1.5;
/// Do-nothing filter: a first action with no area gain, no break, negligible
/// room growth and nowhere near the enemy does nothing on every axis.
/// Penalize (both kinds) so shuffling ranks below every real option. Safe
/// by symmetry: if ALL moves are dead, all share the penalty and order
/// is preserved.
const IDLE_ROOM: f64 = 0.5;
const IDLE_ENEMY_DIST: i8 = 3;
const IDLE_PENALTY: f64 = 2.0;
/// Two-front play: where the position is hot (many close cross-color node
/// pairs), building huge areas FAR from the fighting is nearly unbreakable —
/// go both ways. Bonus for first actions landing far from all enemy
/// nodes, paid only when heat says the local fight is contested.
const FIGHT_DIST: i8 = 4;
const FIGHT_HEAT: usize = 10;
const REMOTE_DIST: i8 = 5;
const REMOTE_BONUS: f64 = 1.0;
/// Thicket density (the screenshot lesson): walls that share nodes are
/// near-unbreakable (any cut touches 2+) AND bank area. Bonus for a first
/// action landing next to 2+ own nodes. Deadwood still bans far no-gain
/// Connects, so this only ever rewards thickets that gain or reach.
const DENSE_DIST: i8 = 1;
const DENSE_COUNT: u32 = 2;
const DENSE_BONUS: f64 = 1.0;
/// Doom discount: area we hold that the enemy pops in one action is false
/// credit in the static eval (it counts area x 12 events as if it banks).
/// For each candidate, the worst one-action pop of our area, times the
/// capped horizon, is subtracted at selection. Doomed megaloops net to ~0,
/// so the bot builds split/remote/dense ground instead of one big grazeable
/// balloon. Computed only where the enemy is to move (their legal set).
/// With [`VULN_W`] on, the "worst pop" widens to the total legally-cuttable
/// area (see below); this constant keeps the scale either way.
const DOOM_W: f64 = 1.0;
/// Capture exposure: nodes held by a single edge can be captured outright.
/// Counts ours vs theirs; each such node is a discrete, hard-to-reverse
/// swing, so it prices higher than a generic edge.
const CAPTURE_W: f64 = 3.0;
/// Patience (rival-analysis steal #2): in the opening, don't snatch tiny
/// loops; wall first, close big later. Penalty for a first-action close
/// gaining less than this, while fewer than this many actions are played.
/// V5 ablation 2026-09-27: does this misfire on forced openings as Blue
/// (delaying closes v1 snatches)? 0.0 = off. RESULT: off is byte-identical
/// across all gates (v3all 4/10+7/10, league -20.0%, gauge 6/6) — the term
/// never flips a pick in any measured line. Kept at 2.0 (harmless).
const PATIENCE_MAX_GAIN: f64 = 2.0;
const PATIENCE_WINDOW: u8 = 12;
const PATIENCE_PENALTY: f64 = 2.0;

/// Legal-cuts-only vulnerability (this file's term, V4d2 field rule): the
/// doom discount subtracts the worst one-action pop; the term adds the
/// EROSION GAP — the total area the enemy can legally cut from the evaluated
/// position BEYOND that worst pop, per distinct edge, from their legal move
/// set. The max prices only the single worst pop; erosion across several
/// cuttable edges over consecutive actions (the 41-cut farming line) is
/// unpriced by it. Zero when only one edge is cuttable, so the term never
/// disturbs single-cut positions. Units match the discount they join:
/// area x min(events, 12), full-horizon points — no invented units.
/// Computed in the same single pass as the worst pop, so the term adds no
/// search cost. Set 0.0 to turn the term off (the ablation control).
const VULN_W: f64 = 1.0;

pub fn best_move(position: &Position) -> Option<Move> {
    best_move_with_avoid(position, &[])
}

/// Like [`best_move`], but steering clear of `avoid`: points near our loops
/// the enemy recently cut. Rebuilding there is how rebuilder-farming works
/// (41 cuts in one game); routing elsewhere denies the repeat cut. Same
/// contract as `search::best_move_with_avoid`, so a gated merge into lib.rs
/// keeps the anti-rebuild routing in `replay` wired.
pub fn best_move_with_avoid(position: &Position, avoid: &[Point]) -> Option<Move> {
    best_move_features(position, MOVE_BUDGET, avoid, true)
}

/// The identical search with the legal-cuts-only vulnerability term OFF: the
/// ablation control. It must pick exactly what `search::best_move` picks —
/// `probe_b_h2h` checks that position for position before trusting any game.
/// (Not every probe uses it; the gate keeps per-example dead-code quiet.)
#[allow(dead_code)]
pub fn baseline_best_move(position: &Position) -> Option<Move> {
    best_move_features(position, MOVE_BUDGET, &[], false)
}

/// The skeleton's selection loop, with the vulnerability term gated. The
/// `Analysis`/`Candidate` reporting surface is not copied: probes read the
/// pick, and the main session merges the term itself.
fn best_move_features(
    position: &Position,
    budget: usize,
    avoid: &[Point],
    vuln: bool,
) -> Option<Move> {
    if let Some(open) = blue_opener(position) {
        return Some(open);
    }
    let mover = position.to_move();
    let opp = mover.opponent();
    let budget = budget.clamp(SMALLEST_BUDGET, MOVE_BUDGET);
    let first_actions = ranked(position, budget / 2, true, avoid);
    let nodes = first_actions.len();
    let width = first_actions.len().min(WIDTH);
    let reply_budget = (budget - nodes) / width.max(1);

    // (adjusted score for ordering, the move it belongs to).
    let mut scored: Vec<(f64, Move)> = Vec::new();
    for (mv, after, _, _) in first_actions.into_iter().take(width) {
        let replies = ranked(&after, reply_budget, false, &[]);
        let mut pv = vec![mv];
        let mut evaluation = value(&after);
        if let Some((reply, _, reply_value, _)) = replies.first() {
            evaluation = *reply_value;
            pv.push(*reply);
        }
        let end = position_after_pv(position, &pv);
        let mut adjusted = sign(mover) * evaluation + horizon_extension(position, &end, mover);
        // Poisoned ground counts at SELECTION, not just ranking: a line that
        // rebuilds where we were just cut (or contests a fresh wall) loses
        // here even if its honest eval is best — the eval can't see the
        // re-cut coming, but the cut history can.
        {
            let hz_sel = f64::from(position.scoring_events_left()).min(HORIZON);
            let mut probe = position.clone();
            let oc = probe.apply_unchecked(mv);
            if oc.broken.is_none() {
                let tgt = mv.target().expect("legal moves end on the board");
                if near_points(avoid.iter().copied(), tgt, CUT_RADIUS) {
                    adjusted -= REBUILD_PENALTY * hz_sel;
                }
                if near_fresh_enemy(position, tgt) {
                    adjusted -= FRESH_PENALTY * hz_sel;
                }
            }
        }
        // Doom discount (see const docs): the proven worst-one-action-pop
        // term, then the term this file adds — the erosion gap, the area
        // the enemy can LEGALLY cut BEYOND that worst pop, per distinct
        // edge. The max prices only the single worst pop, so erosion
        // across several cuttable edges (the 41-cut farming line) is
        // unpriced by it. Legal cuts only, from the enemy's legal move
        // set, in the same single pass the max already runs.
        {
            let doom_at = if end.to_move() == opp {
                Some(&end)
            } else if after.to_move() == opp {
                Some(&after)
            } else {
                None
            };
            if let Some(pos) = doom_at {
                let hz_doom = f64::from(pos.scoring_events_left()).min(HORIZON);
                let (worst, cuttable) = pop_and_cuttable(pos, mover);
                adjusted -= DOOM_W * worst * hz_doom;
                // The term: the erosion gap — area the enemy can LEGALLY cut
                // BEYOND the single worst pop, which no max ever prices.
                // Zero when only one edge is cuttable, so the term bites
                // exactly on multi-edge erosion (the farming line). Same
                // full-horizon units, same single pass as the max.
                if vuln {
                    adjusted -= VULN_W * (cuttable - worst) * hz_doom;
                }
            }
        }
        scored.push((adjusted, mv));
    }
    scored.sort_by(|a, b| {
        b.0.total_cmp(&a.0).then(a.1.index().cmp(&b.1.index()))
    });
    scored.first().map(|(_, mv)| *mv)
}

/// Replays a principal variation from `position`.
fn position_after_pv(position: &Position, pv: &[Move]) -> Position {
    let mut pos = position.clone();
    for &mv in pv {
        pos.apply_unchecked(mv);
    }
    pos
}

/// The beyond-horizon part of an area swing, mover-relative: own area gained
/// plus enemy area destroyed along the line, times the scoring events past
/// the horizon. The static evaluation already counts both up to the horizon;
/// this is the rest, kept symmetric so a break must beat the available
/// claims on the merits instead of winning by default.
fn horizon_extension(before: &Position, end: &Position, mover: Player) -> f64 {
    let gained = (end.area(mover).to_f64() - before.area(mover).to_f64()).max(0.0);
    let destroyed =
        (before.area(mover.opponent()).to_f64() - end.area(mover.opponent()).to_f64()).max(0.0);
    let events = f64::from(end.scoring_events_left());
    (gained + destroyed) * (events - events.min(HORIZON)) * HORIZON_WEIGHT
}

struct Ranked {
    mv: Move,
    after: Position,
    value: f64,
    priority: f64,
}

/// Blue's lead as Scout sees it. Positive favours Blue.
pub fn evaluate(position: &Position) -> f64 {
    let events_left = f64::from(position.scoring_events_left());
    let worth = |player| {
        position.score(player).to_f64()
            + position.area(player).to_f64() * events_left.min(HORIZON)
            + room(position, player) * ROOM_WEIGHT * events_left.min(1.0)
    };
    worth(Player::Blue) - worth(Player::Red)
        + CAPTURE_W
            * (one_edge_nodes(position, Player::Red) as f64
                - one_edge_nodes(position, Player::Blue) as f64)
}

/// The worst one-action pop of `victim`'s area, and the total area the enemy
/// can legally cut from `pos` (enemy to move), counted once per distinct cut
/// edge. One pass over the enemy's legal set: legal cuts only, so a dense
/// cluster — any edge through it touches 2+ opposing edges, which is illegal
/// (one cut per move max) — and shielded edges price zero. No fantasy cuts,
/// no geometric touches (V4d2). The worst pop is exactly the shipped
/// `max_pop`; the cuttable sum is the term's widening of it.
fn pop_and_cuttable(pos: &Position, victim: Player) -> (f64, f64) {
    let held = pos.area(victim).to_f64();
    let mut seen: Vec<Edge> = Vec::new();
    let mut worst: f64 = 0.0;
    let mut cuttable: f64 = 0.0;
    for mv in pos.legal_moves().iter() {
        let mut next = pos.clone();
        let outcome = next.apply_unchecked(mv);
        let loss = held - next.area(victim).to_f64();
        worst = worst.max(loss);
        if let Some(cut) = outcome.broken {
            if !seen.contains(&cut) {
                seen.push(cut);
                cuttable += loss;
            }
        }
    }
    (worst.max(0.0), cuttable.max(0.0))
}

/// How many of `player`'s nodes hang by a single edge (capturable).
fn one_edge_nodes(position: &Position, player: Player) -> u32 {
    use std::collections::HashMap;
    let mut degree: HashMap<usize, u32> = HashMap::new();
    for edge in position.edges(player).iter() {
        for end in [edge.origin(), edge.far()] {
            *degree.entry(end.index()).or_insert(0) += 1;
        }
    }
    position.nodes(player).iter().filter(|n| degree.get(&n.index()) == Some(&1)).count() as u32
}

/// [`evaluate`], except that a drawn game is worth 0.
fn value(position: &Position) -> f64 {
    let drawn = matches!(position.outcome_given(&position.legal_moves()), Some(Outcome::Draw(_)));
    if drawn { 0.0 } else { evaluate(position) }
}

/// The mover's actions, best first, with the position after each and its value. Only the first
/// `budget` of them, longest edges first, are tried.
fn ranked(
    position: &Position,
    budget: usize,
    first: bool,
    avoid: &[Point],
) -> Vec<(Move, Position, f64, f64)> {
    let mover = position.to_move();
    let opp = mover.opponent();
    let room_before = room(position, mover);
    let heat = fight_heat(position, mover, opp);
    let mut moves: Vec<Move> =
        position.legal_moves().iter().filter(|&mv| !repeats_a_connection(position, mv)).collect();
    moves.sort_by_key(|mv| (Reverse(length(*mv)), mv.index()));

    let mut tried: Vec<Ranked> = moves
        .into_iter()
        .take(budget)
        .map(|mv| {
            let mut after = position.clone();
            let outcome = after.apply_unchecked(mv);
            let value = value(&after);
            let mut priority = sign(mover) * value;
            if first && after.to_move() == mover && !after.is_finished() {
                priority += loop_bonus(position, mv, &after, room_before);
            }
            // Fix 1: full-horizon claims and denial, in ranking as well as selection.
            priority += horizon_extension(position, &after, mover);
            let target = mv.target().expect("legal moves end on the board");
            let own_gain = after.area(mover).to_f64() - position.area(mover).to_f64();
            // All priorities run in full-horizon points (area x events), so
            // fixed penalties must scale the same way or they never fire.
            let hz = f64::from(position.scoring_events_left()).min(HORIZON);
            if first {
                // Two-front play: hot position + far target = safe banking.
                if heat >= FIGHT_HEAT
                    && outcome.broken.is_none()
                    && !near_enemy_node(position, opp, target, REMOTE_DIST)
                {
                    priority += REMOTE_BONUS * hz;
                }
                // Fresh/shielded walls: don't fight what can't be cut yet.
                if outcome.broken.is_none()
                    && near_fresh_enemy(position, target)
                {
                    priority -= FRESH_PENALTY * hz;
                }
                // Thicket density: land next to 2+ own nodes.
                if near_own_count(position, mover, target) >= DENSE_COUNT {
                    priority += DENSE_BONUS * hz;
                }
                // Do-nothing filter.
                if outcome.broken.is_none()
                    && own_gain <= 0.0
                    && room(&after, mover) - room_before < IDLE_ROOM
                    && !near_enemy_node(position, opp, target, IDLE_ENEMY_DIST)
                {
                    priority -= IDLE_PENALTY * hz;
                }
                // Anti-rebuild routing: our loops were cut near these points
                // recently; re-closing there gets farmed. Counter-cutting
                // (breaking something) is exempt. (Rival-analysis steal #1.)
                if outcome.broken.is_none() && near_points(avoid.iter().copied(), target, CUT_RADIUS) {
                    priority -= REBUILD_PENALTY * hz;
                }
                // Patience: don't snatch tiny loops in the opening while the
                // board is wide open; set up bigger closes instead (GB delays
                // its first close to action ~13 and banks ~10/loop).
                if outcome.kind == MoveKind::Connect
                    && own_gain < PATIENCE_MAX_GAIN
                    && position.actions_played() < PATIENCE_WINDOW
                {
                    priority -= PATIENCE_PENALTY * hz;
                }
                // Fix 2: don't graze the enemy frontier for free.
                if outcome.broken.is_none()
                    && near_enemy_node(position, opp, target, 1)
                {
                    priority -= CONTACT_PENALTY * hz;
                }
                // Fix 3: don't reinforce the back; expand instead. A Connect
                // that adds no area far from the enemy burns tempo: the safe
                // loop already banks, and redundancy never survives a cut.
                if outcome.kind == MoveKind::Connect
                    && own_gain <= 0.0
                    && !near_enemy_node(position, opp, target, DEADWOOD_ENEMY_DIST)
                {
                    priority -= DEADWOOD_PENALTY * hz;
                }
            }
            Ranked { mv, after, value, priority }
        })
        .collect();
    tried.sort_by(|a, b| b.priority.total_cmp(&a.priority).then(a.mv.index().cmp(&b.mv.index())));
    tried.into_iter().map(|r| (r.mv, r.after, r.value, r.priority)).collect()
}

/// Whether `target` is within Chebyshev `dist` of any of the given points.
fn near_points(mut points: impl Iterator<Item = Point>, target: Point, dist: i8) -> bool {
    points.any(|p| (p.x() - target.x()).abs() <= dist && (p.y() - target.y()).abs() <= dist)
}

/// How many close opposing-node pairs the position holds: the fight heat.
/// Computed once per ranking (not per candidate).
fn fight_heat(position: &Position, mover: Player, opp: Player) -> usize {
    let own: Vec<Point> = position.nodes(mover).iter().collect();
    let foe: Vec<Point> = position.nodes(opp).iter().collect();
    let mut heat = 0;
    for a in &own {
        for b in &foe {
            if (a.x() - b.x()).abs() <= FIGHT_DIST && (a.y() - b.y()).abs() <= FIGHT_DIST {
                heat += 1;
            }
        }
    }
    heat
}

/// Whether `target` is near enemy edges placed last turn (shielded: uncuttable
/// now, so contesting them is burned tempo).
fn near_fresh_enemy(position: &Position, target: Point) -> bool {
    position.shielded_edges().any(|edge| {
        near_points([edge.origin(), edge.far()].into_iter(), target, FRESH_RADIUS)
    })
}

/// How many of `player`'s nodes are within Chebyshev `DENSE_DIST` of `target`.
fn near_own_count(position: &Position, player: Player, target: Point) -> u32 {
    position
        .nodes(player)
        .iter()
        .filter(|node| {
            (node.x() - target.x()).abs() <= DENSE_DIST
                && (node.y() - target.y()).abs() <= DENSE_DIST
        })
        .count() as u32
}

/// Whether `target` is within Chebyshev `dist` of any of `player`'s nodes.
fn near_enemy_node(position: &Position, player: Player, target: Point, dist: i8) -> bool {
    near_points(position.nodes(player).iter(), target, dist)
}

/// A connection between two of the mover's nodes is listed from both ends. This is the second.
fn repeats_a_connection(position: &Position, mv: Move) -> bool {
    let target = mv.target().expect("legal moves end on the board");
    position.nodes(position.to_move()).contains(target) && target < mv.source
}

fn length(mv: Move) -> i8 {
    mv.direction.dx().abs().max(mv.direction.dy().abs())
}

/// For a first action the second can close into a triangle: the room it adds, up to that
/// triangle's area, for the scoring events left. It keeps such plans ahead of long edges that
/// cannot be closed in time.
fn loop_bonus(position: &Position, mv: Move, after: &Position, room_before: f64) -> f64 {
    let mover = position.to_move();
    let target = mv.target().expect("legal moves end on the board");
    let mut triangle: f64 = 0.0;
    for edge in position.edges(mover).iter().filter(|edge| edge.has_endpoint(mv.source)) {
        let (origin, far) = edge.endpoints();
        let third = if origin == mv.source { far } else { origin };
        let closes = Move::between(target, third).is_some_and(|closing| after.check_move(closing).is_ok());
        if closes {
            triangle = triangle.max(f64::from(cross(mv.source, target, third).abs()) * 0.5);
        }
    }
    let added_room = (room(after, mover) - room_before).max(0.0).min(triangle);
    added_room * f64::from(after.scoring_events_left().min(HORIZON as u8))
}

/// The area of the convex hull of a player's nodes: room to grow into.
fn room(position: &Position, player: Player) -> f64 {
    let mut points: Vec<Point> = position.nodes(player).iter().collect();
    points.sort_by_key(|point| (point.x(), point.y()));
    if points.len() < 3 {
        return 0.0;
    }
    let mut hull = half_hull(points.iter().copied());
    hull.extend(half_hull(points.iter().rev().copied()));
    let first = hull[0];
    hull.windows(2).map(|pair| cross(first, pair[0], pair[1]) as f64).sum::<f64>().abs() * 0.5
}

/// One side of the convex hull of points sorted left to right (Andrew's monotone chain), without
/// its last point, which starts the other side.
fn half_hull(points: impl Iterator<Item = Point>) -> Vec<Point> {
    let mut chain: Vec<Point> = Vec::new();
    for point in points {
        while chain.len() >= 2 && cross(chain[chain.len() - 2], chain[chain.len() - 1], point) <= 0 {
            chain.pop();
        }
        chain.push(point);
    }
    chain.pop();
    chain
}

/// Twice the signed area of the triangle `a`, `b`, `c`: positive when it turns counter-clockwise.
fn cross(a: Point, b: Point, c: Point) -> i32 {
    i32::from(b.x() - a.x()) * i32::from(c.y() - a.y()) - i32::from(b.y() - a.y()) * i32::from(c.x() - a.x())
}

/// 1 for Blue and -1 for Red, to turn Blue's lead into the mover's.
fn sign(player: Player) -> f64 {
    if player == Player::Blue { 1.0 } else { -1.0 }
}
