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

## Experiment E (Lane E v7 Q2) — unbreakable-shape dose sweep — 2026-09-28
Term (`eval_phases.rs` `Unbreak{extend_w,create_w}` + `unbreak_best_move_with_avoid`):
bonus for OUR first actions that EXTEND shared-node walls (source degree >= 2)
or CREATE 2+ touch thickets (target degree >= 2 post-move), full-horizon units
(area x min(events,12)), deterministic tiebreak by move index. Dose via
`E_UNBREAK=e,c`; (0,0)/absent = OFF = the doom-OFF control. Probes take
`E_UNBREAK` in `probe_b_h2h`/`league_b`/`gauge_b`; new `probe_b_life` reports
area lifetime / banked-per-built / break counts over the h2h set (Q2 + Q1
metrics). Gates per dose: h2h >= 6/10, league > -9.9% with no row < -300
(diff), gauge 6/6. Structural generality only (no named-pattern matching):
a dose that helps one shape family and hurts others is killed with numbers.
| dose (E_UNBREAK) | h2h W/L | league margin % | gauge W/L | life (variant vs shipped) | verdict |
|---|---|---|---|---|---|
| E0 control (OFF) | 5/10 (blue 1/5 -18.8%, red 4/5 -2.1%) | -9.9% (skip0 +29.0/+39.8, skip10 +29.0/-94.5, skip20 +29.0/-94.5, skip30 +38.9/-55.9; diffs +313/+776/+313/-1038/+313/-1038/+830/-472) | 6/6 (blue +53.5 x3, red +69.3 x3; breaks R=20/3, G=16/2) | life 9.7 vs 12.2, bpb 5.00 vs 6.43, breaks 283 vs 254 | baseline (faithfulness 0/239; league/gauge re-verified byte-identical this lane) |
| E1 rescue dose (1,1), v1 form (all first actions) | 6/10 (blue 2/5 +3.9%, red 4/5 -3.0%) | -24.9% (skip0 -30.3/+19.5, skip10 +18.1/-99.2, skip20 +18.1/-99.2, skip30 +38.5/-64.5) | n/g (league fail) | n/g | REJECT (league -15.0pp; skip0-blue collapses +29.0 -> -30.3; no row < -300 diff) |
| E2 create-only (0,1) | 5/10 (blue 1/5 -11.6%, red 4/5 -0.4%) | n/g (h2h fail) | n/g | n/g | REJECT (control-level; CREATE rarely flips a pick — several lines byte-identical to E0; junction-creating moves already bank DENSE_BONUS, so the marginal bonus only flips near-ties) |
| E3 extend-only (1,0) | 6/10 (blue 2/5 +4.0%, red 4/5 -6.0%) | -25.9% (skip0 -33.9/+15.9, skip10 +18.1/-99.2, skip20 +18.1/-99.2, skip30 +37.7/-64.5) | n/g (league fail) | n/g | REJECT (EXTEND carries E1's h2h gain AND the league collapse; CREATE was mildly mitigating in league) |
| E4 half-extend (0.5,0), v1 form | 6/10 (blue 2/5 -3.7%, red 4/5 -6.8%) | -20.6% (skip0 -19.4/+27.2, skip10 +26.5/-99.2, skip20 +26.5/-99.2, skip30 +37.3/-64.5) | n/g (league fail) | life 10.8 vs 12.8, bpb 5.57 vs 6.65, breaks 281 vs 257 (variant +11% life, +11% bpb, breaks flat vs E0) | REJECT (mechanism fires directionally but league dose-response monotone-negative: 0.0 -> -9.9, 0.5 -> -20.6, 1.0 -> -25.9) |
| E5 v2 non-breaking-only (0.5,0) | 6/10 (blue 2/5 -9.2%, red 4/5 +2.1%) | -15.3% (skip0 -7.4/+39.8, skip10 +29.7/-98.7, skip20 +29.7/-98.7, skip30 +38.9/-55.9) | n/g (league fail) | n/g | REJECT (best league of the sweep: 4 rows back to control-identical, skip0-blue damage halved — but still -36pp there; v1 rewarded cuts launched from shared nodes, v2 removes those flips) |
| E6 v2 quarter (0.25,0) | n/g (league screen fail) | -19.1% (every row control-identical EXCEPT skip0-blue +29.0 -> -46.0) | n/g | n/g | REJECT (non-monotone single-row attractor flip: skip0-blue reads +29.0/ -46.0/ -7.4/ -33.9 across doses 0/0.25/0.5/1.0 — one early pick-flip vs scoutbase swings +/-40pp; the endpoint-bonus form is dead) |
Q2 verdict: REJECT the endpoint-bonus form on all gates — no dose beats control on
h2h + league jointly (best: E5 v2-half, h2h 6/10 + league -15.3%). The life probe
confirms the mechanism direction (lifetime +11%, bpb +11%, breaks flat at half
dose) but the bonus steers early wall play into lines scoutbase punishes and
shipped does not (h2h pass + league fail at every positive dose). Next: Q1
break-fix valuation (charge only missed scoring events, ~2-event doom).
