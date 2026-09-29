//! Retaliator's strategy: Scout-shaped two-action search with three valuation fixes.
//!
//! The search skeleton (ranked first actions, searched replies, horizon-capped
//! static evaluation) follows the site's own Scout bot. The fixes come from our
//! self-play evidence:
//!
//! 1. **Full-horizon valuation.** The static evaluation counts enclosed area
//!    only up to `HORIZON` scoring events, which underprices both big land
//!    claims and loop-breaking retaliation early in the game (up to 3x).
//!    Both the candidate ranking and the final pick add the uncapped
//!    remainder for *both* sides symmetrically: own area gained and enemy
//!    area destroyed, times the events past the horizon. A pure break-first
//!    1-ply bot went 6-0 against our old greedy valuation, so denial counts
//!    in full — but claims count exactly as much, so the bot takes land
//!    instead of chasing cuts for their own sake.
//! 2. **Contact avoidance.** A first action that ends next to an enemy node
//!    without cutting anything just donates a target (the neck/aggro family
//!    lives on this). Penalized.
//! 3. **Dead-wood discount.** A reinforcement between two own nodes that adds
//!    no area, far from the enemy, is tempo burned in the back: the safe loop
//!    already banks every turn, and one cut zeroes a loop no matter how
//!    reinforced it looks. Penalized, so the bot expands instead.
//! 4. **Corridor-root contest (Lane W, M9/E4).** When the enemy builds a
//!    2-wall corridor (shared-node walls: 2+ edges meeting at a node =
//!    uncuttable), their area grows rapidly (E4: 3→71 in 10 moves). We detect
//!    this by tracking enemy shared-node count and area acceleration. The
//!    "corridor root" is the highest-shared node anchoring the wall. Contesting
//!    it early (before the wall shuts) is priced in the M1 exception ledger as
//!    a DENY move that must outbid the best area move. Single variable: the
//!    area-acceleration threshold (dose sweep ≤3 values).

use std::cmp::Reverse;
use std::sync::LazyLock;
use std::time::Instant;

use meridian_engine::{Move, MoveKind, Outcome, Player, Point, Position, notation};

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

/// Corridor-root contest (Lane W, M9/E4): detect enemy 2-wall progress and
/// contest the corridor root before it shuts. Single variable: the area-
/// acceleration threshold (dose sweep ≤3 values). E4 exhibit: 3→71 in 10
/// moves = ~6.8 area/move acceleration.
/// DOSE 1: 5.0 area/10 moves (conservative)
/// DOSE 2: 6.8 area/10 moves (E4 exact)
/// DOSE 3: 8.5 area/10 moves (aggressive)
const CORRIDOR_ACCEL_THRESHOLD: f64 = 6.8; // DOSE 2: E4 exact threshold (best)
/// Minimum shared-node count to qualify as a 2-wall corridor node.
const CORRIDOR_SHARED_MIN: u32 = 2;
/// Radius around corridor root to consider as contest targets.
const CORRIDOR_ROOT_RADIUS: i8 = 2;
/// Contest bonus multiplier (scales with horizon like other terms).
/// Must outbid best area move in M1 ledger: area move ~5 area * 30 events * 0.5 = 75 pts.
/// With severity up to 3x and hz up to 12, bonus=10 gives up to 360 pts.
const CORRIDOR_CONTEST_BONUS: f64 = 10.0;
/// Minimum enemy shared nodes to trigger corridor detection.
/// E4 corridor has many shared nodes (two parallel walls with cross-links).
/// Top bots build long corridors: GB builds ~12 wall actions before closing.
/// Set high to avoid false positives from normal wall-building.
const CORRIDOR_TRIGGER_SHARED: usize = 8;

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

