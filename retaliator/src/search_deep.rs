//! Lane A: deep search — negamax alpha-beta over the game's actions.
//!
//! Plan-v3 §2 / plan-v4 V4e. The lineage verdict shaped this file: fixed 3-ply
//! NAIVE minimax scored -58% league — this eval is not minimax-stable, so depth
//! comes from real alpha-beta with ordering, never naive deepening. Two game
//! facts shape the loop: a turn is 1-2 actions, so the same player often moves
//! twice in a row — the value negates only across turns, never within one
//! (within a turn the mover maximizes, so the window never flips mid-turn);
//! and the eval is a Blue-lead number, so the mover's view is its sign times
//! the eval.
//!
//! Provenance: the evaluation and the root priority function are verbatim
//! copies of `search.rs` (v3 + v4 terms), kept identical so any difference
//! against `search::best_move` isolates the search itself. Three deliberate
//! deltas, all justified:
//! - `one_edge_nodes` runs on a fixed array instead of a HashMap — same
//!   counts, cheaper (the eval runs at every node now, not 4k positions per
//!   move);
//! - the doom discount is dropped: with real depth the search sees the
//!   enemy's pop coming inside the tree, which is what the discount
//!   approximated from outside. Plan-v4 names the catastrophic red line as
//!   needing exactly this depth — this file is the test of that claim;
//! - the beyond-horizon extension is accumulated along the searched line
//!   (signed, Blue-relative, events-free) and applied at the leaf with the
//!   leaf's events factor — the same accounting as the baseline's root-level
//!   extension, correct at any depth;
//! - `value`'s draw check short-circuits at the first legal move instead of
//!   generating them all (the baseline generates every legal move just to ask
//!   whether there are none). Same boolean — a draw is a finished game with
//!   equal scores, or a live game where the mover has no legal move — and
//!   legality still comes from the engine only (`check_move`). This runs at
//!   every node of the deep search, where it was most of the cost.
//! - below the first ply the search values children with [`cheap_value`]
//!   (banked scores + enclosed area at the capped horizon) instead of the
//!   full [`evaluate`]: the room hull, the capture-degree scan and the
//!   stalemate draw check dominate per-node cost and barely move the argmax
//!   at depth. The root and its children (the first ply) keep the full
//!   evaluation, so the deep search still agrees with the baseline's 1-ply
//!   view where that view is all it has.

use std::cmp::Reverse;
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::OnceLock;

use meridian_engine::{Area, Edge, Move, MoveKind, Outcome, Player, Point, Position, Score};

/// The game's action count (engine `position::TOTAL_ACTIONS`, not re-exported).
const TOTAL_ACTIONS: u8 = 120;

// ---------------------------------------------------------------------------
// Evaluation — verbatim from `search.rs` except where noted.
// ---------------------------------------------------------------------------

/// Enclosed area counts as if held for at most this many more scoring events.
const HORIZON: f64 = 12.0;
/// What the room a player's nodes span is worth, per unit of area, against area enclosed.
const ROOM_WEIGHT: f64 = 0.4;
/// Capture exposure: nodes held by a single edge can be captured outright.
const CAPTURE_W: f64 = 3.0;

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

/// [`evaluate`], except that a drawn game is worth 0.
fn value(position: &Position) -> f64 {
    // A draw is a finished game with equal scores, or a live game where the
    // mover has no legal move. `outcome()` is exact for finished games and
    // free (no movegen); the live-game check stops at the first legal move
    // instead of generating them all. Same boolean as the baseline's
    // `outcome_given(&position.legal_moves())` at a fraction of the cost.
    if position.is_finished() {
        return if matches!(position.outcome(), Some(Outcome::Draw(_))) { 0.0 } else { evaluate(position) };
    }
    if !has_legal_move(position) { 0.0 } else { evaluate(position) }
}

/// Whether the mover has any legal move: the engine's own rule check
/// ([`Position::check_move`]) over the mover's nodes and all 48 directions,
/// stopping at the first legal one. The same set `legal_moves()` generates,
/// without generating it all — this runs at every node of the deep search.
pub fn has_legal_move(position: &Position) -> bool {
    let mover = position.to_move();
    position.nodes(mover).iter().any(|node| {
        meridian_engine::geometry::Direction::all().any(|direction| {
            position.check_move(Move { source: node, direction }).is_ok()
        })
    })
}

/// How many of `player`'s nodes hang by a single edge (capturable).
/// Fixed array instead of a HashMap: same counts, no allocation — this runs
/// at every node of the deep search.
pub fn one_edge_nodes(position: &Position, player: Player) -> u32 {
    const NUM_POINTS: usize = meridian_engine::geometry::NUM_POINTS;
    let mut degree = [0u8; NUM_POINTS];
    for edge in position.edges(player).iter() {
        for end in [edge.origin(), edge.far()] {
            degree[end.index()] += 1;
        }
    }
    position.nodes(player).iter().filter(|n| degree[n.index()] == 1).count() as u32
}

