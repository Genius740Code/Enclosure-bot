//! Lane B (eval/phases): the shipped `search.rs` skeleton with the
//! cut+make-gap ablation result — the doom discount gated OFF in `best_move`.
//!
//! This file is the shipped `search.rs` skeleton (2-ply WIDTH-8, full-horizon
//! valuation, every V4/V5 term) with the tested terms kept behind flags, in a
//! separate file because eval lanes add here and the main session merges
//! gated winners into `search.rs` themselves. Copied verbatim, not imported:
//! the skeleton's items are private, and a shared copy would let one drift
//! from the other.
//!
//! **B-2 verdict (cut+make gap, 2026-09-28).** The doom discount prices the
//! enemy's worst one-action pop of our area at the full horizon (worst x 12),
//! but the re-make is shield-delayed two turns — the popped area misses
//! exactly TWO scoring events, not twelve. Ablation (n=8-10 both colors,
//! faithfulness control 0/239 first): doom OFF is better on league -9.9% vs
//! -20.0% (+10.1pp), v1 h2h 5/10 vs 4/10, collapse line -94.5% vs -111.2%
//! (+16.7pp — the term fails the very line it was added for), gauge 8/8
//! equal; only the as-Blue chair regresses (2/5 -> 1/5, n=5). `best_move`
//! runs with the doom OFF (the verified recommendation);
//! `baseline_best_move` keeps it ON (the shipped search, which `probe_b_h2h`
//! checks position for position). The vulnerability (VULN) and re-make
//! (REM) terms are rejected/never-fire — see their const docs.
//!
//! **B-3 term (farm-cycle routing, 2026-09-28).** C2's site autopsy
//! (`c-site-losses-2` H1): the farm cycles run 20-40 actions — far beyond
//! the shipped CUT_MEMORY 6 — and the farmed re-closes are close+cut
//! combis (16013 +4.5 re-popped 4x by 17112; 16970 x4) whose own_gain x
//! hz credit (54-198 full-horizon points at the observed gains) dwarfs
//! the flat rebuild penalty (3.0 x 12 = 36), so the shipped routing can
//! never flip them. The B-3 term scales the cut-ground penalty by the
//! SAME own_gain x hz (`REBUILD_GAIN_W`, gated by `Rebuild`); the avoid
//! set itself is the caller's, and the probes sweep its memory 12/20/40
//! against the shipped 6. Lane D proved the flat routing (mem 6) flips
//! the vlad-style farmer 6-2 -> 8-0; B-3 extends what it remembers and
//! what it prices. Numbers: `research/lane-b3-farm.md`.

use std::cmp::Reverse;

use meridian_engine::{Direction, Edge, IllegalMove, Move, MoveKind, Outcome, Player, Point, Position};

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
/// B-3 extends it with the gain-scaled penalty (`REBUILD_GAIN_W`, gated by
/// `Rebuild`) — the flat 3.0 x hz = 36 full-horizon points can never beat a
/// farmed re-close banking own_gain x hz (54-198).
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

/// Unbreakable-shape building (Lane E v7): rivals win by making walls that
/// share nodes along their length (2+ touches = no legal cut — v4 already
/// avoids cutting there). Mirror term: bonus for OUR first actions that
/// EXTEND shared-node walls / create 2+ touch thickets for us.
/// - A "shared node" = one of our nodes with degree >= 2 (a junction in our wall network).
/// - EXTENDING: the move's source is already a shared node (degree >= 2 before the move).
/// - CREATING: the move's target becomes a shared node (connects to 2+ existing own nodes).
/// Full-horizon units (area x min(events, 12)), deterministic tiebreak by move index.
const EXTEND_SHARED_W: f64 = 1.0;
const CREATE_SHARED_W: f64 = 1.0;

/// Cut-threat steal (rival-analysis #3: GB's cut bonus / spacebot's cut-threat
/// model): at each leaf, prefer moves that cut high-value enemy edges and
/// avoid leaving own big-loop edges exposed. The term scores the net cut
/// exposure: (enemy cuttable area) - (own cuttable area) for edges that are
/// legally cuttable in one action. Weighted by the area each edge shields.
/// Full-horizon units, applied at ranking for first actions.
const CUT_THREAT_W: f64 = 0.5;
/// Doom discount: area we hold that the enemy pops in one action is false
/// credit in the static eval (it counts area x 12 events as if it banks).
/// For each candidate, the worst one-action pop of our area, times the
/// capped horizon, is subtracted at selection. Doomed megaloops net to ~0,
/// so the bot builds split/remote/dense ground instead of one big grazeable
/// balloon. Computed only where the enemy is to move (their legal set).
/// B-2 ablation 2026-09-28 (ON=1.0 vs OFF=0.0, n=8-10 both colors,
/// faithfulness control 0/239 first): league -9.9% vs -20.0% (+10.1pp), v1
/// h2h 5/10 vs 4/10 (as-red 4/5 +9.2% vs 2/5 +3.6%), collapse line -94.5% vs
/// -111.2% (+16.7pp — the term fails the very line it was added for), gauge
/// 8/8 equal. Only regression: as-Blue v1 chair 2/5 -> 1/5 (-5.1pp, n=5
/// exact for the set). Verdict: RECOMMEND OFF — `best_move` runs with the
/// doom off (gated by the `doom` flag); `baseline_best_move` keeps it on
/// (the shipped search). Mechanism (cut+make gap): the re-make is
/// shield-delayed two turns, so the true doom is ~2-3 events, not 12 — see
/// REM_W's doc. Weight if re-enabled: 1.0. Dose response measured
/// 2026-09-28 (n=8-10 both colors per gate): NON-MONOTONE. league:
/// 0.0 -> -9.9% (best), 1.0 -> -20.0%, 0.5 -> -27.3% (worst). Collapse rows:
/// 0.0 -> -94.5/-94.5/-55.9 (best), 0.5 -> -137.7/-137.7/-104.7 (worst),
/// 1.0 -> -111.2/-111.2/-111.4. v1 h2h: 1.0 -> 4/10, 0.0 -> 5/10,
/// 0.5 -> 6/10 (the only configuration crossing the ship threshold; its wins
/// are narrow coin-flips: 4864-blue +0.5%, 5589-red +3.6%, and it loses
/// 9199-red -8.0% that 0.0 wins +5.2%). gauge 8/8 all doses. RECOMMEND 0.0
/// (league + collapse best, most robust); the 0.5 dose is the v1-threshold
/// alternative — the main session weighs the site threshold (plan-v4 Gate 2)
/// against the local league.
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