/// Mesh opening prefix (Lane H mesh8, v6 candidate 2026-09-28): capybara
/// center mesh as forced prefix, 8 own-moves per color. Built with
/// Move::between (canonical king-step encoding), EXACTLY like the probe —
/// integer ids are NOT used for selection because Direction::index is not
/// injective (from_index can decode a different edge variant with the same
/// id; found 2026-09-28: from_index(4867) prints D10-G8 but fails
/// check_move where between(D10,G8) succeeds). h2h: v1 Blue 5-0/Red 4-1,
/// v2 5-0/5-0, scout Blue 5-0/Red 4-1, gauge 6/6, GB mega-loop 4.5.
/// League gate PENDING (H-verify) — do not ship rated on this alone.
/// Blue: D10-D13 D10-G8 G8-J10 J10-G13 G8-J11 G13-J11 G13-D10 J11-M13
const MESH_B_TXT: [[&str; 2]; 8] = [
    ["D10", "D13"],
    ["D10", "G8"],
    ["G8", "J10"],
    ["J10", "G13"],
    ["G8", "J11"],
    ["G13", "J11"],
    ["G13", "D10"],
    ["J11", "M13"],
];
/// Red mirror: P10-M8 M8-J10 J10-M13 M8-J11 M13-J11 M13-P11 M13-P10 P11-M8
const MESH_R_TXT: [[&str; 2]; 8] = [
    ["P10", "M8"],
    ["M8", "J10"],
    ["J10", "M13"],
    ["M8", "J11"],
    ["M13", "J11"],
    ["M13", "P11"],
    ["M13", "P10"],
    ["P11", "M8"],
];

fn decode_pair(pair: [&str; 2]) -> Move {
    let (a, b) = (pair[0], pair[1]);
    Move::between(
        notation::parse_square(a).expect("mesh prefix source square"),
        notation::parse_square(b).expect("mesh prefix target square"),
    )
    .expect("mesh prefix is 1-3 king steps")
}

fn decode_prefix(txt: [[&str; 2]; 8]) -> [Move; 8] {
    txt.map(decode_pair)
}

static MESH_B: LazyLock<[Move; 8]> = LazyLock::new(|| decode_prefix(MESH_B_TXT));
static MESH_R: LazyLock<[Move; 8]> = LazyLock::new(|| decode_prefix(MESH_R_TXT));

/// Turn structure (read off probe ply logs): Blue opens with a single action,
/// then sides alternate in PAIRS starting with Red: totals 1-2 Red, 3-4 Blue,
/// 5-6 Red, ... Never assume strict alternation.
fn side_to_move(total: usize) -> Player {
    if total == 0 {
        Player::Blue
    } else if ((total - 1) / 2) % 2 == 0 {
        Player::Red
    } else {
        Player::Blue
    }
}

/// Forced-prefix routing with probe Bot::forced semantics, stateless via
/// history. Our color is whoever is to move; our slots are the history
/// indices whose turn-side is ours (pair-turn aware, NOT parity). Onset: the
/// first own slot holding prefix[0] — earlier own slots may hold anything
/// (site-forced openings delay the prefix, never skip entries: the probe v1
/// off-by-one lesson). After onset, continuation must be exact; any
/// deviation (illegal entry stall, or a searched move in our slot) truncates
/// the prefix permanently, because history never un-deviates. The frontier
/// entry plays when legal, else search takes this turn (and the resulting
/// deviation truncates all later turns).
pub fn mesh_prefix(position: &Position, history: &[Move]) -> Option<Move> {
    let total = position.actions_played() as usize;
    if history.len() != total {
        return None;
    }
    let us = position.to_move();
    if side_to_move(total) != us {
        return None;
    }
    let prefix: &[Move; 8] = match us {
        Player::Blue => &MESH_B,
        _ => &MESH_R,
    };
    let slots: Vec<usize> = (0..total).filter(|&m| side_to_move(m) == us).collect();
    // Onset: first own slot holding prefix[0].
    let mut onset: Option<usize> = None;
    for (j, &m) in slots.iter().enumerate() {
        if history[m] == prefix[0] {
            onset = Some(j);
            break;
        }
    }
    let verified: usize = match onset {
        None => 0,
        Some(o) => {
            let mut len = 0;
            while o + len < slots.len()
                && len < prefix.len()
                && history[slots[o + len]] == prefix[len]
            {
                len += 1;
            }
            // Any own slot past onset+len that exists means deviation...
            // (only possible when the frontier was skipped, which never
            // happens: frontier plays whenever legal, else this call is None
            // and the search move written to history truncates next call).
            if o + len < slots.len() {
                return None;
            }
            len
        }
    };
    if verified >= prefix.len() {
        return None;
    }
    let mv = prefix[verified];
    position.check_move(mv).ok().map(|_| mv)
}

