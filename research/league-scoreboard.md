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