/// Gain-scaled close priority (Lane D H-B-VLAD1): a first-action close is
/// priced by its gained area against CLOSE_T, all game (no opening window —
/// the difference from PATIENCE). Closes below the threshold are tiny (they
/// bank ~0.2-0.9/close in the farmed losses while the mimic's greedy close
/// banks ~2-4) and lose (CLOSE_T - gain) x hz; closes at or above it gain
/// (gain - CLOSE_T) x hz, so a 0.5-area close never outranks a wall that
/// leads to a 3+ area close. Continuous in gain, full-horizon units
/// (area x min(events, 12)), deterministic tiebreak by move index. Applied
/// to our candidates only (first-action ranking + selection), like
/// REBUILD/FRESH; the opponent's reply model is the shipped skeleton's.
/// CLOSE_B2 sweep (doom OFF, CLOSE_W=1.0, n=8-10 both colors per gate,
/// faithfulness control re-verified first): see research/lane-b2-close.md.
const CLOSE_T: f64 = 3.0;
const CLOSE_W: f64 = 1.0;

/// B-3 farm-cycle routing: the gain-scaled part of the anti-rebuild
/// penalty. The flat `REBUILD_PENALTY` is 3.0 x hz = 36 full-horizon points
/// at hz 12, but the farmed re-close it must beat banks own_gain x hz —
/// 54-198 points for the site gains (+4.5 re-close 16013 popped 4x by
/// 17112; +9.0..+10.5 re-close 16970 x4; c-site-losses-2 H1) — so the flat
/// penalty can never flip the pick (36 < 54). This weight scales the
/// penalty by the SAME own_gain x hz on cut ground, so a re-close there
/// nets ~zero capped-area credit and any fresh-ground move with a
/// positive net outranks it (C2's confirm target). Full-horizon units,
/// deterministic tiebreak by move index, legality via the engine only.
/// The farm cycles run 20-40 actions — far beyond the shipped CUT_MEMORY
/// 6 — so the avoid set's memory is the caller's concern (B-3 sweeps
/// 12/20/40 there, `B3_MEM` in the gate probes). `Rebuild::ScaledAll`
/// also prices BREAKING re-closes (the C2 farmed re-closes are close+cut
/// combis, exempt under the shipped form); the flat part keeps its
/// non-breaking domain, so pure counter-cuts (gain 0) stay free. Dose 1.0
/// = exact cancellation of the capped-area credit; the beyond-horizon
/// extension credit (0.5 x max(0, events-12)) is left in place and
/// measured.
///
/// **Dose response (both forms measured; see `REBUILD_FULL` and
/// `research/lane-b3-farm.md`):** the capped factor (form A) does not
/// flip the mid-game act-51 confirm target (the re-close keeps its
/// beyond-horizon credit, 4.5 x 0.5 x ~23 ~= 52 points) but is the
/// gate-best dose; the full credit factor (form B) flips BOTH C2
/// targets but loses every gate — early-game the full factor is 3x the
/// capped one (36 vs 12 at 60 events left), and ~30 cuts/game of normal
/// play means the penalty starves ordinary consolidation, not just the
/// farm. Neither dose reaches v1 >= 6/10; the disease needs
/// discrimination (per-edge cut COUNTERS, C2 H1's original form), not
/// only a bigger penalty — the avoid slice cannot carry counts.
const REBUILD_GAIN_W: f64 = 1.0;

/// Whether the scaled penalty uses the full credit factor
/// `min(E,12) + 0.5 x max(0, E-12)` (form B) or only the capped `min(E,12)`
/// (form A — the task-literal "own_gain x hz"). Both doses measured
/// (research/lane-b3-farm.md): form B flips BOTH C2 confirm targets
/// (act-51 + act-112) but loses every gate catastrophically (league -25.4%
/// at mem 6, v1 3/10; -70.7%/2-3/10 at deeper memories — the E>12 factor is
/// 3x the capped one early-game, poisoning normal consolidation); form A
/// does NOT flip the mid-game act-51 target (the re-close keeps its
/// beyond-horizon credit) but is the gate-best dose: league +1.2% at mem 6
/// (+11.1pp vs the -9.9% control), collapse line fixed to -270/-270/-306
/// (vs -1038/-1038/-472), gauge 8/8 blue +76.0 — the cost is v1 4/10 vs
/// 5/10. Left at `false` (form A) so the gate-best config is reproducible
/// from HEAD; flip to `true` only to reproduce the form-B rows.
const REBUILD_FULL: bool = false;