/// The area of the convex hull of a player's nodes: room to grow into.
pub fn room(position: &Position, player: Player) -> f64 {
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

/// The beyond-horizon events factor of a position with `events` scoring
/// events left: the share of an area swing past the capped horizon, at half
/// weight. The static evaluation already counts area up to the horizon; this
/// is the rest.
fn extension_factor(events: f64) -> f64 {
    (events - events.min(HORIZON)) * 0.5
}

// ---------------------------------------------------------------------------
// Root priority — verbatim from `search.rs` (v3 + v4 terms).
// ---------------------------------------------------------------------------

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
/// How many of the latest actions count as "recent" for cut avoidance.
pub const CUT_MEMORY: usize = 6;
/// Fresh/shielded-wall avoidance: enemy edges placed last turn can't be
/// cut this turn, so contesting them burns tempo. Penalty for a
/// non-breaking first action landing near them.
const FRESH_RADIUS: i8 = 2;
const FRESH_PENALTY: f64 = 1.5;
/// Do-nothing filter: a first action with no area gain, no break, negligible
/// room growth and nowhere near the enemy does nothing on every axis.
const IDLE_ROOM: f64 = 0.5;
const IDLE_ENEMY_DIST: i8 = 3;
const IDLE_PENALTY: f64 = 2.0;
/// Two-front play: where the position is hot (many close cross-color node
/// pairs), building huge areas FAR from the fighting is nearly unbreakable.
const FIGHT_DIST: i8 = 4;
const FIGHT_HEAT: usize = 10;
const REMOTE_DIST: i8 = 5;
const REMOTE_BONUS: f64 = 1.0;
/// Thicket density: walls that share nodes are near-unbreakable AND bank area.
const DENSE_DIST: i8 = 1;
const DENSE_COUNT: u32 = 2;
const DENSE_BONUS: f64 = 1.0;
/// Patience (rival-analysis steal #2): in the opening, don't snatch tiny
/// loops; wall first, close big later.
const PATIENCE_MAX_GAIN: f64 = 2.0;
const PATIENCE_WINDOW: u8 = 12;
const PATIENCE_PENALTY: f64 = 2.0;

/// XBot move ordering (captures > cuts > closes) — ported from XBot search.rs:34-55.
/// Captures (+100) > cuts (+50) > own-loop closes (+30). Added to priority as the
/// primary move-ordering seed. Toggle for ablation: on vs off at fixed depth/budget.
const USE_XBOT_ORDERING: bool = true;
const XBOT_CAPTURE_BONUS: f64 = 100.0;
const XBOT_CUT_BONUS: f64 = 50.0;
const XBOT_CLOSE_BONUS: f64 = 30.0;

/// Q14 killer moves + history heuristic (on top of the XBot order above).
/// Ordering only: killers/TT-best/history reorder children, never the values
/// (`priority` stays the exact leaf value at depth 1). Toggle for the
/// ablation gate (nodes/pos drop >20% vs XBot alone, value-exact).
const USE_KILLER_HISTORY: bool = true;

/// Aspiration window delta: the half-width around the previous depth's score.
/// When a depth search fails high/low, the window is widened by this factor.
const ASPIRATION_DELTA: f64 = 50.0; // full-horizon points
const ASPIRATION_WIDEN: f64 = 3.0;  // widen factor on fail high/low
const ASPIRATION_MAX_DELTA: f64 = 10000.0; // effectively infinite

/// Blue's forced first action: D10-F7. Measured over 12 games vs three
/// opponents (v1, v2, scout-class): +20% to +41% margins, 12/12 positive.
/// Kept identical to the baseline so the deep-search probes share the opener
/// and isolate the search.
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

struct Ranked {
    mv: Move,
    after: Position,
    value: f64,
    priority: f64,
}

/// The mover's actions, best first, with the position after each and its value. All of them are
/// tried (the deep search wants complete root coverage; truncation is what misled the naive
/// minimax). Longest edges first before prioritizing.
fn ranked(position: &Position, avoid: &[Point]) -> Vec<Ranked> {
    let mover = position.to_move();
    let opp = mover.opponent();
    let room_before = room(position, mover);
    let heat = fight_heat(position, mover, opp);
    let mut moves: Vec<Move> =
        position.legal_moves().iter().filter(|&mv| !repeats_a_connection(position, mv)).collect();
    moves.sort_by_key(|mv| (Reverse(length(*mv)), mv.index()));

    let mut tried: Vec<Ranked> = moves
        .into_iter()
        .map(|mv| {
            let mut after = position.clone();
            let outcome = after.apply_unchecked(mv);
            let value = value(&after);
            let mut priority = sign(mover) * value;
            if after.to_move() == mover && !after.is_finished() {
                priority += loop_bonus(position, mv, &after, room_before);
            }
            // Fix 1: full-horizon claims and denial, in ranking as well as selection.
            priority += horizon_extension(position, &after, mover);
            let target = mv.target().expect("legal moves end on the board");
            let own_gain = after.area(mover).to_f64() - position.area(mover).to_f64();
            // All priorities run in full-horizon points (area x events), so
            // fixed penalties must scale the same way or they never fire.
            let hz = f64::from(position.scoring_events_left()).min(HORIZON);
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
            // board is wide open; set up bigger closes instead.
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
            // Fix 3: don't reinforce the back; expand instead.
            if outcome.kind == MoveKind::Connect
                && own_gain <= 0.0
                && !near_enemy_node(position, opp, target, DEADWOOD_ENEMY_DIST)
            {
                priority -= DEADWOOD_PENALTY * hz;
            }
            // XBot move-ordering seed: captures (+100) > cuts (+50) > own-loop closes (+30).
            // Added to priority so it acts as the primary ordering key. Toggle with USE_XBOT_ORDERING.
            if USE_XBOT_ORDERING {
                let xbot_bonus = match outcome.kind {
                    MoveKind::Capture => XBOT_CAPTURE_BONUS,
                    MoveKind::Connect => {
                        if own_gain > 0.0 && outcome.broken.is_none() {
                            XBOT_CLOSE_BONUS
                        } else if outcome.broken.is_some() {
                            XBOT_CUT_BONUS
                        } else {
                            0.0
                        }
                    }
                    _ => {
                        if outcome.broken.is_some() {
                            XBOT_CUT_BONUS
                        } else {
                            0.0
                        }
                    }
                };
                priority += xbot_bonus;
            }
            Ranked { mv, after, value, priority }
        })
        .collect();
    tried.sort_by(|a, b| b.priority.total_cmp(&a.priority).then(a.mv.index().cmp(&b.mv.index())));
    tried
}

/// Whether `target` is within Chebyshev `dist` of any of the given points.
fn near_points(mut points: impl Iterator<Item = Point>, target: Point, dist: i8) -> bool {
    points.any(|p| (p.x() - target.x()).abs() <= dist && (p.y() - target.y()).abs() <= dist)
}

/// How many close opposing-node pairs the position holds: the fight heat.
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
pub fn repeats_a_connection(position: &Position, mv: Move) -> bool {
    let target = mv.target().expect("legal moves end on the board");
    position.nodes(position.to_move()).contains(target) && target < mv.source
}

/// The cheap evaluation used below the root: banked scores plus enclosed area
/// at the capped horizon. Skips the room hull, the capture-degree scan and the
/// draw check — the parts that dominate per-node cost at depth. The full
/// [`evaluate`] runs at the root and the first ply only.
pub fn cheap_value(position: &Position) -> f64 {
    cheap_parts(
        position.score(Player::Blue),
        position.score(Player::Red),
        position.area(Player::Blue),
        position.area(Player::Red),
        position.scoring_events_left(),
    )
}

/// [`cheap_value`] from parts, so the apply-free fast path below and the
/// materialized path agree bit for bit: same expression shape, same rounding.
fn cheap_parts(score_b: Score, score_r: Score, area_b: Area, area_r: Area, events_left: u8) -> f64 {
    let events_left = f64::from(events_left);
    let worth = |score: Score, area: Area| score.to_f64() + area.to_f64() * events_left.min(HORIZON);
    worth(score_b, area_b) - worth(score_r, area_r)
}

// ---------------------------------------------------------------------------
// Fast children — apply-free evaluation of moves that cannot change an area.
//
// `Position::apply_unchecked` only recomputes an area in two cases: the move
// cuts an enemy edge (removing it can only shrink that area) or it may close
// a loop of the mover's (a connect, or an edge crossing one of the mover's
// edges away from its ends). Every other child keeps both areas, so its
// scores (banked at turn ends from unchanged areas) and its cheap value are
// pure arithmetic. Deciding which case a move is in needs the engine's own
// "shares a point" test, so it is copied verbatim below and precomputed into
// a directions bitmask per edge origin offset — the same trick the engine's
// own tables use, so the classification is exact, not an approximation.
// ---------------------------------------------------------------------------

/// The king-step reach of a move (the engine's window is 7 x 7).
const REACH: i32 = 3;
/// How far an edge's origin can be from a move's source and still touch it.
const NEARBY: i32 = 2 * REACH;
const NEARBY_SIDE: usize = (2 * NEARBY + 1) as usize;
const NUM_CANONICAL: usize = meridian_engine::geometry::NUM_CANONICAL_DIRECTIONS;

/// Twice the signed area of `a` x `b`.
fn seg_cross(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    ax * by - ay * bx
}

/// Which side of the line through `a` and `b` the point `p` is on.
fn seg_side(ax: i32, ay: i32, bx: i32, by: i32, px: i32, py: i32) -> i32 {
    seg_cross(bx - ax, by - ay, px - ax, py - ay).signum()
}

/// For a point `p` on the line through `a` and `b`: whether it lies between them.
fn seg_within(ax: i32, ay: i32, bx: i32, by: i32, px: i32, py: i32) -> bool {
    px >= ax.min(bx) && px <= ax.max(bx) && py >= ay.min(by) && py <= ay.max(by)
}

/// Whether the segments `ab` and `cd` share a point, ends included. The
/// engine's own `segments_touch` (vendor/meridian-bot-kit/engine/src/geometry.rs),
/// on plain coordinates; the fast/slow decision below must classify exactly
/// what `Position::apply_unchecked` treats as a cut or a crossing.
fn segments_touch(ax: i32, ay: i32, bx: i32, by: i32, cx: i32, cy: i32, dx: i32, dy: i32) -> bool {
    let (c_side, d_side) = (seg_side(ax, ay, bx, by, cx, cy), seg_side(ax, ay, bx, by, dx, dy));
    let (a_side, b_side) = (seg_side(cx, cy, dx, dy, ax, ay), seg_side(cx, cy, dx, dy, bx, by));
    if c_side != d_side && a_side != b_side {
        return true;
    }
    (c_side == 0 && seg_within(ax, ay, bx, by, cx, cy))
        || (d_side == 0 && seg_within(ax, ay, bx, by, dx, dy))
        || (a_side == 0 && seg_within(cx, cy, dx, dy, ax, ay))
        || (b_side == 0 && seg_within(cx, cy, dx, dy, bx, by))
}

/// `masks[(oy + NEARBY) * side + (ox + NEARBY)][canonical]`: the bitmask of
/// candidate directions (bit = `Direction::index`) whose segment from the
/// source shares a point with the edge from `(ox, oy)` in `canonical`.
/// One process-wide build: 13 x 13 offsets x 24 canonical x 48 candidates.
fn touch_masks() -> &'static Box<[u64; NEARBY_SIDE * NEARBY_SIDE * NUM_CANONICAL]> {
    static MASKS: OnceLock<Box<[u64; NEARBY_SIDE * NEARBY_SIDE * NUM_CANONICAL]>> = OnceLock::new();
    MASKS.get_or_init(|| {
        let mut masks = Box::new([0u64; NEARBY_SIDE * NEARBY_SIDE * NUM_CANONICAL]);
        for oy in -NEARBY..=NEARBY {
            for ox in -NEARBY..=NEARBY {
                for canonical in meridian_engine::geometry::Direction::all()
                    .filter(|direction| direction.is_canonical())
                {
                    let (cdx, cdy) = (i32::from(canonical.dx()), i32::from(canonical.dy()));
                    let mut mask = 0u64;
                    for candidate in meridian_engine::geometry::Direction::all() {
                        let (dx, dy) = (i32::from(candidate.dx()), i32::from(candidate.dy()));
                        if segments_touch(0, 0, dx, dy, ox, oy, ox + cdx, oy + cdy) {
                            mask |= 1 << candidate.index();
                        }
                    }
                    let index = ((oy + NEARBY) as usize * NEARBY_SIDE + (ox + NEARBY) as usize)
                        * NUM_CANONICAL
                        + (canonical.index() - meridian_engine::geometry::NUM_DIRECTIONS / 2);
                    masks[index] = mask;
                }
            }
        }
        masks
    })
}

