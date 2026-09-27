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