/// The full-horizon factor the eval pays a close's gained area at:
/// the capped static part plus the horizon extension's beyond-cap part
/// (0.5 weight), i.e. exactly what `evaluate` + `horizon_extension`
/// credit for `gain` area.
fn credit_factor(events_left: u8) -> f64 {
    let capped = f64::from(events_left).min(HORIZON);
    let beyond = (f64::from(events_left) - capped).max(0.0);
    capped + 0.5 * beyond
}

/// The B-3 farm-cycle routing form, gated per entry point (the flat
/// shipped routing is always on whenever `avoid` is non-empty).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Rebuild {
    /// Shipped routing only: flat penalty, non-breaking first actions.
    Off,
    /// Flat + gain-scaled penalty, keeping the shipped counter-cut
    /// exemption (non-breaking first actions only).
    ScaledExempt,
    /// Flat + gain-scaled penalty on every first action on cut ground,
    /// breaking included — the farm-cycle form: the C2 farmed re-closes
    /// are close+cut combis ([51] 16013, [113] 16970), exempt under the
    /// shipped form. Pure counter-cuts (gain 0) still pay nothing.
    ScaledAll,
}

/// Legal-cuts-only vulnerability (V4d2 field rule): the doom discount subtracts
/// the worst one-action pop; the term adds the EROSION GAP — the total area the
/// enemy can legally cut from the evaluated position BEYOND that worst pop, per
/// distinct edge, from their legal move set. The max prices only the single
/// worst pop; erosion across several cuttable edges over consecutive actions
/// (the 41-cut farming line) is unpriced by it. Zero when only one edge is
/// cuttable, so the term never disturbs single-cut positions. Units match the
/// discount it joins: area x min(events, 12), full-horizon points — no
/// invented units. Computed in the same single pass as the worst pop, so the
/// term adds no search cost.
/// VULN ablation 2026-09-27 (ON=1.0 vs OFF=0.0, n=8-10 both colors per gate,
/// faithfulness control 0 mismatches / 239 positions first):
/// v1 h2h 3/10 vs 4/10 (as-blue avg -24.8% vs -11.6%), league -25.5% vs
/// -20.0% (baseline exact), gauge 8/8 vs 8/8 (blue margin +54.7% vs +66.0%).
/// Collapse line NOT fixed (skip=10/20 red -111.2% -> -120.7%). WORSE on
/// every gate. Verdict: REJECTED — left at 0.0 (the shipped search, which the
/// OFF control reproduces position-for-position). 0.5 untested; trend
/// uniformly negative, dose unlikely to flip it.
const VULN_W: f64 = 0.0;

/// Cut+make gap (the gap between the enemy's cut and our re-make) — EXPLORED,
/// NOT SHIPPED (2026-09-27/28). The doom discount prices the enemy's worst
/// one-action pop of our area at the full horizon (worst x 12), but a popped
/// loop is re-made only after a two-turn shield delay: the pop's own placed
/// edge crosses the re-place path and is fresh/shielded on our next turn, so
/// the re-place is illegal until the shield drops — the popped area misses
/// exactly TWO scoring events, not twelve. Two implementations of the
/// re-make-aware correction never fired: (1) checking the re-place from the
/// post-pop position always fails SourceNotOwned (it is the enemy's move
/// there); (2) checking it after every legal reply always fails
/// BreaksShieldedEdge (the pop's placed edge always crosses the path). The
/// correct form needs the enemy's block tree over their remaining actions —
/// too heavy for an eval term. The ablation that DOES verify the mechanism:
/// the doom discount OFF entirely (DOOM_W 1.0 -> 0.0, n=8-10 both colors,
/// faithfulness control 0/239 first) — league -9.9% vs -20.0% (+10.1pp), v1
/// h2h 5/10 vs 4/10 (as-red 4/5 +9.2% vs 2/5 +3.6%), collapse line -94.5% vs
/// -111.2% (+16.7pp — the term fails the very line it was added for; the V4
/// note "1098 -> 1031 against" recorded this same regression), gauge 8/8
/// equal. Only regression: as-Blue v1 chair 2/5 -11.6% -> 1/5 -16.7% (n=5,
/// exact for the set). Verdict: RECOMMEND DOOM OFF — `best_move` runs with
/// the doom discount off (the verified recommendation); `baseline_best_move`
/// keeps it on (the shipped search, which the faithfulness probe checks
/// position for position). 0.5 dose untested on the current gates (V4 tested
/// it worse on the collapse line); the rig supports it (flip DOOM_W + the
/// doom flag) if the as-Blue regression confirms.
const REM_W: f64 = 1.0;

pub fn best_move(position: &Position) -> Option<Move> {
    best_move_with_avoid(position, &[])
}