/// The directions from `source` whose move shares a point with `edge`.
fn touching_directions(source: Point, edge: Edge) -> u64 {
    let ox = i32::from(edge.origin().x()) - i32::from(source.x());
    let oy = i32::from(edge.origin().y()) - i32::from(source.y());
    if ox.abs() > NEARBY || oy.abs() > NEARBY {
        return 0;
    }
    let index = (oy + NEARBY) as usize * NEARBY_SIDE * NUM_CANONICAL
        + (ox + NEARBY) as usize * NUM_CANONICAL
        + (edge.canonical_direction().index() - meridian_engine::geometry::NUM_DIRECTIONS / 2);
    touch_masks()[index]
}

/// The directions from `source` that need the engine, not arithmetic: any
/// enemy edge they touch is cut (that area can shrink), and any own edge they
/// touch away from `source` can close a loop (that area can grow). Edges with
/// `source` as an endpoint always "touch" but never do either.
pub fn source_danger(position: &Position, source: Point) -> u64 {
    let mover = position.to_move();
    let mut mask = 0u64;
    for edge in position.edges(mover).iter() {
        if !edge.has_endpoint(source) {
            mask |= touching_directions(source, edge);
        }
    }
    for edge in position.edges(mover.opponent()).iter() {
        mask |= touching_directions(source, edge);
    }
    mask
}

/// The 1-ply facts of a child the ordering pass needs: the cheap value after
/// the move, the area swing (own gained + enemy destroyed), whether the game
/// ends, and the scoring events left. Pure arithmetic for moves that cannot
/// change an area; everything else is materialized through the engine.
#[derive(Debug)]
pub struct ChildFacts {
    pub val: f64,
    pub swing: f64,
    pub finished: bool,
    pub events: u8,
}

