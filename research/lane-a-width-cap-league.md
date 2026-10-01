# Lane A: width-capped deep search — league gate closed (2026-10-01)

Continuation of `research/lane-a-search-deep.md` (deleted at `15a0923` when the
unbounded depth-3 probe was rejected). This note closes the open item that
commit recorded: **the width-capped league gate**.

## What exists (recovered + completed)

`retaliator/src/search_deep.rs` restored from `0b1c0ba` (Lane A2(d)) onto
branch `lane-a-search`. The architecture, per plan-v3 §2:

1. **Negamax alpha-beta over actions.** The eval is a Blue-lead number; the
   mover's view is its sign times the eval. Negation happens only ACROSS
   turns — a turn is 1-2 actions and the same player often moves twice, so
   within a turn the mover maximizes and the window never flips.
2. **Transposition table**, keyed by the canonical `Position` hash through
   `DefaultHasher` (fixed keys, deterministic across runs). Entries are
   depth-1 nodes: their value is `max(p_finish, p_live + sign(mover) x ext x
   factor)` — the path-independent parts plus the caller's accumulated line
   extension, exact for any path. Cleared per move, so a call's result never
   depends on what earlier calls searched.
3. **Iterative deepening under a `Budget`**: `Ms` (native-CPU clock via
   /proc, soft — a new depth starts only under the cap, a running depth stops
   between root children), `Nodes` (deterministic), `Unbounded` (the gates).
   Deeper iterations' candidates are kept; shallower ones fill gaps
   (`merge_analyses`).
4. **Width caps** `Width{root,inner}` — DEFAULT 8/6, both priority-ordered
   with the move-index tiebreak, the TT's best move enters the root beam.
   This is THE fix for the cost disease.
5. **Easy paths**: one legal move returns instantly; the budgeted root stops
   between children once the cap is spent.
6. **Eval + root priority verbatim** from `search.rs` (v3+v4 terms, D10-F7
   opener, all penalties in full-horizon points), engine-only legality
   (`has_legal_move` — the exact short-circuit draw check, verified 596/596
   against engine movegen at commit `2fc206b`).

Why the width caps are the load-bearing piece (the DISCOVERY in `a897bb5`):
turn-start roots hand their same-mover ply a +INF beta, so nothing prunes and
their depth-3 tree is ~b^2 leaf-parent nodes — 27 CPU-minutes for 8 games
(0 completed). Capping the root at 8 and interiors at 6 makes depth 3
0.00–0.06s/move wall on this box, 300x under the 20s site limit.

## Numbers (all gates, n=8 minimum, both colors)

Gate config: `Budget::Unbounded` + `DEFAULT_WIDTH` + depth 3 — deterministic.

| Gate | Result | Baseline | Verdict |
|---|---|---|---|
| league (scoutbase, 8 games) | AVG **-8.2%** | -20.0% | **PASS** (bar AVG > -9.9%; worst row -76.4% > -300%) |
| league as Blue | +28.7% avg, 3W/1L | +29.4% genuine | holds |
| league as Red | -45.0% avg, 1W/3L | +38.1% genuine | **weak** (1-2 unique lines) |
| gauge (greedy, 6 games) | **6/6** (+69.8/+73.0) | 6/6 | no regression |
| h2h vs `search::best_move` (8 games) | **6/8** (blue 3/4 +15.0%, red 3/4 +14.3%) | — | deep beats its baseline |
| cost (benchcap, 20 positions) | 0.00–0.06s/move | 20ms (2-ply) | 300x headroom |

League detail (margin, ret perspective): blue +48.5, +48.5, -5.5, +23.1;
red +15.0, -76.4, -76.4, -42.1. The skip=10/20 red rows are the SAME converged
line (identical scores 996-1758), so the Red sample is 3 unique lines, not 4.

The catastrophic collapse line improved -111.2% -> -76.4% but persists —
depth alone (this file, no vulnerability pricing) narrows it, does not kill it.

## Verdict

**MERGE-READY (search core).** Depth pays once it comes from real alpha-beta
with ordering and width caps at turn-start roots — the lineage verdict (naive
deepening -58%/-44%) stands, and the unbounded-width form stays dead on cost.
The deep search beats the 2-ply baseline head-to-head 6/8, holds the league
bar, and keeps gauge at 6/6.

Open follow-ups, in order of expected value:
1. **Red-side league weakness** (-45.0% avg, 1-2 unique lines vs scoutbase).
   The collapse line needs Lane B vulnerability pricing (V4b shield-expiry
   threats) or deeper search at the same width — not more weight terms.
2. **Stashed variant** (`stash@{0}`): my TT+ID+timed rewrite with a unified
   root-relative leaf extension (`ext_rl`) and two-tier lazy ordering
   (HEAD_WIDTH=8 + on-demand tail). Same ideas, different shape; the
   `0b1c0ba` form is the one the gate validated, so it is the merge
   candidate. The stash is kept for the ext_rl accounting (path-independent,
   exact at any depth) if the merge wants it.
3. **Deeper depth under `Budget::Ms`** on the deployment line (2000ms native)
   — `best_move_with_budget` exists and is value-exact; ungated.

Main session merges only gated winners; I do not touch `search.rs`/`lib.rs`.
Lane files committed on `lane-a-search` and pushed.