/// Like [`best_move`], but steering clear of `avoid`: points near our loops
/// the enemy recently cut. Rebuilding there is how rebuilder-farming works
/// (41 cuts in one game); routing elsewhere denies the repeat cut. Same
/// contract as `search::best_move_with_avoid`, so a gated merge into lib.rs
/// keeps the anti-rebuild routing in `replay` wired.
pub fn best_move_with_avoid(position: &Position, avoid: &[Point]) -> Option<Move> {
    // The B-3 base config: the doom-OFF control (B-2's verified
    // recommendation) with every added term off — CLOSE OFF too (the B-2
    // sweep verdict: all four CLOSE_T doses lost to the close-off control;
    // HEAD carried the last swept state for reproducibility, B-3 restores
    // the control). The B-3 term is NOT on this path: it is gated per entry
    // point (`farm_best_move_with_avoid`), so the faithfulness control and
    // the gate defaults stay the published doom-OFF control, and this path
    // keeps the shipped routing semantics (flat penalty) for any avoid set
    // the caller passes.
    best_move_features(position, MOVE_BUDGET, avoid, false, false, false, false, Rebuild::Off, false, false)
}

/// The B-3 term under test: the shipped anti-rebuild routing plus the
/// gain-scaled cut-ground penalty in `form` (see `REBUILD_GAIN_W`). The
/// avoid set is the caller's; the gate probes sweep its memory 12/20/40
/// (`B3_MEM`) against the shipped 6.
pub fn farm_best_move_with_avoid(
    position: &Position,
    avoid: &[Point],
    form: Rebuild,
) -> Option<Move> {
    best_move_features(position, MOVE_BUDGET, avoid, false, false, false, false, form, false, false)
}

/// The shipped `search::best_move` exactly: the doom discount ON, no added
/// terms — the ablation control. `probe_b_h2h` checks it position for
/// position against this before trusting any game. (Not every probe uses it;
/// the gate keeps per-example dead-code quiet.)
#[allow(dead_code)]
pub fn baseline_best_move(position: &Position) -> Option<Move> {
    best_move_features(position, MOVE_BUDGET, &[], false, false, true, false, Rebuild::Off, false, false)
}

/// The CLOSE_B2 ablation control: doom OFF (the B-2 recommendation) with the
/// close priority OFF — the same doom setting as the sweep, so
/// `probe_b_close`'s pick-flip count and the control gates isolate the close
/// term alone.
#[allow(dead_code)]
pub fn ablation_best_move(position: &Position) -> Option<Move> {
    best_move_features(position, MOVE_BUDGET, &[], false, false, false, false, Rebuild::Off, false, false)
}