/// Deployed entry: mesh prefix, then the legacy game-start opener, then search.
/// Timed think (Q6-amendment, v7): searched moves spend at least THINK_SOFT_MS
/// (no snap replies), more while volatile, capped at THINK_HARD_MS.
/// Keeps [`best_move_with_avoid`] untouched for probes and back-compat.
pub fn best_move_routed(
    position: &Position,
    history: &[Move],
    avoid: &[Point],
) -> Option<Move> {
    if let Some(prefix) = mesh_prefix(position, history) {
        return Some(prefix);
    }
    if let Some(open) = blue_opener(position) {
        return Some(open);
    }
    analyze_timed(position, avoid)
        .candidates
        .first()
        .map(|candidate| candidate.mv)
}

/// How many of the latest actions count as "recent" for cut avoidance.
pub const CUT_MEMORY: usize = 6;
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
/// Doom discount: DISABLED (V5, 2026-09-28). Lane B ablations: the discount
/// prices the enemy's one-action pop at ×12, but the re-make is shield-delayed
/// two turns (the pop's placed edge crosses the re-place path and shields it),
/// so the popped area misses exactly two scoring events — ×12 overcharges by
/// ~6x and the bot avoids healthy ground. Measured in eval_phases rig (control
/// faithful 0/239): doom OFF = league -9.9% vs -20.0%, v1 h2h 5/10 vs 4/10,
/// collapse -55.9% vs -111.4%, gauge 8/8 equal; cost as-Blue 2/5 -> 1/5.
/// Doom discount: ENABLED (V6, 2026-09-28). V5 disabled it on rig evidence
/// (league -9.9% vs -20.0%) but the rig bypassed the avoid path and the site
/// verdict was catastrophic (v5: 1349 Elo, 4W-36L/40 vs v3 1477 13-11 and
/// v4 1428 11-21, both doom-ON). V6 league gate with mesh8: doom-ON +21.6%
/// worst -57.6% vs doom-OFF +18.3% worst -71.7% — ON wins locally too.
/// Restored to 1.0; the OFF dose lives in git history (v6-mesh variant A).
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

/// Reply budget for the confirming ply: sized for near-complete coverage
/// like the second ply. A truncated longest-first subset picks unrepresen-
/// tative replies and the minimax actively misleads (measured: -58%
/// league). Full coverage costs ~2x total nodes, still milliseconds.
pub struct Analysis {
    /// Blue's lead after the best move and its reply.
    pub evaluation: f64,
    /// 2 when replies were searched.
    pub depth: usize,
    /// Positions searched.
    pub nodes: usize,
    /// The best moves first.
    pub candidates: Vec<Candidate>,
}

pub struct Candidate {
    pub mv: Move,
    /// Blue's lead after this move and the best reply (honest static value;
    /// ordering uses the retaliation-adjusted score, kept separately).
    pub evaluation: f64,
    /// This move, its reply, and the confirming third ply where searched.
    pub pv: Vec<Move>,
    /// 1, plus the replies searched.
    pub visits: usize,
}

pub fn best_move(position: &Position) -> Option<Move> {
    best_move_with_avoid(position, &[])
}

/// Like [`best_move`], but steering clear of `avoid`: points near our loops
/// the enemy recently cut. Rebuilding there is how rebuilder-farming works
/// (41 cuts in one game); routing elsewhere denies the repeat cut.
pub fn best_move_with_avoid(position: &Position, avoid: &[Point]) -> Option<Move> {
    if let Some(open) = blue_opener(position) {
        return Some(open);
    }
    analyze_with_avoid(position, MOVE_BUDGET, avoid).candidates.first().map(|candidate| candidate.mv)
}

pub fn analyze(position: &Position, budget: usize) -> Analysis {
    analyze_with_avoid(position, budget, &[])
}

