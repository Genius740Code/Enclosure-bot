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
//!   extension, correct at any depth.

use std::cmp::Reverse;

use meridian_engine::{Move, MoveKind, Outcome, Player, Point, Position};

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
    let drawn = matches!(position.outcome_given(&position.legal_moves()), Some(Outcome::Draw(_)));
    if drawn { 0.0 } else { evaluate(position) }
}

/// How many of `player`'s nodes hang by a single edge (capturable).
/// Fixed array instead of a HashMap: same counts, no allocation — this runs
/// at every node of the deep search.
fn one_edge_nodes(position: &Position, player: Player) -> u32 {
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
fn repeats_a_connection(position: &Position, mv: Move) -> bool {
    let target = mv.target().expect("legal moves end on the board");
    position.nodes(position.to_move()).contains(target) && target < mv.source
}

fn length(mv: Move) -> i8 {
    mv.direction.dx().abs().max(mv.direction.dy().abs())
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
}

impl Ctx {
    fn new() -> Ctx {
        Ctx {
            nodes: 0,
            pv: vec![Move::from_index(0).expect("move 0 exists"); MAX_PLY * MAX_PLY].into_boxed_slice(),
            pv_len: [0; MAX_PLY],
        }
    }

    fn line(&self, ply: usize) -> Vec<Move> {
        self.pv[ply * MAX_PLY..ply * MAX_PLY + self.pv_len[ply]].to_vec()
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
    let mover = position.to_move();
    let opp = mover.opponent();
    let mut ctx = Ctx::new();
    let root = ranked(position, avoid);
    let mut scored: Vec<(f64, DeepCandidate)> = Vec::with_capacity(root.len());
    let mut alpha = f64::NEG_INFINITY;

    for Ranked { mv, after, priority, .. } in root {
        let before_nodes = ctx.nodes;
        // The root's accumulated extension is zero (no plies above), so the
        // child's value at depth 1 is exactly its priority.
        let (v, child_pv) = if after.is_finished() || depth <= 1 {
            ctx.pv_len[1] = 0;
            (priority, Vec::new())
        } else {
            let swing = (after.area(mover).to_f64() - position.area(mover).to_f64()).max(0.0)
                + (position.area(opp).to_f64() - after.area(opp).to_f64()).max(0.0);
            let child_ext = sign(mover) * swing;
            let v = if after.to_move() == mover {
                // Same player's second action: the window never flips.
                negamax(&after, depth - 1, 1, alpha, f64::INFINITY, child_ext, &mut ctx)
            } else {
                -negamax(&after, depth - 1, 1, -f64::INFINITY, -alpha, child_ext, &mut ctx)
            };
            let child_pv = ctx.line(1);
            (v, child_pv)
        };
        let mut pv = vec![mv];
        pv.extend(child_pv);
        // Poisoned ground counts at SELECTION, not just ranking: a line that
        // rebuilds where we were just cut (or contests a fresh wall) loses
        // here even if its honest eval is best — the cut history can see what
        // the eval can't. (Same terms as the baseline's selection.)
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
        scored.push((adjusted, DeepCandidate { mv, evaluation: sign(mover) * v, pv, visits }));
        if v > alpha {
            alpha = v;
        }
    }

    scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.mv.index().cmp(&b.1.mv.index())));
    let candidates = scored.into_iter().map(|(_, c)| c).collect::<Vec<_>>();
    let evaluation = candidates.first().map_or_else(|| value(position), |best| best.evaluation);
    DeepAnalysis { evaluation, depth: usize::from(depth), nodes: ctx.nodes, candidates }
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
        return sign(mover) * value(pos) + sign(mover) * ext * extension_factor(events);
    }
    let legal: Vec<Move> =
        pos.legal_moves().iter().filter(|&mv| !repeats_a_connection(pos, mv)).collect();
    if legal.is_empty() {
        ctx.pv_len[ply] = 0;
        // The game is a draw: the static part is 0, the line's extension counts.
        let events = f64::from(pos.scoring_events_left());
        return sign(mover) * ext * extension_factor(events);
    }

    // Order the children: mover-signed 1-ply value + horizon extension — the
    // core of the root priority function. The first-action penalties are
    // root-only (tuned there); depth sees their consequences anyway. A
    // connection between own nodes is deduped by `repeats_a_connection`, so
    // no position is searched twice.
    let mut children: Vec<(f64, Move, f64, f64, bool)> = Vec::with_capacity(legal.len());
    for mv in legal {
        let mut after = pos.clone();
        after.apply_unchecked(mv);
        let val = value(&after);
        let gained = (after.area(mover).to_f64() - pos.area(mover).to_f64()).max(0.0);
        let destroyed =
            (pos.area(mover.opponent()).to_f64() - after.area(mover.opponent()).to_f64()).max(0.0);
        let factor_c = extension_factor(f64::from(after.scoring_events_left()));
        let finished = after.is_finished();
        let priority = sign(mover) * val + (gained + destroyed) * factor_c;
        // A finished child's leaf has 0 events left: the extension vanishes.
        let leaf_factor = if finished { 0.0 } else { factor_c };
        let swing = gained + destroyed;
        children.push((priority, mv, leaf_factor, swing, finished));
    }
    children.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.index().cmp(&b.1.index())));

    let mut best = f64::NEG_INFINITY;
    for (priority, mv, leaf_factor, swing, finished) in children {
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
            break;
        }
    }
    best
}