/// [`ChildFacts`] for `mv` from `position`, given `source_danger`'s mask for
/// `mv.source`. `None` when the move must be materialized: it connects or
/// captures (kind), cuts, or may close a loop.
pub fn child_facts(position: &Position, mv: Move, danger: u64) -> Option<ChildFacts> {
    let target = mv.target().expect("legal moves end on the board");
    if position.node_owner(target).is_some() || danger >> mv.direction.index() & 1 != 0 {
        return None;
    }
    // No cut, no loop close: `apply_unchecked` would only bank scores at the
    // turn end (or game end) from unchanged areas, and advance the clock.
    let actions = position.actions_played() + 1;
    let finished = actions >= TOTAL_ACTIONS;
    let events = if finished { 0 } else { (TOTAL_ACTIONS - actions) / 2 + 1 };
    let banked = actions % 2 == 1 || finished;
    let (mut score_b, mut score_r) = (position.score(Player::Blue), position.score(Player::Red));
    if banked {
        score_b += position.area(Player::Blue);
        score_r += position.area(Player::Red);
    }
    Some(ChildFacts {
        val: cheap_parts(
            score_b,
            score_r,
            position.area(Player::Blue),
            position.area(Player::Red),
            events,
        ),
        swing: 0.0,
        finished,
        events,
    })
}

fn length(mv: Move) -> i8 {
    mv.direction.dx().abs().max(mv.direction.dy().abs())
}

/// This process's CPU time in seconds (utime + stime from `/proc/self/stat`):
/// the load-independent "native" clock a [`Budget::Ms`] runs on. 10 ms
/// resolution — plenty for caps of hundreds of milliseconds.
fn cpu_now() -> f64 {
    let stat = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let Some(rest) = stat.rsplit_once(')').map(|(_, rest)| rest) else { return 0.0 };
    let fields = rest.split_whitespace().collect::<Vec<_>>();
    // fields[0] is /proc's field 3 (state), so utime is fields[11].
    let utime: u64 = fields.get(11).and_then(|s| s.parse().ok()).unwrap_or(0);
    let stime: u64 = fields.get(12).and_then(|s| s.parse().ok()).unwrap_or(0);
    (utime + stime) as f64 / 100.0
}

/// A per-move budget for [`analyze_with_budget`].
pub enum Budget {
    /// Native CPU milliseconds, soft: a new depth starts only while under the
    /// cap, and a running depth additionally stops between root children once
    /// the cap is spent (partial, priority-ordered root coverage — the same
    /// shape as a width-capped search). NOT deterministic across runs.
    Ms(u64),
    /// Total nodes across iterations, checked between root children:
    /// deterministic — the same budget searches the same tree every time.
    Nodes(u64),
    /// No budget: the full depth, complete root coverage. The gates run this.
    Unbounded,
}

/// The deployment default: 2000 ms native per move.
pub const DEFAULT_BUD_MS: u64 = 2000;

/// How wide the search looks: root children searched, and children per
/// interior node, both in priority order with the move-index tiebreak.
/// Turn-start roots are why: their depth-3 tree is ~b^2 leaf-parent nodes
/// (the same-mover ply hands down a +INF beta), so full width there costs
/// tens of seconds; 8 x 6 makes depth 3 a few tens of milliseconds.
/// Deterministic — same widths, same tree, every run.
#[derive(Clone, Copy)]
pub struct Width {
    pub root: usize,
    pub inner: usize,
}

/// The capped default: 8 root children, 6 below.
pub const DEFAULT_WIDTH: Width = Width { root: 8, inner: 6 };
/// Full width: the uncapped search the earlier lanes measured.
pub const FULL_WIDTH: Width = Width { root: usize::MAX, inner: usize::MAX };

/// When a running search stops: the root loop checks between children.
enum Stop {
    Cpu { started: f64, cap: f64 },
    Nodes { used: u64, limit: u64 },
}

impl Stop {
    fn exceeded(&self, ctx: &Ctx) -> bool {
        match self {
            Stop::Cpu { started, cap } => cpu_now() - started >= *cap,
            Stop::Nodes { used, limit } => used + ctx.nodes >= *limit,
        }
    }
}

/// Keep the deeper iteration's candidates, filling root moves it did not
/// reach from the shallower one. Deeper values are the informed ones;
/// shallower values only fill gaps, so a budget that reaches few root
/// children degrades towards the previous depth rather than to noise.
fn merge_analyses(mut deeper: DeepAnalysis, shallower: DeepAnalysis, nodes: u64) -> DeepAnalysis {
    let have: std::collections::HashSet<usize> =
        deeper.candidates.iter().map(|c| c.mv.index()).collect();
    for candidate in shallower.candidates {
        if !have.contains(&candidate.mv.index()) {
            deeper.candidates.push(candidate);
        }
    }
    deeper
        .candidates
        .sort_by(|a, b| b.evaluation.total_cmp(&a.evaluation).then(a.mv.index().cmp(&b.mv.index())));
    if let Some(best) = deeper.candidates.first() {
        deeper.evaluation = best.evaluation;
    }
    deeper.nodes = nodes;
    deeper
}

/// Iterative deepening under a budget: depth 1, 2, ..., `max_depth`, keeping
/// the deepest analysis that completed (or, under `Ms`, the deepest whose
/// root loop got at least one child in). Full width — see [`analyze_capped`]
/// for the width-capped form.
pub fn analyze_with_budget(
    position: &Position,
    max_depth: u8,
    avoid: &[Point],
    budget: &Budget,
) -> DeepAnalysis {
    analyze_capped(position, max_depth, avoid, budget, FULL_WIDTH)
}

/// [`analyze_with_budget`] with width caps: the configurable form. The
/// deployment line is `Budget::Ms(DEFAULT_BUD_MS)` + `DEFAULT_WIDTH`; the
/// gates run `Budget::Unbounded` + `DEFAULT_WIDTH` (deterministic).
pub fn analyze_capped(
    position: &Position,
    max_depth: u8,
    avoid: &[Point],
    budget: &Budget,
    width: Width,
) -> DeepAnalysis {
    analyze_capped_kh(position, max_depth, avoid, budget, width, USE_KILLER_HISTORY)
}

