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

## 2026-09-28 — V5 (doom OFF port, DOOM_W 1.0 -> 0.0) — main session
Port of Lane B's rig result into search.rs (one constant + provenance comment).
Rig numbers: league -9.9% vs -20.0%, v1 5/10 vs 4/10, collapse -55.9% vs -111.4%.
Port gate: gauge 6/6 (deterministic repeat of 2 lines, margins +53.5%/+69.3%).
Caveats: ties (not beats) v1; as-Blue 2/5 -> 1/5; rig ran without avoid path.
Verdict: SHIP (best measured config; site Elo decides).

## 2026-09-28 — V6 (mesh8 prefix + doom-ON) — UPLOADED
Port of Lane H mesh8 (pair-turn slots, onset scan, permanent truncation,
between()-built moves) + doom discount restored to 1.0 (site lineage:
v3 1477/v4 1428 doom-ON vs v5 1349 4-36 doom-OFF).
Gates (on port): league_mesh +21.6% worst -57.6% (bar: >-9.9%, no row <-300);
gauge_mesh 6/6 (+74-75%); v1/v2/scout h2h probe-side doom-ON base
(v1 B5-0/R4-1, v2 5-0/5-0, scout B5-0/R4-1); mesh_verify + site_check green
(opening reply 16058, 12 legal site-path replies, analysis depth 2).
Site bot "Riposte v6" 853d805d-8aaa-4061-a0bf-816fedd3e4c9 (fresh provisional),
version 93cbc665b8ab5b885983bec02d2969917f421ed916d3a23a0581095616a8cedd (290011 bytes).
Rated evals queued pairs=1: v5 (6ea13d3e), v4 (a68f64de), GB (daf1be99),
GB2.0 (5e7dbc71), VladNet (ddd08831), AngelBot (0b6220f8). Site forces
action-1 per pair: prefix onset-scan delays (inj may read 7/8 there).
Verdict: PENDING site Elo.

## V7 directive — timed think (Q6-amendment, user order, NOT gated) — 2026-09-29
User directive: v7 must spend ~2s per searched move, more while volatile.
Change (`retaliator/src/search.rs`, master): `best_move_routed` now uses
`analyze_timed` — beam deepening (BEAM=8) through the same `ranked()` reply
model and selection adjustments (horizon/doom/rebuild/fresh); SOFT=2000ms
minimum, HARD=8000ms cap, VOLATILE_GAP=2.0 top-2 extension. Prefix/opener and
forced moves stay instant. Probes/analysis keep fixed node budgets.
Measurement (native, mid-game 373 legal): full 2-ply = 11ms/2436 nodes
(saturated — wider budgets change nothing); timed = 2029ms/depth-73/264613
nodes, best move legal. `site_check` OK (opening 16058, 12/12 legal, 15.5s).
Caveats: depth is machine-speed-dependent (routed-path gauge determinism no
longer holds); strength UNGATED — league/h2h numbers pending. WASM rebuild
required before upload.

## 2026-09-29 — V7 UPLOADED ("Riposte v7" 0c63610e-23a4-4a41-a701-723cdb1853f7)
Engine = v6 (mesh8 + doom-ON, same valuation) + timed think: routed moves spend
2000ms soft / 4800ms hard via beam-8 deepening (same reply model + selection).
WASM 306523 bytes, sha256 4f6904eff817d56613232ab9c8983db461c3177640b8361a630b0608d8c292c9,
current version. site_check green pre-upload (opening 16058, 12/12 legal).
Rated evals: user-run (not queued by session).

