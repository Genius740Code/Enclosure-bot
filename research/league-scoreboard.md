# League scoreboard (append-only; date, variant, margins, verdict)

## Baseline — 2026-09-27 (current `search.rs`: v3 + v4 terms, D10-F7 opener)

`probe_v3all` (v1/v2 round robin, 5 openings × 2 colors = 10 games each):
- v3 vs v1: **4/10**. As Blue 2-3 (W: None, 4864; L: 5589, 11723, 9199).
  As Red 2-3 (W: None, 11723; L: 4864, 5589, 9199).
  - v3 vs v1 open=None a=blue 1322-786 A WINS
  - v3 vs v1 open=None a=red 1109-865 A WINS
  - v3 vs v1 open=Some(4864) a=blue 1243-1237 A WINS
  - v3 vs v1 open=Some(4864) a=red 1131-1218 b wins
  - v3 vs v1 open=Some(5589) a=blue 1256-1745 b wins
  - v3 vs v1 open=Some(5589) a=red 1218-1219 b wins
  - v3 vs v1 open=Some(11723) a=blue 1637-1806 b wins
  - v3 vs v1 open=Some(11723) a=red 1343-1259 A WINS
  - v3 vs v1 open=Some(9199) a=blue 1415-2120 b wins
  - v3 vs v1 open=Some(9199) a=red 1541-1578 b wins
- v3 vs v2: **7/10** (matches lineage claim).
`probe_league` (scoutbase, 8 games): AVG margin **-20.0%** (ret perspective).
Genuine games (skip=0) both wins: blue +29.4%, red +38.1%.
Collapse line persists: skip=10/20 red -111.2%, skip=30 red -111.4%
(single 30+ pop ~t=60 + erosion — needs depth or vulnerability pricing).
`gauge` (greedy, 6 games): **6/6** (deterministic repeat of 2 lines).

## Experiment V5-1 — patience-off ablation (Blue-chair first-close timing) — 2026-09-27
Hypothesis: PATIENCE_PENALTY (no sub-2.0 closes before action 12) misfires on
forced openings as Blue, delaying closes v1 snatches.
Change: `PATIENCE_PENALTY` 2.0 -> 0.0 in `retaliator/src/search.rs`.
Result: byte-identical lines in every gate — v3all v1 4/10 / v2 7/10,
league -20.0%, gauge 6/6. Term never flips a pick in any measured line.
Verdict: **REJECTED (no effect)**. Reverted to 2.0 with provenance comment.

## Experiment V5-2 — Lane A deep search: negamax alpha-beta, depth 3 (`search_deep.rs`) — 2026-09-28
Hypothesis: fixed 3-ply NAIVE deepening scored -58% league; real alpha-beta with
move ordering (turn-aware negation, ordered children, beyond-horizon extension
along the line, no doom discount) fixes it. Evaluation, D10-F7 opener and root
priority verbatim from `search.rs`, so the h2h isolates the search itself.
Change: `pub mod search_deep;` (lib.rs, one-line addition) + `search_deep.rs`
(lane-a-search state capture; one compile fix: an unbound `outcome` at line 543;
one exact optimization: `value`'s draw check short-circuits at the first legal
move instead of generating them all — 596 positions checked, 0 mismatches vs
engine movegen, ~2-3x faster: t=12 67s vs 214s, t=24 571s vs 991s wall).
Result: **NO GATE COMPLETED** — h2h W/L n/a, league margin n/a, gauge n/a.
h2h game 1 (deep=blue, open=None) left running at action 60-80/120, ~2.1 hrs CPU,
game ~50% done; league (skip 0,30) killed at ~40%; gauge killed early.
Cost (bench): ~10k-16k nodes/move, ~10-30s CPU/move mid-game rising to ~165-450s
late; a full game ~3-6 hrs CPU = 1.5-22x OVER the site's ~20s/move budget. The
brief's "1000x time headroom" is falsified for depth 3.
Verdict: **REJECTED (over budget; W/L unmeasurable on this box)**. The exact
draw-check optimization is kept on lane-a-search for future search work; depth
needs a time-budget/iterative-deepening layer (outside Lane A scope) or a much
faster box before these gates can run.

## Experiment V7-S-control — Lane S budgeted deployment config (tip 667661b + probe) — 2026-09-28
Config: `search_deep::best_move_capped(depth 3, Budget::Ms(2000), DEFAULT_WIDTH 8/6)`
vs three gates (deployment line, NOT the Unbounded full-width form).
`probe_deep h2h_capped_budget` (vs shipped `search::best_move`, 5 openings x 2 colors = 10):
- **2/10** (blue 2/5, red 0/5). Red losses catastrophic (511-1646, 1648-4669,
  1011-4154, 381-1477, 589-4244); blue wins open=None 1243-727, 5589 1452-1246.
`probe_deep league_capped_budget` (vs scoutbase, skip 0/10/20/30 x 2 colors = 8):
- AVG margin **-96.4%** (ret perspective). Blue rows win (+37.0/+37.0/-27.7/+45.0);
  red rows collapse (-129.8/-221.9/-185.2/-325.7).
`probe_deep gauge_capped_budget` (vs greedy, 6): **6/6** (lines repeat byte-identically).
Verdict: control recorded. FAILS h2h (needs >=6/10) and league (>-9.9%, no row<-300).
Red-only collapse + full-window-correct code path implicates the aspiration window:
center passed Blue-relative into a mover-perspective window (wrong side as Red).
Next: V7-S0 aspiration-perspective fix, then Q14/Q16/Q6.

## Experiment V7-S0 — aspiration window mover-perspective fix — 2026-09-28
Root cause (found while confirming the control): `analyze_capped` passed the
previous depth's Blue-relative `evaluation` as the aspiration center, but the
root window lives in the mover's perspective — as Red the search opened on the
wrong side of the true value and burned the ~2s budget on widening re-searches
(delta 50->150->450->1350->4050->12150) instead of searching.
Change (search_deep.rs only): center = `sign(to_move) * evaluation` in both
budget arms; window-hit check compares the mover-perspective best value.
`probe_deep h2h_capped_budget` (10): **2/10** (blue 2/5, red 0/5 — same W/L as
control, changed lines: red losses 871-3469, 719-2251, 950-4293, 1482-4062,
924-4060; blue wins open=None 1243-727, 5589 1452-1246).
`probe_deep league_capped_budget` (8): AVG **-108.1%** (blue rows +37.0/+37.0/
-27.7/+45.0; red rows -149.1/-237.3/-177.5/-392.1).
`probe_deep gauge_capped_budget` (6): **6/6**.
Verdict: real bug, fixed — but NOT the red-collapse root cause (W/L unchanged).
Red still concedes 2000-4000 to the 2-ply baseline as Blue. Next: Q14 ordering.