/// [`analyze_capped`] with explicit killer/history ordering control: the Q14
/// ablation gate (nodes/pos drop >20% vs XBot alone, value-exact). Production
/// always passes `USE_KILLER_HISTORY`.
pub fn analyze_capped_kh(
    position: &Position,
    max_depth: u8,
    avoid: &[Point],
    budget: &Budget,
    width: Width,
    use_kh: bool,
) -> DeepAnalysis {
    match budget {
        Budget::Unbounded => {
            // Single-depth call (used by gates): no iterative deepening, no aspiration
            analyze_impl(position, max_depth, avoid, None, width, None, use_kh)
        }
        Budget::Ms(cap_ms) => {
            let started = cpu_now();
            let cap = *cap_ms as f64 / 1000.0;
            let mut analysis = analyze_impl(position, 1, avoid, None, width, None, use_kh);
            let mut nodes = analysis.nodes;
            for depth in 2..=max_depth {
                if cpu_now() - started >= cap {
                    break;
                }
                let stop = Stop::Cpu { started, cap };
                // Aspiration window centered on previous depth's best evaluation.
                // `evaluation` is Blue-relative; the window lives in the
                // mover's perspective, so convert (matters when mover is Red).
                let aspiration_center = sign(position.to_move()) * analysis.evaluation;
                let next = analyze_impl(position, depth, avoid, Some(&stop), width, Some(aspiration_center), use_kh);
                nodes += next.nodes;
                analysis = merge_analyses(next, analysis, nodes);
            }
            analysis
        }
        Budget::Nodes(limit) => {
            let mut analysis = analyze_impl(position, 1, avoid, None, width, None, use_kh);
            let mut used = analysis.nodes;
            for depth in 2..=max_depth {
                if used >= *limit {
                    break;
                }
                let stop = Stop::Nodes { used, limit: *limit };
                // Mover-perspective center (see Ms arm): `evaluation` is Blue-relative.
                let aspiration_center = sign(position.to_move()) * analysis.evaluation;
                let next = analyze_impl(position, depth, avoid, Some(&stop), width, Some(aspiration_center), use_kh);
                used += next.nodes;
                analysis = merge_analyses(next, analysis, used);
            }
            analysis
        }
    }
}

/// Like [`best_move_with_avoid`], but budget + width configurable: the
/// deployment path (`Budget::Ms(DEFAULT_BUD_MS)` + `DEFAULT_WIDTH`) and the
/// gate configuration (`Budget::Unbounded` + `DEFAULT_WIDTH`) in one entry.
pub fn best_move_capped(
    position: &Position,
    max_depth: u8,
    avoid: &[Point],
    budget: &Budget,
    width: Width,
) -> Option<Move> {
    if let Some(open) = blue_opener(position) {
        return Some(open);
    }
    analyze_capped(position, max_depth, avoid, budget, width)
        .candidates
        .first()
        .map(|c| c.mv)
}

/// Like [`best_move_with_avoid`], but under a [`Budget`] with iterative
/// deepening: the deployment path. `Budget::Ms(DEFAULT_BUD_MS)` holds a
/// ~2 s/move native line even where full depth 3 does not fit.
pub fn best_move_with_budget(
    position: &Position,
    max_depth: u8,
    avoid: &[Point],
    budget: &Budget,
) -> Option<Move> {
    if let Some(open) = blue_opener(position) {
        return Some(open);
    }
    analyze_with_budget(position, max_depth, avoid, budget)
        .candidates
        .first()
        .map(|c| c.mv)
}

/// For a first action the second can close into a triangle: the room it adds, up to that
/// triangle's area, for the scoring events left.
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

/// The beyond-horizon part of an area swing along one ply, mover-relative: own
/// area gained plus enemy area destroyed, times the scoring events past the
/// horizon. The static evaluation already counts both up to the horizon.
fn horizon_extension(before: &Position, end: &Position, mover: Player) -> f64 {
    let gained = (end.area(mover).to_f64() - before.area(mover).to_f64()).max(0.0);
    let destroyed =
        (before.area(mover.opponent()).to_f64() - end.area(mover.opponent()).to_f64()).max(0.0);
    let events = f64::from(end.scoring_events_left());
    (gained + destroyed) * (events - events.min(HORIZON)) * 0.5
}

// ---------------------------------------------------------------------------
// Deep search: negamax alpha-beta over actions.
// ---------------------------------------------------------------------------

/// Default depth of the search, in actions.
pub const DEFAULT_DEPTH: u8 = 3;
/// Deepest ply the PV table tracks (one per action, plus slack).
const MAX_PLY: usize = 121;

pub struct DeepAnalysis {
    /// Blue's lead after the best line, in the same points as `search::Analysis`.
    pub evaluation: f64,
    /// The depth searched, in actions.
    pub depth: usize,
    /// Positions searched.
    pub nodes: u64,
    /// Transposition diagnostics: value hits and stores.
    pub tt_hits: u64,
    pub tt_stores: u64,
    /// The best moves first.
    pub candidates: Vec<DeepCandidate>,
}

pub struct DeepCandidate {
    pub mv: Move,
    /// Blue's lead after this move and the best continuation.
    pub evaluation: f64,
    /// The line this move starts.
    pub pv: Vec<Move>,
    /// Nodes searched under this move.
    pub visits: u64,
}

struct Ctx {
    nodes: u64,
    /// Triangular PV table: `pv[ply * MAX_PLY ..][.. pv_len[ply]]`.
    pv: Box<[Move]>,
    pv_len: [usize; MAX_PLY],
    /// Transpositions within this search. Cleared per move: entries are only
    /// exact in the context of one root's accumulated extensions.
    tt: HashMap<u64, TtEntry>,
    /// Diagnostics: value hits (a depth-1 pass skipped), any-probes, stores.
    tt_hits: u64,
    tt_stores: u64,
    /// This search's width: children per interior node, priority-ordered.
    width: Width,
    /// Killer moves (Q14): two per ply, tried right after the TT move. Cutoff
    /// producers — ordering only, never touch values. Cleared per search.
    killers: [[Option<Move>; 2]; MAX_PLY],
    /// History heuristic (Q14): cumulative depth-squared cutoff credit per
    /// move index. Breaks priority ties in ordering; never touches values.
    history: Box<[i32; meridian_engine::NUM_MOVES]>,
    /// Whether killer/history ordering is active (ablation toggle for the
    /// >20% nodes/pos gate; production always runs with it on).
    use_kh: bool,
}

