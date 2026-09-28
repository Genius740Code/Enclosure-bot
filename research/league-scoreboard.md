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

## Experiment B-1 — legal-cuts-only vulnerability term (V4d2 erosion gap) — 2026-09-27
Variant: `eval_phases` vuln ON (VULN_W=1.0). Faithfulness control first:
term OFF == shipped `search.rs`, 0 mismatches in 239 positions; OFF league
reproduces baseline -20.0% exactly; OFF v1-h2h games match baseline scores.
| variant | v1 W/L | league margin % | gauge W/L | verdict |
|---|---|---|---|---|
| eval_phases vuln ON (VULN_W=1.0) | 3/10 (blue 1/5 avg -24.8%, red 2/5 avg -7.3%) | -25.5% | 8/8 (blue +54.7%, red +69.0%) | **REJECT** |
| control = shipped (VULN_W=0.0) | 4/10 (blue 2/5 avg -11.6%, red 2/5 avg +3.6%) | -20.0% (baseline exact) | 8/8 (blue +66.0%, red +69.3%) | baseline |
Note: WORSE on every gate, incl. the as-Blue chair (-13.2pp avg). Collapse
line NOT fixed (skip=10/20 red -111.2% -> -120.7%). Erosion gap prices
cuttable area at full horizon weight on every candidate; ~25 cuts/side/game
is normal play, so it discounts all big-area claims — small-loop habit.
`VULN_W` left at 0.0 (only verified terms in the rig). Numbers:
`research/lane-b-vuln.md`. 0.5 dose untested (trend uniformly negative).

## Experiment B-2 — cut+make gap math / doom-discount ablation — 2026-09-28
Mechanism verified in the engine: the doom discount prices the enemy's
one-action pop at x12, but the re-make is shield-delayed two turns (the
pop's placed edge crosses the re-place path and shields it), so the popped
area misses TWO scoring events, not twelve. Two re-make-aware corrections
never fired (SourceNotOwned / BreaksShieldedEdge always block); the correct
form needs the enemy's block tree — too heavy for an eval term.
| variant | v1 W/L | league margin % | gauge W/L | verdict |
|---|---|---|---|---|
| eval_phases doom OFF (DOOM_W=0.0) | **5/10** (blue 1/5 -16.7%, red 4/5 +9.2%) | **-9.9%** | 8/8 (blue +53.5, red +69.3) | **RECOMMEND (merge candidate)** |
| shipped = doom ON (control) | 4/10 (blue 2/5 -11.6%, red 2/5 +3.6%) | -20.0% | 8/8 (blue +66.0, red +69.3) | baseline |
Note: doom OFF better on league (+10.1pp), v1 (+1 win, as-red +5.6pp),
collapse lines (+16.7pp skip=10: -94.5% vs -111.2%; +55.5pp skip=30: -55.9%
vs -111.4%), gauge equal. The doom discount FAILS the line it was added for
(V4 note "1098 -> 1031 against" recorded the same regression). Only
regression: as-Blue chair 2/5 -> 1/5 (n=5 exact). Faithfulness control
0/239 positions (re-verified after each restructure). Numbers:
`research/lane-b-gap.md`. RE off.
Dose test (0.5): v1 **6/10** (only config crossing the ship threshold;
as-blue chair 2/5 kept) BUT league **-27.3%** (worst) and collapse rows
**-137.7%** (worst) — non-monotone, no config dominates. 0.5 = the
v1-threshold alternative for the main session; 0.0 = the league-best
default. 0.25 untested.

## Experiment B-3 — farm-cycle detector + scaled rebuild routing (C2 H1) — 2026-09-28
Hypothesis (c-site-losses-2 H1): farm cycles run 20-40 actions, far beyond
CUT_MEMORY 6, and the flat rebuild penalty (3.0 x hz = 36 full-horizon
points) can never flip a farmed re-close banking own_gain x hz (54-198);
the farmed re-closes are close+cut combis, exempt under the shipped
non-breaking condition. Term (eval_phases.rs, gated `Rebuild` +
`REBUILD_GAIN_W=1.0`): scale the cut-ground penalty by own_gain x credit
factor, in ranking AND selection; the gate probes feed OUR avoid set live
(lib.rs replay semantics, memory generalized, `B3_MEM`/`B3_SCALE`).
Faithfulness control re-verified 0/239 after every restructure; the
no-avoid control reproduced the doom-OFF rows byte-for-byte. All 9
treatment configs on all 3 gates; `research/lane-b3-farm.md` has the
full matrix + confirm-probe evidence.
| variant | v1 W/L | league margin % | gauge W/L | verdict |
|---|---|---|---|---|
| control = doom OFF, no avoid | 5/10 (blue 1/5 -16.7%, red 4/5 +9.2%) | -9.9% (collapse diffs -1038/-1038/-472) | 8/8 (blue +53.5) | baseline |
| mem 12/20/40 flat routing | 4/10 each | -6.8 / -9.3 / -11.7% (collapse -901/-901/-664 all depths) | 8/8 each | REJECT (v1 -1 at every depth; league non-monotone) |
| ScaledAll form B, mem 6/12/20/40 | 3/2/3/2 of 10 | -25.4 / -31.8 / -47.6 / **-70.7%** | 8/8 each | REJECT (monotone-catastrophic in memory; flips BOTH C2 confirm targets but starves normal consolidation: ~30 cuts/game is field-normal) |
| **ScaledAll form A (own_gain x hz), mem 6** | **4/10** (blue 1/5 -30.4%, red 3/5 -6.0%) | **+1.2%** (collapse **-270/-270/-306**) | **8/8 (blue +76.0)** | REJECT as merge, KEEP as documented near-miss |
Note: form A at the shipped memory is the session's one big positive —
league +11.1pp, the collapse line FIXED (two of three rows inside the
-300 target), gauge blue +22.5pp — but v1 4/10 vs 5/10 (as-red 11723
flips on a -36 diff; as-blue margins worse) fails the no-regression bar,
and it does not flip the mid-game act-51 site target (the farmed
re-close keeps its beyond-horizon credit; form B flips both targets and
loses every gate). Root cause for the next lane: the avoid slice cannot
carry COUNTS — one recent cut and a 4x-re-popped cycle price the same;
C2 H1's original per-edge cut-counter form discriminates them (needs an
API change). Confirm harness: `probe_b_farm` (deployed fidelity 60/60 on
446956a1; act-51/act-113 checks; whole-game scorecard).