pub fn analyze_with_avoid(position: &Position, budget: usize, avoid: &[Point]) -> Analysis {
    let mover = position.to_move();
    let opp = mover.opponent();
    let budget = budget.clamp(SMALLEST_BUDGET, MOVE_BUDGET);
    let first_actions = ranked(position, budget / 2, true, avoid);
    let mut nodes = first_actions.len();
    let mut depth = usize::from(nodes > 0);
    let width = first_actions.len().min(WIDTH);
    // NB: budget < nodes is normal (300+ legal vs small budgets). checked_sub
    // preserves release behavior (full reply width) without debug underflow.
    let reply_budget = budget.checked_sub(nodes).map(|r| r / width.max(1)).unwrap_or(usize::MAX);

    // (adjusted score for ordering, candidate with honest static evaluation).
    let mut scored: Vec<(f64, Candidate)> = Vec::new();
    for (mv, after, _, _) in first_actions.into_iter().take(width) {
        let replies = ranked(&after, reply_budget, false, &[]);
        nodes += replies.len();
        let mut pv = vec![mv];
        let mut evaluation = value(&after);
        if let Some((reply, _, reply_value, _)) = replies.first() {
            evaluation = *reply_value;
            pv.push(*reply);
            depth = 2;
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
        let visits = 1 + replies.len();
        // Doom discount (see const docs).
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
                adjusted -= DOOM_W * max_pop(pos, mover) * hz_doom;
            }
        }
        scored.push((adjusted, Candidate { mv, evaluation, pv, visits }));
    }
    scored.sort_by(|a, b| {
        b.0.total_cmp(&a.0).then(a.1.mv.index().cmp(&b.1.mv.index()))
    });
    let candidates = scored.into_iter().map(|(_, c)| c).collect::<Vec<_>>();
    let evaluation = candidates.first().map_or_else(|| value(position), |best| best.evaluation);
    Analysis { evaluation, depth, nodes, candidates }
}

/// Timed think (Q6-amendment, v7): every searched move spends at least
/// THINK_SOFT_MS, extending to THINK_HARD_MS while the position is volatile
/// (top two candidates disagree). Site envelope (bot-api.md): aim for
/// limits.moveTimeMs = 5000ms; until 2026-10-04 moves stop at ~23s browser /
/// 25s server, FROM that date a move past the limit FAILS (forfeit). HARD
/// stays under 5000 with margin. WASM clock via wasi clock_time_get.
/// Depth is real minimax, not budget inflation: measured 2026-09-29, full-width
/// 2-ply costs ~11ms/2436 nodes at 373 legal moves and wider visit budgets
/// change nothing (saturated). So extra time goes into beam deepening — each
/// round extends every live line one ply through the same `ranked()` reply
/// model, re-sorted by the same selection adjustments as `analyze_with_avoid`
/// (horizon extension, doom discount, rebuild/fresh penalties); only the leaf
/// is deeper. Solved lines (terminal) stop early: exact needs no more time.
/// Forced moves (0-1 legal), prefix and opener return instantly.
/// Machine-speed-dependent depth, NOT probe-deterministic: probes and the
/// analysis endpoint keep fixed node budgets via `analyze_with_avoid`.
pub const THINK_SOFT_MS: u64 = 2000;
pub const THINK_HARD_MS: u64 = 4800;
/// Mover-relative adjusted gap below which the lead is undecided: keep
/// thinking to the hard cap.
const VOLATILE_GAP: f64 = 2.0;
/// Beam width for timed deepening (matches WIDTH).
const BEAM: usize = 8;

struct BeamLine {
    mv: Move,
    after: Position,
    pv: Vec<Move>,
    end: Position,
    leaf_blue: f64,
    adjusted: f64,
    visits: usize,
    solved: bool,
}

/// Selection value mirroring `analyze_with_avoid`: mover-relative honest leaf
/// plus horizon extension, minus rebuild/fresh penalties and the doom
/// discount (evaluated at the deepest opp-to-move position available).
fn selection_adjusted(
    position: &Position,
    after: &Position,
    end: &Position,
    leaf_blue: f64,
    mover: Player,
    opp: Player,
    mv: Move,
    avoid: &[Point],
) -> f64 {
    let mut adjusted = sign(mover) * leaf_blue + horizon_extension(position, end, mover);
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
    let doom_at = if end.to_move() == opp {
        Some(end)
    } else if after.to_move() == opp {
        Some(after)
    } else {
        None
    };
    if let Some(pos) = doom_at {
        let hz_doom = f64::from(pos.scoring_events_left()).min(HORIZON);
        adjusted -= DOOM_W * max_pop(pos, mover) * hz_doom;
    }
    adjusted
}

fn sort_beam(beam: &mut [BeamLine]) {
    beam.sort_by(|a, b| {
        b.adjusted.total_cmp(&a.adjusted).then(a.mv.index().cmp(&b.mv.index()))
    });
}