/// The skeleton's selection loop, with the doom discount and the added terms
/// gated. The `Analysis`/`Candidate` reporting surface is not copied: probes
/// read the pick, and the main session merges the term itself.
fn best_move_features(
    position: &Position,
    budget: usize,
    avoid: &[Point],
    vuln: bool,
    remake: bool,
    doom: bool,
    close: bool,
    rebuild: Rebuild,
    unbreakable: bool,
    cut_threat: bool,
) -> Option<Move> {
    if let Some(open) = blue_opener(position) {
        return Some(open);
    }
    let mover = position.to_move();
    let opp = mover.opponent();
    let budget = budget.clamp(SMALLEST_BUDGET, MOVE_BUDGET);
    let first_actions = ranked(position, budget / 2, true, avoid, close, rebuild, unbreakable, cut_threat);
    let nodes = first_actions.len();
    let width = first_actions.len().min(WIDTH);
    let reply_budget = (budget - nodes) / width.max(1);

    // (adjusted score for ordering, the move it belongs to).
    let mut scored: Vec<(f64, Move)> = Vec::new();
    for (mv, after, _, _) in first_actions.into_iter().take(width) {
        let replies = ranked(&after, reply_budget, false, &[], false, Rebuild::Off, false, false);
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
            let tgt = mv.target().expect("legal moves end on the board");
            let on_cut = near_points(avoid.iter().copied(), tgt, CUT_RADIUS);
            if oc.broken.is_none() {
                if on_cut {
                    adjusted -= REBUILD_PENALTY * hz_sel;
                }
                if near_fresh_enemy(position, tgt) {
                    adjusted -= FRESH_PENALTY * hz_sel;
                }
                // Gain-scaled close priority at selection (Lane D H-B-VLAD1,
                // gated): the same term as in ranking, so the pick itself
                // weighs closes by gained area x hz against CLOSE_T.
                if close && oc.kind == MoveKind::Connect {
                    let gain_sel =
                        probe.area(mover).to_f64() - position.area(mover).to_f64();
                    adjusted += CLOSE_W * (gain_sel - CLOSE_T) * hz_sel;
                }
            }
            // B-3 farm-cycle routing at selection (gated): the same
            // gain-scaled term as in ranking, so the pick itself flips the
            // farmed re-close off cut ground. `ScaledAll` also prices the
            // breaking re-closes (the close+cut combis the site losses are
            // made of); the flat part above keeps its non-breaking domain.
            if rebuild != Rebuild::Off
                && on_cut
                && (oc.broken.is_none() || rebuild == Rebuild::ScaledAll)
            {
                let gain_sel = probe.area(mover).to_f64() - position.area(mover).to_f64();
                let factor = if REBUILD_FULL {
                    credit_factor(position.scoring_events_left())
                } else {
                    hz_sel
                };
                adjusted -= REBUILD_GAIN_W * gain_sel.max(0.0) * factor;
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
                let (worst, cuttable, worst_mv) = pop_cut_remake(pos, mover);
                if doom {
                    adjusted -= DOOM_W * worst * hz_doom;
                }
                // The (rejected) vulnerability term: the erosion gap — area the
                // enemy can LEGALLY cut BEYOND the single worst pop, which no
                // max ever prices. Zero when only one edge is cuttable. Same
                // full-horizon units, same single pass as the max.
                // Ablation 2026-09-27: WORSE on every gate (v1 3/10 vs 4/10,
                // league -25.5% vs -20.0%, gauge blue -11.3pp). Left off.
                if vuln {
                    adjusted -= VULN_W * (cuttable - worst) * hz_doom;
                }
                // The cut+make gap: the doom discount prices the enemy's worst
                // one-action pop at the full horizon, but the popped area we
                // can legally re-close in one action is restored next turn —
                // worth ONE scoring event, not twelve. Add back the
                // over-discount for that part. Zero when the re-make is
                // illegal (the pop was part of a surrounding attack): the doom
                // discount then stands at full strength.
                if remake {
                    let remakeable = remake_area(pos, worst_mv, mover);
                    adjusted += REM_W * remakeable * (hz_doom - 1.0);
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

/// The worst one-action pop of `victim`'s area, the total area the enemy
/// can legally cut from `pos` (enemy to move), counted once per distinct cut
/// edge, and the move that caused the worst pop. One pass over the enemy's
/// legal set: legal cuts only, so a dense cluster — any edge through it
/// touches 2+ opposing edges, which is illegal (one cut per move max) — and
/// shielded edges price zero. No fantasy cuts, no geometric touches (V4d2).
/// The worst pop is exactly the shipped `max_pop`; the cuttable sum is the
/// (rejected) vulnerability term's widening of it; the worst pop's move feeds
/// the cut+make gap term.
fn pop_cut_remake(pos: &Position, victim: Player) -> (f64, f64, Option<Move>) {
    let held = pos.area(victim).to_f64();
    let mut seen: Vec<Edge> = Vec::new();
    let mut worst: f64 = 0.0;
    let mut cuttable: f64 = 0.0;
    let mut worst_mv: Option<Move> = None;
    for mv in pos.legal_moves().iter() {
        let mut next = pos.clone();
        let outcome = next.apply_unchecked(mv);
        let loss = held - next.area(victim).to_f64();
        if loss > worst {
            worst = loss;
            worst_mv = Some(mv);
        }
        if let Some(cut) = outcome.broken {
            if !seen.contains(&cut) {
                seen.push(cut);
                cuttable += loss;
            }
        }
    }
    (worst.max(0.0), cuttable.max(0.0), worst_mv)
}

/// The part of the worst one-action pop that `victim` can legally re-close in
/// one action (the re-make): apply the enemy's worst pop, then require the
/// re-place of the cut edge to be legal after EVERY legal reply the enemy
/// still has — the pop was their first action of the turn, their reply comes
/// before `victim` moves again, and any reply that blocks the re-place (a
/// fresh edge across its path, or a second cut changing the geometry) kills
/// it; the enemy picks such a reply whenever it is best. Both endpoints of
/// the cut edge are still `victim`'s nodes, and the edge is not shielded (it
/// was ours, not their placement), so the blockers are the engine's own
/// (`check_move` from the post-reply position: breaks-several, fresh-shield,
/// overlaps). The re-make restores the loop next turn, so the popped area is
/// worth one scoring event, not the horizon. Zero when the re-place can be
/// blocked — the doom discount then stands at full strength.
fn remake_area(pos: &Position, pop: Option<Move>, victim: Player) -> f64 {
    let Some(mv) = pop else { return 0.0 };
    let mut after_pop = pos.clone();
    let outcome = after_pop.apply_unchecked(mv);
    let loss = pos.area(victim).to_f64() - after_pop.area(victim).to_f64();
    if loss <= 0.0 {
        return 0.0;
    }
    let Some(cut) = outcome.broken else { return 0.0 };
    let Some(re) = Move::between(cut.origin(), cut.far()) else { return 0.0 };
    for reply in after_pop.legal_moves().iter() {
        let mut after_reply = after_pop.clone();
        after_reply.apply_unchecked(reply);
        if after_reply.check_move(re).is_err() {
            return 0.0;
        }
    }
    loss
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
    close: bool,
    rebuild: Rebuild,
    unbreakable: bool,
    cut_threat: bool,
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
                // Unbreakable-shape building (Lane E v7): extend shared-node walls /
                // create 2+ touch thickets. A "shared node" = our node with degree >= 2.
                // EXTENDING: source is already a shared node (degree >= 2 before the move).
                // CREATING: target becomes a shared node (will connect to 2+ existing own nodes).
                if unbreakable {
                    let src_deg = node_degree(position, mover, mv.source);
                    let tgt_deg_after = node_degree(&after, mover, target);
                    if src_deg >= 2 {
                        priority += EXTEND_SHARED_W * hz;
                    }
                    if tgt_deg_after >= 2 {
                        priority += CREATE_SHARED_W * hz;
                    }
                }
                // Cut-threat steal (rival-analysis #3: GB's cut bonus / spacebot):
                // net cut exposure after our move = (enemy cuttable area) - (own cuttable area).
                // Prefer moves that reduce our exposure and/or increase enemy exposure.
                if cut_threat && outcome.broken.is_none() {
                    let our_exposure = cut_exposure(&after, mover);
                    let opp_exposure = cut_exposure(&after, opp);
                    priority += CUT_THREAT_W * (opp_exposure - our_exposure) * hz;
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
                // B-3 farm-cycle routing (gated by `rebuild`): the flat
                // penalty above is 36 full-horizon points at hz 12, but the
                // farmed re-close it must beat banks own_gain x hz
                // (54-198 for the site gains) — it can never flip the pick.
                // Scale the penalty by the same own_gain x hz so a
                // cut-ground re-close nets ~zero area credit and any
                // fresh-ground move with a positive net outranks it.
                // `ScaledAll` also prices breaking re-closes (the farmed
                // re-closes are close+cut combis); the flat part keeps its
                // non-breaking domain, so pure counter-cuts (gain 0) stay
                // free.
                if rebuild != Rebuild::Off
                    && (outcome.broken.is_none() || rebuild == Rebuild::ScaledAll)
                    && near_points(avoid.iter().copied(), target, CUT_RADIUS)
                {
                    let factor = if REBUILD_FULL {
                        credit_factor(position.scoring_events_left())
                    } else {
                        hz
                    };
                    priority -= REBUILD_GAIN_W * own_gain.max(0.0) * factor;
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
                // Gain-scaled close priority (Lane D H-B-VLAD1, gated): a
                // close is priced by its gained area against CLOSE_T, all
                // game. Tiny closes (the farmed 0.2-0.9/close) lose
                // (CLOSE_T - gain) x hz; big closes gain (gain - CLOSE_T) x
                // hz, so a 0.5-area close never outranks a wall that leads to
                // a 3+ area close. Continuous in gain; full-horizon units.
                if close && outcome.kind == MoveKind::Connect {
                    priority += CLOSE_W * (own_gain - CLOSE_T) * hz;
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

/// Degree of `player`'s node at `point` (number of incident edges).
fn node_degree(position: &Position, player: Player, point: Point) -> u32 {
    position
        .edges(player)
        .iter()
        .filter(|edge| edge.origin() == point || edge.far() == point)
        .count() as u32
}

/// Cut exposure for `player` at `pos` (player to move is the opponent who can cut).
/// Sum of area shielded by each legally cuttable edge of `player` from the
/// opponent's legal move set. An edge is "cuttable" if the opponent has a
/// legal move that breaks it (and it's not shielded).
fn cut_exposure(pos: &Position, player: Player) -> f64 {
    let mut seen: Vec<Edge> = Vec::new();
    let mut exposure: f64 = 0.0;
    for mv in pos.legal_moves().iter() {
        let mut next = pos.clone();
        let outcome = next.apply_unchecked(mv);
        if let Some(cut) = outcome.broken {
            if !seen.contains(&cut) {
                seen.push(cut);
                // The cut edge belongs to `player` (the victim).
                // Area lost by this cut = pos.area(player) - next.area(player).
                // But we want the area this specific edge shields.
                // Approximation: the area loss from this cut, assuming it's the only one.
                let loss = pos.area(player).to_f64() - next.area(player).to_f64();
                exposure += loss.max(0.0);
            }
        }
    }
    exposure
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

// =====================================================================
// Lane B phase-aware terms (2026-10-01): pure functions + probe support.
//
// The mission terms, all read-only over `&Position` in the f64 Blue-lead
// convention: phase gates, one-move loop-closing potential (ALL closable
// pairs, not just `loop_bonus`'s triangles), mobility, legal-cuts-only
// cuttable-edge counts, frontier/node differentials. Nothing below is
// called by the rig above; the terms enter the bot only through
// `replay_adjusted` + `probe_eval` (external re-ranking of
// `search::analyze`'s candidates, one term at a time), per the mission's
// "call search::analyze and recompute ordering with your terms" option.
// The KEY question — the true banked-score swing of cut+make vs
// make-only lines, and whether the beyond-horizon denial weight (shipped:
// symmetric 0.5) is right — is answered by `probe_eval`'s cutline mode,
// whose pricing model (leaf value + `destroyed x beyond-horizon events
// x 0.5`) lives there beside the playouts it prices.
// =====================================================================

/// Phase gates: opening before this many actions, endgame from this many
/// scoring events left, middlegame in between (the mission's gates; the
/// opening gate matches PATIENCE_WINDOW).
pub const OPENING_ACTIONS: u8 = 12;
pub const ENDGAME_EVENTS: u8 = 12;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Opening,
    Middlegame,
    Endgame,
}

impl Phase {
    pub fn index(self) -> usize {
        match self {
            Phase::Opening => 0,
            Phase::Middlegame => 1,
            Phase::Endgame => 2,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Phase::Opening => "opening",
            Phase::Middlegame => "middlegame",
            Phase::Endgame => "endgame",
        }
    }
}

pub fn phase(position: &Position) -> Phase {
    if position.actions_played() < OPENING_ACTIONS {
        Phase::Opening
    } else if position.scoring_events_left() <= ENDGAME_EVENTS {
        Phase::Endgame
    } else {
        Phase::Middlegame
    }
}

/// The same board with `player` to move: edges and scores frozen, the
/// action parity flipped (one action at a turn start, two mid-turn, none
/// at the last action), shielded/fresh cleared. A static hypothetical —
/// "what could each side do if it were their turn now" — never a played
/// position; `Position::setup` allows exactly that (recent edges AtMost).
/// Clearing the shields only ever ADDS enemy cuts, so cut counts taken
/// here are the honest "could they, from this board" reads.
fn as_if_to_move(position: &Position, player: Player) -> Option<Position> {
    if position.is_finished() {
        return None;
    }
    if position.to_move() == player {
        return Some(position.clone());
    }
    let a = position.actions_played();
    let flip = u16::from(a) + if a % 2 == 0 { 1 } else { 2 };
    let blue: Vec<Edge> = position.edges(Player::Blue).iter().collect();
    let red: Vec<Edge> = position.edges(Player::Red).iter().collect();
    Position::setup(
        &blue,
        &red,
        flip as u8,
        [position.score(Player::Blue), position.score(Player::Red)],
        &[],
    )
    .ok()
}

/// One-move loop-closing potential: the distinct pairs of `player`'s
/// nodes that a single LEGAL action could connect (king-3 apart, legal
/// by the engine's own `check_move`). `loop_bonus` prices only the
/// triangles hanging off the move's source; this counts every closable
/// pair — any legal connect closes some cycle and banks from the next
/// scoring event on.
pub fn closable_pairs(position: &Position, player: Player) -> usize {
    let Some(probe) = as_if_to_move(position, player) else { return 0 };
    let nodes: Vec<Point> = probe.nodes(player).iter().collect();
    let mut count = 0usize;
    for (i, &a) in nodes.iter().enumerate() {
        for &b in &nodes[i + 1..] {
            if let Some(mv) = Move::between(a, b) {
                if probe.check_move(mv).is_ok() {
                    count += 1;
                }
            }
        }
    }
    count
}

/// Mobility: the player's legal-move count, by the engine's own
/// movegen — the real set when it is their turn, the frozen-board
/// hypothetical when it is not.
pub fn mobility(position: &Position, player: Player) -> usize {
    as_if_to_move(position, player).map_or(0, |probe| probe.legal_moves().len())
}

/// Speed cap for `cuttable_edges`: enemy moves tried per read. The
/// engine's legal order (source, then direction) makes the capped read
/// a deterministic prefix.
pub const CUTTABLE_CAP: usize = 512;

/// Distinct edges of `victim` the enemy can LEGALLY cut in one action
/// (one touch only — dense clusters and shielded walls price zero), by
/// playing the enemy's legal set on the enemy-to-move board and reading
/// the engine's own `broken` edge, once per distinct edge. The
/// count-based cousin of the rig's area-based (and rejected) VULN term.
pub fn cuttable_edges(position: &Position, victim: Player) -> usize {
    let Some(probe) = as_if_to_move(position, victim.opponent()) else { return 0 };
    let mut seen: Vec<Edge> = Vec::new();
    for mv in probe.legal_moves().iter().take(CUTTABLE_CAP) {
        let mut next = probe.clone();
        if let Some(cut) = next.apply_unchecked(mv).broken {
            if !seen.contains(&cut) {
                seen.push(cut);
            }
        }
    }
    seen.len()
}

/// Frontier: `player`'s nodes with at least one geometrically free
/// direction — on the board, not on either side's node, not strictly
/// inside the player's own edge. An "alive nodes" proxy (bases with
/// room to extend), computed without the full legality machinery.
pub fn frontier_nodes(position: &Position, player: Player) -> usize {
    let mut inside: Vec<Point> = Vec::new();
    for edge in position.edges(player).iter() {
        inside.extend(edge.interior_points());
    }
    let blocked = |p: Point| {
        position.nodes(Player::Blue).contains(p)
            || position.nodes(Player::Red).contains(p)
            || inside.contains(&p)
    };
    position
        .nodes(player)
        .iter()
        .filter(|&n| Direction::all().any(|d| n.step(d).is_some_and(|t| !blocked(t))))
        .count()
}

pub fn node_count(position: &Position, player: Player) -> usize {
    position.nodes(player).len()
}

/// The shipped blue opener, for probes that must play the exact shipped
/// move at action 1 before their own selection takes over.
pub fn opener(position: &Position) -> Option<Move> {
    blue_opener(position)
}

/// Everything the phase-aware terms read off one position. Per-player
/// counts in [Blue, Red] order; the differentials are Blue-lead.
pub struct TermSnapshot {
    pub actions_played: u8,
    pub events_left: u8,
    pub phase: Phase,
    pub area: [f64; 2],
    pub closable: [usize; 2],
    pub mobility: [usize; 2],
    pub cuttable: [usize; 2],
    pub frontier: [usize; 2],
    pub nodes: [usize; 2],
}

pub fn term_snapshot(position: &Position) -> TermSnapshot {
    TermSnapshot {
        actions_played: position.actions_played(),
        events_left: position.scoring_events_left(),
        phase: phase(position),
        area: [position.area(Player::Blue).to_f64(), position.area(Player::Red).to_f64()],
        closable: [closable_pairs(position, Player::Blue), closable_pairs(position, Player::Red)],
        mobility: [mobility(position, Player::Blue), mobility(position, Player::Red)],
        cuttable: [cuttable_edges(position, Player::Blue), cuttable_edges(position, Player::Red)],
        frontier: [frontier_nodes(position, Player::Blue), frontier_nodes(position, Player::Red)],
        nodes: [node_count(position, Player::Blue), node_count(position, Player::Red)],
    }
}

/// Probe-side term weights for `replay_adjusted`. Defaults reproduce the
/// shipped selection arithmetic exactly (claim = deny = 1, every count
/// term off); a term is "on" by giving it a nonzero weight.
#[derive(Clone, Copy)]
pub struct ProbeTerms {
    /// Beyond-horizon weight on own area gained along the line.
    pub claim_w: f64,
    /// Beyond-horizon weight on enemy area destroyed along the line
    /// (shipped: 1.0, i.e. the symmetric `HORIZON_WEIGHT` 0.5 on both).
    pub deny_w: f64,
    pub close_w: f64,
    pub mob_w: f64,
    pub vuln_edge_w: f64,
    pub node_w: f64,
    pub frontier_w: f64,
}

impl Default for ProbeTerms {
    fn default() -> Self {
        ProbeTerms {
            claim_w: 1.0,
            deny_w: 1.0,
            close_w: 0.0,
            mob_w: 0.0,
            vuln_edge_w: 0.0,
            node_w: 0.0,
            frontier_w: 0.0,
        }
    }
}

impl ProbeTerms {
    fn counts_on(&self) -> bool {
        self.close_w != 0.0
            || self.mob_w != 0.0
            || self.vuln_edge_w != 0.0
            || self.node_w != 0.0
            || self.frontier_w != 0.0
    }
}

/// The shipped selection score for one analyzed candidate, replayed from
/// `search::analyze`'s output, then extended by the [`ProbeTerms`]:
/// `sign(mover) * evaluation + horizon_extension` (with the claim/denial
/// split) + the selection-time fresh-wall poison − the doom discount,
/// + every count term on `end`. Must reproduce the shipped ordering of
/// `analyze`'s own candidates with default terms — `probe_eval` checks
/// that position for position (its fidelity line) before trusting any
/// ablation flip.
pub fn replay_adjusted(
    position: &Position,
    mv: Move,
    evaluation: f64,
    pv: &[Move],
    terms: &ProbeTerms,
) -> f64 {
    let mover = position.to_move();
    let opp = mover.opponent();
    let mut after = position.clone();
    let oc = after.apply_unchecked(mv);
    let end = position_after_pv(position, pv);

    // Sign*evaluation, then the beyond-horizon extension with the
    // claim/denial split (claim_w = deny_w = 1.0 is the shipped pricing).
    let gained = (end.area(mover).to_f64() - position.area(mover).to_f64()).max(0.0);
    let destroyed =
        (position.area(opp).to_f64() - end.area(opp).to_f64()).max(0.0);
    let events = f64::from(end.scoring_events_left());
    let beyond = events - events.min(HORIZON);
    let mut adjusted =
        sign(mover) * evaluation + (gained * terms.claim_w + destroyed * terms.deny_w) * beyond * HORIZON_WEIGHT;

    // Selection-time poison: fresh walls (the avoid/rebuild part never
    // fires on probe positions, which analyze with avoid = []).
    if oc.broken.is_none() {
        let hz_sel = f64::from(position.scoring_events_left()).min(HORIZON);
        if let Some(tgt) = mv.target() {
            if near_fresh_enemy(position, tgt) {
                adjusted -= FRESH_PENALTY * hz_sel;
            }
        }
    }

    // Doom discount: the shipped worst one-action pop (pop_cut_remake's
    // first component is exactly `search::max_pop`).
    let doom_at = if end.to_move() == opp {
        Some(&end)
    } else if after.to_move() == opp {
        Some(&after)
    } else {
        None
    };
    if let Some(pos) = doom_at {
        let hz_doom = f64::from(pos.scoring_events_left()).min(HORIZON);
        adjusted -= DOOM_W * pop_cut_remake(pos, mover).0 * hz_doom;
    }

    // The phase-aware count terms, all on `end` (the line's leaf).
    if terms.counts_on() {
        let own_close = closable_pairs(&end, mover);
        let (own_mob, opp_mob) = (mobility(&end, mover), mobility(&end, opp));
        let (own_cut, opp_cut) = (cuttable_edges(&end, mover), cuttable_edges(&end, opp));
        let (own_nodes, opp_nodes) = (node_count(&end, mover), node_count(&end, opp));
        let (own_front, opp_front) = (frontier_nodes(&end, mover), frontier_nodes(&end, opp));
        adjusted += terms.close_w * own_close as f64
            + terms.mob_w * (own_mob as f64 - opp_mob as f64)
            + terms.vuln_edge_w * (opp_cut as f64 - own_cut as f64)
            + terms.node_w * (own_nodes as f64 - opp_nodes as f64)
            + terms.frontier_w * (own_front as f64 - opp_front as f64);
    }
    adjusted
}