## Verification + landing — Lane B final state re-verified on `lane-b-eval` — 2026-09-28
Provenance: the lane's work lived on `lane-v7-eval` through the 429 death
(worktree game-e); the main session rescued the final delta as `e4b7ec5`
("Unreviewed, NOT gated"); this continuation fast-forwarded `lane-b-eval`
to that commit and re-verified every headline gate on this box before
pushing — all reproduce the recorded numbers byte-for-byte:
- faithfulness control **0/239** positions (probe_b_h2h, control vs shipped)
- gauge **8/8** (blue +53.5, red +69.3 — exact)
- v1 h2h **5/10** (blue 1/5 -16.7%, red 4/5 +9.2% — exact, probe_b_v3all)
- league **-9.9%** (collapse rows -94.5/-94.5/-55.9 — exact; genuine blue
  +29.0, red +39.8)
NEW evidence (the gap the lane left): **v2 h2h 6/10** (blue 1/5 -6.9%, red
5/5 sweep +23.3%) via `probe_b_v3all` (the plan's v1/v2 gate) — doom OFF
beats v1 +1 (4->5, crosses the plan-v4 ship threshold) and gives back -1
on v2 (7->6); the as-Blue chair from forced openings stays 0/4 vs v1 (the
open problem, unchanged by every term this lane tested).
| variant | league avg margin | gauge W-L | one-line verdict |
|---|---|---|---|
| lane-b-eval final (doom OFF, VULN 0.0, Rebuild off) | **-9.9%** (shipped control -20.0%) | **8/8** | **VERIFIED merge candidate: remove the doom discount (+10.1pp league, v1 5/10, collapse +16.7/+55.5pp, v2 6/10, chair cost 2/5->1/5); vulnerability/re-make/mobility/shield-expiry/phase terms all REJECTED or SKIPPED with numbers — main session gates the search.rs merge** |