pub fn analyze_timed(position: &Position, avoid: &[Point]) -> Analysis {
    let mover = position.to_move();
    let opp = mover.opponent();
    let legal_count = position.legal_moves().len();
    // Forced: nothing to decide (prefix/opener handled by the caller).
    if legal_count == 0 {
        return Analysis { evaluation: value(position), depth: 0, nodes: 0, candidates: vec![] };
    }
    let t0 = Instant::now();
    let firsts = ranked(position, legal_count, true, avoid);
    let mut nodes = firsts.len();
    let mut beam: Vec<BeamLine> = Vec::new();
    for (mv, after, _, _) in firsts.into_iter().take(BEAM) {
        let replies = ranked(&after, after.legal_moves().len(), false, &[]);
        nodes += replies.len();
        let (mut pv, _) = (vec![mv], after.clone());
        let mut leaf = value(&after);
        if let Some((reply, rafter, _, _)) = replies.first() {
            pv.push(*reply);
            leaf = value(rafter);
        }
        let end = position_after_pv(position, &pv);
        let adjusted = selection_adjusted(position, &after, &end, leaf, mover, opp, mv, avoid);
        let solved = end.is_finished();
        let visits = 1 + replies.len();
        beam.push(BeamLine { mv, after, pv, end, leaf_blue: leaf, adjusted, visits, solved });
    }
    sort_beam(&mut beam);
    if beam.iter().all(|l| l.solved) {
        return finish_timed(position, beam, nodes);
    }
    loop {
        for line in beam.iter_mut().filter(|l| !l.solved) {
            if line.end.is_finished() {
                line.solved = true;
                continue;
            }
            let opts = ranked(&line.end, line.end.legal_moves().len(), false, &[]);
            nodes += opts.len();
            match opts.first() {
                Some((rmv, rafter, _, _)) => {
                    line.pv.push(*rmv);
                    line.end = rafter.clone();
                    line.leaf_blue = value(&line.end);
                    line.visits += opts.len();
                    if line.end.is_finished() {
                        line.solved = true;
                    }
                }
                None => line.solved = true,
            }
            line.adjusted =
                selection_adjusted(position, &line.after, &line.end, line.leaf_blue, mover, opp, line.mv, avoid);
            if t0.elapsed().as_millis() as u64 >= THINK_HARD_MS {
                break;
            }
        }
        sort_beam(&mut beam);
        let elapsed = t0.elapsed().as_millis() as u64;
        if beam.iter().all(|l| l.solved) {
            break; // solved: exact, nothing more to learn
        }
        if beam.len() < 2 {
            break; // nothing to choose between — save the clock (cf. Lane T)
        }
        if elapsed >= THINK_HARD_MS {
            break;
        }
        if elapsed >= THINK_SOFT_MS
            && (beam[0].adjusted - beam[1].adjusted).abs() >= VOLATILE_GAP
        {
            break; // decided, soft budget spent
        }
    }
    finish_timed(position, beam, nodes)
}

