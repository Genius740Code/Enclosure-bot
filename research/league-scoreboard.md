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

## Lane C autopsies — blue chair + red collapse (analysis only, no src changes) — 2026-09-28
Full catalog: `research/c-blunder-catalog.md`. Probes: `probe_bluechair` (fixed
opener-response test), `probe_balloon`. Reproduced the 2026-09-27 baseline
byte-for-byte (all 8 measured finals).
| C:bluechair v3-vs-v1 forced 5589 | L 1256-1745 | divergence [54] M1-K4 +13.8, eval −205→−428 | Red's double-duty cut-closes unpriced (10+ of Red's closes 54-118 also cut us) |
| C:bluechair v3-vs-v1 forced 11723 | L 1637-1806 | divergence [49] F7-E8 pops −37.3, eval +207→−288 | close-survival blindness: doomed closes credited full area×12 |
| C:bluechair v3-vs-v1 forced 9199 | L 1415-2120 | divergence [34]-[38] cut blitz, eval +25→−75 | rebuild-farming: REBUILD_PENALTY 36 < 54-198 needed; worthless cut paid a bonus at [40] |
| C:bluechair opener-response (fixed) | 1/4 (9199 W) | flips 2 of 5 in opposite directions (4864 W→L, 9199 L→W) | first-response placement is first-order; no fixed response dominates |
| C:balloon collapse (skip=10/20/30) | −111% | single 30+ pop DEAD (maxEnemyPop 0.0 everywhere, doom discount works); farm cycle remains: same-triangle re-closes +13.5 vs identical re-pops | routing never chosen; skip=10 ≡ skip=20 (league double-counts one game) |
Shared finding: losses are area-lifetime asymmetry (opponents hold 20-118 area alive for
dozens of turns; our area is recyclable), NOT cut volume (the None win suffered 41 cuts).
Handed to Lane A/B: hypotheses #1-4 in the catalog (enemy-release term, per-close survival
pricing, scaled rebuild penalty + break-bonus weighting, farm-cycle detector). Also: every
local gate measures the bot WITHOUT its deployed avoid routing (§3.6) — fix the gate.