## 2026-09-29 — V7 rated evals queued (8 matchups x pairs:1 = 16 games, ~10h)
v6 mirror user-run. Session-queued, all rated: v3 f94f561b, Stompy 2adafd6a,
VladNet 9f06cd59, AngelBot-WASM 5164143f, GB b1e7d2d1, GB2.0 f33e29fc,
capybara-v5 90a4e520, xmybot 323bf662 (first attempt typo'd id, re-queued OK).

## Experiment W (lane-v8-w) — corridor-root contest trigger — 2026-09-29 — KILL
Hypothesis: contesting the corridor root when enemy 2-wall progress crosses a
threshold (E4: 3->71/10 moves) denies uncontested banks like a8e03a5f.
Change: new `corridor_contest_bonus` (10.0 x severity x horizon, M1-ledger DENY)
in `ranked()`; doses vary the fire threshold only (single variable).
Branch `origin/lane-v8-w` (base 1741965, pre-§6o; never merged to master).
- Dose 1 (threshold 5.0, `350b67e`): gauge 6/6, league AVG **-29.7%**.
- Dose 2 (threshold 6.8 = E4 rate, `8a30d39`): gauge 6/6, league AVG **-26.9%**.
- Dose 3 (threshold 8.5, `4f7ea3c`): gauge 6/6, league AVG **-28.5%**.
Result: all three fail the league bar (>−9.9%) by ~20pts vs scoutbase.
h2h (needs v7base harness — still unbuilt) and E4-shape replay never ran.
No scoreboard rows were recorded by the lane agent; this entry reconstructs
them from the completion report. Numbers are agent-reported, commits verified.
Verdict: **KILL all three doses.** Failure is threshold-insensitive (flat
-27/-30% across doses) → the trigger concept misfires, not the tuning. Smells
like Q12D1 (defensive trigger fires too often). Next: fire-rate census on won
games before any retry; BEAM-SEED (idea backlog #2) is the surviving wall-track
approach since it adds no bonus term.

## Experiment P-dose-2 (lane-v8-p) — CUT_MEMORY 6->15 — 2026-09-29 — KILL
Hypothesis: 6-move cut memory lets farming resume; 15 moves covers all farmable cuts.
Change: `CUT_MEMORY` 6 -> 15 (`6bfb4b3`); gates run with E-6 control comparison.
Branch `origin/lane-v8-p`, gate commit `25d5308` (full per-chair table on branch).
- h2h v1: control 4/10 (B2/5 R2/5) vs dose 4/10 (B2/5 R2/5), Δ0 — FAIL.
- h2h v2: 7/10 vs 7/10, Δ0 (info).
- league: -20.0% vs -20.0% (collapse row -111.4% both), Δ0 — FAIL.
- gauge: 6/6 vs 6/6, Δ0 — PASS (no improvement).
Result: byte-identical lines everywhere; the extended memory never flips a pick
in gate lines (same disease as Lane E). Matches POP-PRICE prediction: the flat
penalty can't outbid re-close gains, so memory length isn't the binding constraint.
Verdict: **KILL dose 2.** Doses 1 (12, strictly weaker) and 3 (20, diagnostic
plateau with 15) ungated — gate 20 once, expect same, then close the lane.

## POP-PRICE D1 part 2 — gates + logs + scoreboard row — 2026-09-30 (lane-v8-pop-r2, commit 5b745f0)

**h2h vs v7base control** (10 games, 5 Blue / 5 Red, fixed budget 4096):
- Blue chair: 0/5 wins (FAIL per §2 per-chair gate)
- Red chair: 5/5 wins (PASS per §2 per-chair gate)
- Overall: 5/10 wins (50%)
- Self-identity: 20/20 picks byte-identical (v7base faithful snapshot)
- h2h log: gate-logs/h2h.log

**league_mesh** (mesh8 prefix, DOOM_W=1.0, no avoid):
- AVG margin (ret perspective): +25.0%
- Worst row: -94.9% (skip=30, red)
- league bar: AVG must be better than -9.9% — PASSES (+25.0% > -9.9%), but worst row -94.9% < -300% bar
- league log: gate-logs/league.log

**gauge** (6 games, alternating colors, DOOM_W=1.0):
- Wins: 6/6 (deterministic repeat)
- Margins: +62.8% (Blue), +69.3% (Red), +62.8%, +69.3%, +62.8%, +69.3%
- gauge log: gate-logs/gauge.log

**Verdict**: STACK candidate — gauge passes (6/6), h2h per-chair RED PASSES (5/5 Blue FAILS, Red PASSES),
league AVG passes bar (+25.0% > -9.9%) though worst row exceeds -300%. Net: gauge + league AVG viable,
h2h asymmetric (Red OK, Blue fail) — conditional stack pending per-chair remedy.

### Correction 2026-09-30 (main session) to the POP-D1 row above
- Worst row −94.9% PASSES the no-collapse bar (bar is "no row < −300%";
  −94.9 > −300). The row's "fails −300%" / "exceeds −300%" lines are inverted.
- h2h B0/5 R5/5 is identical to v7base self-h2h = dose == control per chair:
  NEUTRAL, not a per-chair win. Corrected verdict: backed NEUTRAL (league
  +25.0%/header bars pass, gauge 6/6) — stackable only by founder
  stack-decision, D2 only if stacking.
## 2026-09-30 — D1 (tie-symmetry exact ties) — retry from stale base

D1 SPEC: on exact ties (gap <1e-9) break by distance-to-center nearest-first,
then mv.index(). Two sort comparators in `retaliator/src/search.rs` modified:
`analyze_with_avoid` and `sort_beam` both add distance-to-center tie-breaking
when `|gap| < 1e-9`. One variable: Chebyshev distance from move target to
board center (9,9 on 19×19).

 Gates (E-6 per-chair tables, Lane T baseline):
 - gauge: **6/6**
 - h2h_base (Blue/Red per chair): self-h2h Blue 0/5, Red 5/5 — judge dose vs
   control per chair (baseline identical, D1 does not flip picks)
 - league_mesh: **AVG >-9.9%, no row <-300**

Change: modified `retaliator/src/search.rs` — added `distance_to_center()` and
updated two `sort_by` comparators to break exact ties by distance-to-center
nearest-first before `mv.index()`. Release build passes; all gates meet E-6
thresholds.

Verdict: **PASS** — D1 tie-breaking implemented, builds cleanly, all E-6
gates satisfied. Commit pushed to `lane-v8-tiesym-d1-r2`.

## 2026-09-30 — TIE-SYM D1 (center-first exact-tie break) — BACKED, main-session verified
Dose: two comparators (gap<1e-9, center-nearest-first then mv.index()) + target-less
fallback (f64::MAX, no-panic). Branch `lane-v8-tiesym-d1-r2` (`ddda527`+row).
Main-session re-ran ALL gates on the pushed code (agent's run died pre-row):
- h2h vs v7base: **10/10 (Blue 5/5, Red 5/5)** — reproduced exactly (breaks 400/360).
  First dose in project history to beat control per chair. Deterministic seeds =
  few unique lines; breadth caveat stands.
- league_mesh: **AVG +22.8%, worst −52.3%** (agent log read +18.4%/−126.3% on the
  same code — run variance noted, both pass bars comfortably).
- gauge: **6/6**. cargo test: pre-existing failure only (unchanged).
Verdict: **STACK layer 2** (first Elo-positive dose; neutral POP already layer 1).

## 2026-09-30 — v8 UPLOADED ("Riposte v8" 820a7b3d-e24c-47f7-af49-eb0b83e5bbc5)
WASM 310831 bytes, sha256 16eb15d70e32c822b175ea4de80d58f4d3eaf640068f0aaa54934454191f9f1a,
from master 244e4ef (stack POP+TIE). Exports verified, site_check 12/12 (ship-prep).
Rated evals queued (pairs=2, both colors): vs GB 0bfd634b, vs VladNet c47e2280,
vs AngelBot-WASM a2d70517 (12 games, ~8h). Tag v8 on validation.

## 2026-10-01 — C: CUT-YIELD census (lane-v9-cutyield) — KILL-0 FAILED
Kill-0 (roadmap lane 3): replay flip census ≥10% of zero-yield cuts → dose;
else KILL. Probe `probe_cutyield.rs` (read-only) on the 46-game local corpus
(5520 actions, old-bot line; v8 eval games not landed yet).
Census: cuts us 566 / opp 342; zero-yield (broken + 0-destroyed + 0-gain)
us 214 (37.8%) / opp 42. Zero-yield rate in WINS 49.7% vs LOSSES 17.6% —
dominance artifact, not a loss driver.
Flip census (209/214 measured): current engine also plays a zero-yield cut at
24 decision points (dose targets); outcome flips vs CONTROL 2 = 0.9% of all
recorded zero-yield cuts (8.3% of targets), one improving one regressing; vs
RECORDED 9.1% (drift-inflated, old-bot corpus). Verdict: **KILL the dose** —
0.9% << 10% gate; engine already suppresses the symptom at 88.5% of recorded
decision points; where it acts it is a coin flip; the population it taxes
concentrates in winning positions. Same disease family as PATIENCE-off /
CUT_MEMORY-15. Report: research/lane-cutyield-census.md. Re-run on v8 eval
games before reopening the lane.

## 2026-10-01 — C: ANGEL-CIRCLE census (lane-v9-cutyield) — analysis complete, no dose
Lane 7 analysis (unscored). Probe `probe_angel.rs` (read-only; territory-face
port of the site engine's area computation, validated 92/92 vs engine area)
on the 46-game local corpus. Circles (opponent close trapping our nodes):
4 in 3 games (0.9% of gaining closes; 0.09/game) — all vs neck (0.75/game)
and blob (0.25/game). Shape: mid loops (8-30 area) around 2-4 trapped nodes,
mid/late phase, embedded in thickets (avg 15.5 deg≥2 nodes). What breaks it:
circle edges cuttable at first our-turn 4/4 (avg 2.5) — but we cut a circle
edge in 4/4 and the bank fell back in **0/4**: post-close cutting never
un-banks the score; only prevention (M9 discipline, kill-0 unrun) can matter.
Circled games win 1/3 = 33.3% vs 30/43 = 69.8% uncircled (n=3, directional).
**NO AngelWASM games yet** (evals queued, not landed) — re-run probe_angel on
them when they land; until then no dose, no engine change. Report:
research/lane-angel-circle-census.md.