fn finish_timed(position: &Position, beam: Vec<BeamLine>, nodes: usize) -> Analysis {
    let depth = beam.iter().map(|l| l.pv.len()).max().unwrap_or(0);
    let candidates = beam
        .into_iter()
        .map(|l| Candidate { mv: l.mv, evaluation: l.leaf_blue, pv: l.pv, visits: l.visits })
        .collect::<Vec<_>>();
    let evaluation = candidates.first().map_or_else(|| value(position), |best| best.evaluation);
    Analysis { evaluation, depth, nodes, candidates }
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

/// The worst our-area loss across the enemy's legal replies from `pos`
/// (enemy to move). The one-graze cost of our shape.
fn max_pop(pos: &Position, victim: Player) -> f64 {
    let held = pos.area(victim).to_f64();
    let mut worst: f64 = 0.0;
    for mv in pos.legal_moves().iter() {
        let mut next = pos.clone();
        next.apply_unchecked(mv);
        worst = worst.max(held - next.area(victim).to_f64());
    }
    worst.max(0.0)
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
                // Corridor-root contest (Lane W, M9/E4): if enemy is building a
                // 2-wall corridor (high shared nodes + area acceleration), contest
                // the corridor root before it shuts. Priced in M1 ledger as DENY
                // that must outbid best area move. Single variable: threshold.
                if first && outcome.broken.is_none() {
                    let contest_bonus = corridor_contest_bonus(position, mover, opp, target, hz);
                    priority += contest_bonus;
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

/// Count shared nodes for a player: nodes where 2+ edges meet (uncuttable wall nodes).
/// This is the key detector for 2-wall corridors (E4: shared-node walls).
fn shared_node_count(position: &Position, player: Player) -> u32 {
    use std::collections::HashMap;
    let mut degree: HashMap<usize, u32> = HashMap::new();
    for edge in position.edges(player).iter() {
        for end in [edge.origin(), edge.far()] {
            *degree.entry(end.index()).or_insert(0) += 1;
        }
    }
    degree.values().filter(|&&d| d >= CORRIDOR_SHARED_MIN).count() as u32
}

/// Find the corridor root: the enemy node with highest shared-edge count (most
/// anchoring the wall). Returns None if no qualifying corridor nodes exist.
fn corridor_root(position: &Position, opp: Player) -> Option<Point> {
    use std::collections::HashMap;
    let mut degree: HashMap<usize, u32> = HashMap::new();
    for edge in position.edges(opp).iter() {
        for end in [edge.origin(), edge.far()] {
            *degree.entry(end.index()).or_insert(0) += 1;
        }
    }
    degree
        .into_iter()
        .filter(|(_, d)| *d >= CORRIDOR_SHARED_MIN)
        .max_by_key(|(_, d)| *d)
        .map(|(idx, _)| Point::from_index(idx).expect("valid point"))
}

/// Check if enemy is building a corridor: very high shared-node count BUT
/// area still low (corridor not yet closed). This is the PRE-CLOSURE threat
/// window. E4: at move 40 (actions~80), shared nodes high, area only 3.0.
/// At move 50, corridor closes, area jumps to 71. We must contest during
/// moves 40-50. Strict filters to avoid false positives on normal walls.
fn is_corridor_threat(position: &Position, opp: Player) -> bool {
    let shared = shared_node_count(position, opp);
    // Require very high shared nodes: only substantial corridors (two long
    // parallel walls with cross-links) reach this. Normal walls: 2-4 shared.
    if shared < 8 {
        return false;
    }
    // Threat heuristic: high shared nodes but area NOT yet banked.
    // Corridor building phase: shared nodes high, area low for action count.
    let actions = position.actions_played() as f64;
    let area = position.area(opp).to_f64();
    let threshold = actions * CORRIDOR_ACCEL_THRESHOLD / 10.0;
    if area >= threshold {
        return false; // Corridor already closed, too late
    }
    // Additional filter: corridor root must be in a threatening position.
    // The root should be advancing toward our side, not retreating.
    let Some(root) = corridor_root(position, opp) else {
        return false;
    };
    let mover = position.to_move();
    // For Blue (mover), enemy is Red. Red's corridor root threatening us
    // would be on Blue's side (negative x for Blue starting at D10=-6,0).
    // For Red (mover), enemy is Blue. Blue's corridor root threatening us
    // would be on Red's side (positive x for Red starting at P10=6,0).
    let threatening = match mover {
        Player::Blue => root.x() <= -2, // Red corridor reaching toward Blue
        Player::Red => root.x() >= 2,   // Blue corridor reaching toward Red
    };
    threatening
}

/// Contest bonus for moves targeting the corridor root. Returns the bonus
/// (scaled by horizon) if the move contests the corridor, 0 otherwise.
fn corridor_contest_bonus(
    position: &Position,
    _mover: Player,
    opp: Player,
    target: Point,
    hz: f64,
) -> f64 {
    if !is_corridor_threat(position, opp) {
        return 0.0;
    }
    let Some(root) = corridor_root(position, opp) else {
        return 0.0;
    };
    // Contest if target is near the corridor root
    if (target.x() - root.x()).abs() <= CORRIDOR_ROOT_RADIUS
        && (target.y() - root.y()).abs() <= CORRIDOR_ROOT_RADIUS
    {
        // Must outbid best area move: bonus scales with threat severity
        let shared = shared_node_count(position, opp) as f64;
        let severity = (shared / CORRIDOR_TRIGGER_SHARED as f64).min(3.0);
        CORRIDOR_CONTEST_BONUS * severity * hz
    } else {
        0.0
    }
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