impl Ctx {
    fn new() -> Ctx {
        Ctx {
            nodes: 0,
            pv: vec![Move::from_index(0).expect("move 0 exists"); MAX_PLY * MAX_PLY].into_boxed_slice(),
            pv_len: [0; MAX_PLY],
            tt: HashMap::new(),
            tt_hits: 0,
            tt_stores: 0,
            width: FULL_WIDTH,
            killers: [[None; 2]; MAX_PLY],
            history: vec![0i32; meridian_engine::NUM_MOVES]
                .into_boxed_slice()
                .try_into()
                .expect("history table sized by NUM_MOVES"),
            use_kh: USE_KILLER_HISTORY,
        }
    }

    fn line(&self, ply: usize) -> Vec<Move> {
        self.pv[ply * MAX_PLY..ply * MAX_PLY + self.pv_len[ply]].to_vec()
    }
}

/// The position's identity for the transposition table: the engine's own
/// structural hash (edges, scores, shielded, fresh, actions played), through
/// `DefaultHasher` — fixed keys, so searches stay deterministic across runs.
/// 64-bit collisions are the standard accepted risk; nothing verifies a hit.
fn position_hash(position: &Position) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(position, &mut hasher);
    hasher.finish()
}

/// One transposition-table entry. The value parts belong to depth-1 nodes,
/// where every live child shares one extension factor, so the node's value is
/// `max(p_finish, p_live + sign(mover) * ext * factor)`: path-independent
/// parts plus the caller's accumulated line extension, exact for any path.
/// `exact` says every child was searched (a cutoff leaves a lower bound,
/// sound only for beta cutoffs). `stalemate` marks the no-legal-move draw.
/// `best` is the best child found, for move ordering at any depth.
#[derive(Clone)]
struct TtEntry {
    best: Option<Move>,
    valued: bool,
    exact: bool,
    stalemate: bool,
    p_finish: f64,
    p_live: f64,
    factor: f64,
}

impl TtEntry {
    fn new() -> TtEntry {
        TtEntry {
            best: None,
            valued: false,
            exact: false,
            stalemate: false,
            p_finish: f64::NEG_INFINITY,
            p_live: f64::NEG_INFINITY,
            factor: 0.0,
        }
    }

    /// The node's value (or, when not `exact`, a lower bound on it) under the
    /// caller's line extension `ext`.
    fn bound(&self, mover: Player, ext: f64) -> f64 {
        if self.stalemate {
            return sign(mover) * ext * self.factor;
        }
        let live = self.p_live + sign(mover) * ext * self.factor;
        if self.p_finish > live {
            self.p_finish
        } else {
            live
        }
    }
}

pub fn best_move(position: &Position) -> Option<Move> {
    best_move_with_avoid(position, DEFAULT_DEPTH, &[])
}

/// Like [`best_move`], but steering clear of `avoid`: points near our loops
/// the enemy recently cut. Rebuilding there is how rebuilder-farming works.
pub fn best_move_with_avoid(position: &Position, depth: u8, avoid: &[Point]) -> Option<Move> {
    if let Some(open) = blue_opener(position) {
        return Some(open);
    }
    analyze_with_avoid(position, depth, avoid).candidates.first().map(|candidate| candidate.mv)
}

pub fn analyze(position: &Position, depth: u8) -> DeepAnalysis {
    analyze_with_avoid(position, depth, &[])
}

pub fn analyze_with_avoid(position: &Position, depth: u8, avoid: &[Point]) -> DeepAnalysis {
    analyze_impl(position, depth, avoid, None, FULL_WIDTH, None, USE_KILLER_HISTORY)
}

/// The analysis itself. `stop` is the budget's root-level checkpoint: the
/// root loop checks between children (the first always runs), so a budgeted
/// search keeps the deepest completed iteration's full work plus whatever
/// root children fit under the cap. `width` caps how many root children are
/// searched at all; the root list is priority-ordered, so this is the same
/// "top-N by 1-ply priority" cut the budget makes, but deterministic.
/// `aspiration` is an optional center value for aspiration window search,
/// in the mover's perspective (Blue-relative evaluation times the mover's
/// sign): if Some(score), the root search uses a narrow window around that
/// score, widening on fail high/low until the true value is bracketed.
fn analyze_impl(
    position: &Position,
    depth: u8,
    avoid: &[Point],
    stop: Option<&Stop>,
    width: Width,
    aspiration: Option<f64>,
    use_kh: bool,
) -> DeepAnalysis {
    let mover = position.to_move();
    let opp = mover.opponent();
    let root = ranked(position, avoid);
    let root_moves: Vec<Ranked> = root.into_iter().take(width.root).collect();

    // Create context once; reused across aspiration re-searches (TT persists).
    let mut ctx = Ctx::new();
    ctx.width = width;
    ctx.use_kh = use_kh;

    // Aspiration window search: start narrow around the previous depth's best score,
    // widen on fail high/low until the true value is bracketed.
    let mut delta = aspiration.map(|_| ASPIRATION_DELTA).unwrap_or(f64::INFINITY);
    let mut best_analysis: Option<DeepAnalysis> = None;
    let mut first_search = true;

    loop {
        // Clear TT before each re-search to avoid pollution from failed-window bounds.
        // We keep the PV table but the TT must be clean for correct bounds.
        if !first_search {
            ctx.tt.clear();
        }
        first_search = false;

        let mut scored: Vec<(f64, DeepCandidate)> = Vec::with_capacity(root_moves.len());

        // Initial alpha/beta for the root search
        let (mut alpha, mut beta) = if let Some(center) = aspiration {
            (center - delta, center + delta)
        } else {
            (f64::NEG_INFINITY, f64::INFINITY)
        };

        let mut search_aborted = false;
        for (searched, Ranked { mv, after, priority, .. }) in root_moves.iter().enumerate() {
            if searched > 0 && stop.is_some_and(|stop| stop.exceeded(&ctx)) {
                search_aborted = true;
                break;
            }
            let before_nodes = ctx.nodes;
            let (v, child_pv) = if after.is_finished() || depth <= 1 {
                ctx.pv_len[1] = 0;
                (*priority, Vec::<Move>::new())
            } else {
                let swing = (after.area(mover).to_f64() - position.area(mover).to_f64()).max(0.0)
                    + (position.area(opp).to_f64() - after.area(opp).to_f64()).max(0.0);
                let child_ext = sign(mover) * swing;
                let v = if after.to_move() == mover {
                    negamax(&after, depth - 1, 1, alpha, beta, child_ext, &mut ctx)
                } else {
                    -negamax(&after, depth - 1, 1, -beta, -alpha, child_ext, &mut ctx)
                };
                let child_pv = ctx.line(1);
                (v, child_pv)
            };
            let mut pv = vec![mv.clone()];
            pv.extend(child_pv);
            let mut adjusted = v;
            {
                let hz_sel = f64::from(position.scoring_events_left()).min(HORIZON);
                let cut_something = after.edges(opp).len() < position.edges(opp).len();
                if !cut_something {
                    let tgt = mv.target().expect("legal moves end on the board");
                    if near_points(avoid.iter().copied(), tgt, CUT_RADIUS) {
                        adjusted -= REBUILD_PENALTY * hz_sel;
                    }
                    if near_fresh_enemy(position, tgt) {
                        adjusted -= FRESH_PENALTY * hz_sel;
                    }
                }
            }
            let visits = ctx.nodes - before_nodes;
            scored.push((adjusted, DeepCandidate { mv: mv.clone(), evaluation: sign(mover) * v, pv, visits }));
            if v > alpha {
                alpha = v;
            }
            if alpha >= beta {
                break;
            }
        }

        if search_aborted && best_analysis.is_some() {
            return best_analysis.unwrap();
        }

        scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.mv.index().cmp(&b.1.mv.index())));
        let candidates = scored.into_iter().map(|(_, c)| c).collect::<Vec<_>>();
        let evaluation = candidates.first().map_or_else(|| value(position), |best| best.evaluation);
        let analysis = DeepAnalysis {
            evaluation,
            depth: usize::from(depth),
            nodes: ctx.nodes,
            tt_hits: ctx.tt_hits,
            tt_stores: ctx.tt_stores,
            candidates,
        };

        // Check if aspiration window succeeded (mover-perspective best value
        // within the mover-perspective window around the center).
        if let Some(center) = aspiration {
            let mover_best = sign(mover) * evaluation;
            if mover_best > center - delta && mover_best < center + delta {
                return analysis; // Window succeeded
            }
            // Fail high or fail low: widen window and re-search
            delta *= ASPIRATION_WIDEN;
            if delta > ASPIRATION_MAX_DELTA {
                // Window wide enough, accept this result
                return analysis;
            }
            // Continue loop with widened window; context (TT) is reused
            best_analysis = Some(analysis);
            // Reset PV lens for the new search (but keep TT)
            ctx.pv_len.fill(0);
            continue;
        }

        return analysis;
    }
}

/// Negamax alpha-beta over actions. Returns the value from the perspective of
/// `pos.to_move()`. `ext` is the signed, Blue-relative area swing accumulated
/// along the line so far (events-free); the beyond-horizon factor is applied
/// at the leaf, where `scoring_events_left` is known.
fn negamax(
    pos: &Position,
    depth: u8,
    ply: usize,
    mut alpha: f64,
    beta: f64,
    ext: f64,
    ctx: &mut Ctx,
) -> f64 {
    ctx.nodes += 1;
    let mover = pos.to_move();
    if pos.is_finished() {
        ctx.pv_len[ply] = 0;
        return sign(mover) * value(pos);
    }
    if depth == 0 {
        ctx.pv_len[ply] = 0;
        let events = f64::from(pos.scoring_events_left());
        return sign(mover) * cheap_value(pos) + sign(mover) * ext * extension_factor(events);
    }
    let key = position_hash(pos);
    // A depth-1 node's value is exactly the stored parts plus this line's
    // extension, so a hit replaces the whole movegen + ordering pass. A
    // cutoff bound can only be reused for another cutoff.
    if depth == 1 {
        if let Some(entry) = ctx.tt.get(&key) {
            if entry.valued {
                let bound = entry.bound(mover, ext);
                if entry.exact || bound >= beta {
                    ctx.tt_hits += 1;
                    ctx.pv_len[ply] = 0;
                    return bound;
                }
            }
        }
    }
    let moves = pos.legal_moves();
    let legal: Vec<Move> =
        moves.iter().filter(|&mv| !repeats_a_connection(pos, mv)).collect();
    if legal.is_empty() {
        ctx.pv_len[ply] = 0;
        // The game is a draw: the static part is 0, the line's extension counts.
        let factor = extension_factor(f64::from(pos.scoring_events_left()));
        let mut entry = ctx.tt.remove(&key).unwrap_or_else(TtEntry::new);
        entry.valued = true;
        entry.exact = true;
        entry.stalemate = true;
        entry.factor = factor;
        ctx.tt.insert(key, entry);
        return sign(mover) * ext * factor;
    }

    // Order the children: mover-signed 1-ply value + horizon extension — the
    // core of the root priority function. The first-action penalties are
    // root-only (tuned there); depth sees their consequences anyway. A
    // connection between own nodes is deduped by `repeats_a_connection`, so
    // no position is searched twice.
    //
    // Below the first ply the static value is [`cheap_value`]: banked scores
    // plus enclosed area at the capped horizon. The room hull, the
    // capture-degree scan and the stalemate draw check that make up most of
    // [`evaluate`]'s cost do not change the argmax much at depth — area is
    // the dominant term — and the root/first ply still see them in full.
    // Stalemate (a live leaf where the mover has no legal move) is the one
    // exactness loss: `value` scores it 0, `cheap_value` scores the material.
    // Interior nodes still catch it exactly via their own movegen.
    //
    // Children that cannot change an area — the large majority: no cut, no
    // loop close — are arithmetic (see [`child_facts`]); only the rest pay
    // for a clone and an apply.
    let mut danger = [0u64; meridian_engine::geometry::NUM_POINTS];
    for (source, _) in moves.by_source() {
        danger[source.index()] = source_danger(pos, source);
    }
    let mut children: Vec<(f64, Move, f64, f64, bool)> = Vec::with_capacity(legal.len());
    for mv in legal {
        let Some(facts) = child_facts(pos, mv, danger[mv.source.index()]) else {
            // This child needs the engine: it connects or captures, cuts an
            // enemy edge, or may close a loop of the mover's.
            let target_owner = pos.node_owner(mv.target().expect("legal moves end on the board"));
            let mut after = pos.clone();
            after.apply_unchecked(mv);
            let gained = (after.area(mover).to_f64() - pos.area(mover).to_f64()).max(0.0);
            let destroyed =
                (pos.area(mover.opponent()).to_f64() - after.area(mover.opponent()).to_f64()).max(0.0);
            let swing = gained + destroyed;
            let finished = after.is_finished();
            let factor_c = extension_factor(f64::from(after.scoring_events_left()));
            let mut priority = sign(mover) * cheap_value(&after) + swing * factor_c;
            // XBot move-ordering seed for child moves (captures > cuts > closes).
            if USE_XBOT_ORDERING {
                let xbot_bonus = match target_owner {
                    Some(owner) if owner == mover.opponent() => XBOT_CAPTURE_BONUS, // Capture
                    Some(_) => {
                        // Connect: own-loop close only if gained area and didn't cut.
                        let cut = after.edges(mover.opponent()).len() < pos.edges(mover.opponent()).len();
                        if gained > 0.0 && !cut { XBOT_CLOSE_BONUS } else if cut { XBOT_CUT_BONUS } else { 0.0 }
                    }
                    None => {
                        // Extend: cut if it broke enemy edges.
                        if after.edges(mover.opponent()).len() < pos.edges(mover.opponent()).len() {
                            XBOT_CUT_BONUS
                        } else {
                            0.0
                        }
                    }
                };
                priority += xbot_bonus;
            }
            // A finished child's leaf has 0 events left: the extension vanishes.
            let leaf_factor = if finished { 0.0 } else { factor_c };
            children.push((priority, mv, leaf_factor, swing, finished));
            continue;
        };
        let factor_c = extension_factor(f64::from(facts.events));
        let priority = sign(mover) * facts.val + facts.swing * factor_c;
        // Fast-path moves (Extend, no cut, no close) have XBot bonus = 0.
        // A finished child's leaf has 0 events left: the extension vanishes.
        let leaf_factor = if facts.finished { 0.0 } else { factor_c };
        children.push((priority, mv, leaf_factor, facts.swing, facts.finished));
    }
    // Move ordering (Q14): the TT move first, then this ply's killers, then
    // priority with history breaking ties and the move index last. Ordering
    // only — the values (`priority`) are untouched, so an exact (full-window)
    // search returns the same value and best move; only cutoffs arrive sooner.
    // With `use_kh` off this degrades to the old shape (TT move first — which
    // was coded but never populated before Q14 — then priority, then index).
    let tt_best =
        ctx.tt.get(&key).and_then(|entry| entry.best).map(|candidate| candidate.index());
    let (k0, k1) = if ctx.use_kh && ply < MAX_PLY {
        (
            ctx.killers[ply][0].map(|killer| killer.index()),
            ctx.killers[ply][1].map(|killer| killer.index()),
        )
    } else {
        (None, None)
    };
    let tier = |id: usize| {
        if Some(id) == tt_best {
            0
        } else if Some(id) == k0 {
            1
        } else if Some(id) == k1 {
            2
        } else {
            3
        }
    };
    children.sort_by(|a, b| {
        tier(a.1.index())
            .cmp(&tier(b.1.index()))
            .then(b.0.total_cmp(&a.0))
            .then(ctx.history[b.1.index()].cmp(&ctx.history[a.1.index()]))
            .then(a.1.index().cmp(&b.1.index()))
    });
    // Width cap: the beam. The children are priority-ordered (the TT move
    // first), so this keeps the sharpest few — cuts, captures and area
    // swings sort high in the cheap priority — and drops the quiet tail.
    children.truncate(ctx.width.inner);

    let mut best = f64::NEG_INFINITY;
    let mut best_move: Option<Move> = None;
    let mut cutoff = false;
    // The depth-1 value parts to store: priority maxima by child kind, and
    // the one extension factor every live child shares.
    let (mut p_finish, mut p_live, mut live_factor) = (f64::NEG_INFINITY, f64::NEG_INFINITY, 0.0);
    for (priority, mv, leaf_factor, swing, finished) in children {
        if depth == 1 {
            if finished {
                if priority > p_finish {
                    p_finish = priority;
                }
            } else if priority > p_live {
                p_live = priority;
                live_factor = leaf_factor;
            }
        }
        let v = if finished {
            // The game ended: the extension vanishes (0 events left), so the
            // ordering value is already the exact value.
            priority
        } else if depth == 1 {
            // Exact leaf shortcut: the ordering pass already computed the
            // static value and this ply's extension; the line's accumulated
            // swing adds one multiply. (priority = static + swing x factor,
            // and the true leaf value = priority + sign(mover) x ext x factor.)
            priority + sign(mover) * ext * leaf_factor
        } else {
            let mut after = pos.clone();
            after.apply_unchecked(mv);
            let child_ext = ext + sign(mover) * swing;
            if after.to_move() == mover {
                // Same player's next action: no negation, window unflipped.
                negamax(&after, depth - 1, ply + 1, alpha, beta, child_ext, ctx)
            } else {
                -negamax(&after, depth - 1, ply + 1, -beta, -alpha, child_ext, ctx)
            }
        };
        if v > best {
            best = v;
            best_move = Some(mv);
            // PV update: this move, then the child's line.
            let base = ply * MAX_PLY;
            ctx.pv[base] = mv;
            let len = if finished || depth == 1 { 0 } else { ctx.pv_len[ply + 1] };
            for i in 0..len {
                ctx.pv[base + 1 + i] = ctx.pv[(ply + 1) * MAX_PLY + i];
            }
            ctx.pv_len[ply] = 1 + len;
        }
        if v > alpha {
            alpha = v;
        }
        if alpha >= beta {
            cutoff = true;
            // Q14: this move refuted the node — killer + history credit, so
            // siblings and revisits try it first. Ordering data only; the
            // value returned above is already fixed.
            if ctx.use_kh && ply < MAX_PLY {
                if ctx.killers[ply][0] != Some(mv) {
                    ctx.killers[ply][1] = ctx.killers[ply][0];
                    ctx.killers[ply][0] = Some(mv);
                }
                ctx.history[mv.index()] = ctx.history[mv.index()]
                    .saturating_add((depth as i32) * (depth as i32));
            }
            break;
        }
    }
    // Store what this visit learned about the position. The best move orders
    // children at any depth; the value parts only mean something at depth 1,
    // where one factor serves every live child (see [`TtEntry`]).
    let mut entry = ctx.tt.remove(&key).unwrap_or_else(TtEntry::new);
    // Gated on `use_kh` so the ablation's OFF arm reproduces the tip exactly
    // (the field existed but was never populated before Q14).
    if ctx.use_kh && best_move.is_some() {
        entry.best = best_move;
    }
    if depth == 1 {
        entry.valued = true;
        entry.exact = !cutoff;
        entry.stalemate = false;
        entry.p_finish = p_finish;
        entry.p_live = p_live;
        entry.factor = live_factor;
    }
    ctx.tt.insert(key, entry);
    ctx.tt_stores += 1;
    best
}
