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

## Experiment E (Lane E v7 Q1) — break-fix valuation dose sweep — 2026-09-29
Term (`eval_phases.rs` `Breakfix{missed,floor}` + `breakfix_best_move_with_avoid`):
the enemy's worst one-action pop charged at MISSED events (area x
min(events,missed)), not the full horizon — popped ground re-made after the
~two-turn shield delay banks every event except ~2, and ground re-made before
the next event misses none (~0). Q1b adds a per-break fixed tempo floor: any
live pop charges at least floor x hz (the re-make burns a turn whatever the
area). Dose via `E_MISSED=m[,f]`; absent = OFF = doom-OFF control. Doom stays
off on this path (charges never stack). Same gates as Q2.
| dose (E_MISSED) | h2h W/L | league margin % | gauge W/L | life (variant vs shipped) | verdict |
|---|---|---|---|---|---|
| Q1a missed-only (2,0) | 5/10 (blue 1/5 -17.4%, red 4/5 -1.8%) | -23.5% (skip0 +26.8/+39.8, skip10 +26.8/-137.7, skip20 +26.8/-137.7, skip30 +36.1/-68.6; diffs +285/+776/+285/-1131/+285/-1131/+741/-536) | 6/6 byte-identical to E0 (blue +53.5 x3, red +69.3 x3; breaks R=20/3, G=16/2 — term never flips a gauge pick) | life 10.2 vs 11.8, bpb 5.27 vs 6.18 (FELL), breaks 286 vs 258 (ROSE +28) | KILL (h2h = control; league -13.6pp with collapse skip10/20-red -94.5 -> -137.7 — even 2 events overcharge there; life watch both wrong way) |
| Q1b missed + floor (2,0.5) | 5/10, all 10 game scores byte-identical to Q1a (floor never flips an h2h pick) | -17.1% (skip0 +26.8/+39.8, skip10 +26.8/-123.1, skip20 +26.8/-123.1, skip30 +57.5/-68.6; diffs +285/+776/+285/-1116/+285/-1116/+1154/-536) | 6/6 byte-identical to E0/Q1a (term never fires) | life 10.2 vs 11.8, bpb 5.27 vs 6.18, breaks 286 vs 258 (identical to Q1a) | KILL (floor adds charge exactly where dose-response says to subtract; league still -7.2pp vs control) |
Q1 verdict: KILL the break-fix valuation form on all gates — no dose beats control
on any gate (best: Q1b league -17.1% vs -9.9%; h2h never exceeds 5/10; life bpb
falls and breaks rise at both doses). Mechanism note: pricing worst-pop at the
true ~2-event cost still loses because the charge steers AWAY from holding
breakable area on the collapse lines (skip10/20-red), where shipped's unpriced
exposure banks anyway — the disease is the max-pop charge itself, not its size.
Per lane plan (Q1 failed both doses): Q4 deny-remote SKIPPED (gated on Q1 pass),
Q5 single dose next.

## Experiment E (Lane E v7 Q5) — delayed-close bonus scaled by open space — 2026-09-29
Term (`eval_phases.rs` `Delay{w}` + `delay_best_move_with_avoid`): a first-action
close (Connect, gain > 0) gets bonus w x gain x openfrac x hz at ranking +
selection, where openfrac = room/(room+area) post-move in [0,1] — closes that
keep the map open keep ~full gain credit, closes that shut our last room keep
~none. Engine structure only (room/area), full-horizon units, deterministic
tiebreak by move index. Single dose via `E_DELAY=w`.
| dose (E_DELAY) | h2h W/L | league margin % | gauge W/L | life (variant vs shipped) | verdict |
|---|---|---|---|---|---|
| Q5 open-space close (w=1.0) | 4/10 (blue 1/5 -24.2%, red 3/5 -23.4%; open-None as red -140.4% blowout) | -32.4% (skip0 -14.3/+34.3, skip10 -14.3/-124.3, skip20 -14.3/-124.3, skip30 +12.7/-14.8; diffs -157/+695/-157/-1215/-157/-1215/+168/-166) | 6/6 (blue +60.7 x3, red +69.4 x3; breaks R=13/5, G=9/2 — term fires here, unlike Q1) | life 8.5 vs 15.6, bpb 4.72 vs 7.65 (FELL), breaks 301 vs 225 (ROSE +76) | KILL (worse than control on h2h, league, and life; the bonus rewards open-space closes that scout-class opponents cut — openness without thickness is grazeable) |
Q5 verdict: KILL after the single tasked dose. Lane E v7 closes with no surviving
candidate: Q2 REJECT, Q1 KILL (both doses), Q5 KILL. Control (doom-OFF) stands.

## Experiment E (Lane E v7 Q4) — deny-remote contest bonus — 2026-09-29
Term (`eval_phases.rs` `Deny{w}` + `deny_best_move_with_avoid`): mirror of the
shipped REMOTE_BONUS — bonus w x (foe remote-wall count contested) x hz at
ranking + selection for OUR non-breaking first actions landing within 3 of
enemy nodes that are junctioned (degree >= 2, shared-node unbreakable) AND
far from all our nodes (> REMOTE_DIST 5, far from the fight), heat-gated
(heat >= FIGHT_HEAT 10) like the shipped mirror. Priced by shared-node
COUNT, never by area held. Full-horizon units, deterministic tiebreak by
move index; CONTACT still bans the free graze underneath. Dose via `E_DENY=w`.
Q4 was SKIPPED in v7 (gated on a Q1 pass that never came); un-gated per task.
Same gates as Q2 plus the life watch (enemy area-lifetime must fall without
ours falling, dose-vs-control).
| dose (E_DENY) | h2h W/L | league margin % | gauge W/L | life dose vs control | verdict |
|---|---|---|---|---|---|
| Q4D1 contest-remote (w=0.25) | 5/10 (blue 1/5 -18.8%, red 4/5 -2.1%) — all 10 game scores byte-identical to E0 | -9.9% — all 8 rows byte-identical to E0 (skip0 +29.0/+39.8, skip10 +29.0/-94.5, skip20 +29.0/-94.5, skip30 +38.9/-55.9; worst row -94.5% > -300%) | 6/6 byte-identical to E0 (blue +53.5 x3, red +69.3 x3; breaks R=20/3, G=16/2) | dose life line-for-line identical to control life (variant 9.7 vs shipped 12.2, bpb 5.00 vs 6.43, breaks 283 vs 254): enemy lifetime did NOT fall, ours did NOT fall — zero pick-flips | KILL (term never flips a pick at the tasked small weight on any of 34 games; h2h 5/10 < 6/10, league -9.9% not > -9.9%, life gate vacuous. Not directionally positive, so no Dose 2 per plan. Either the heat x remote-junction conjunction never arms in these lines, or 0.25 x hz never breaks a tie — both mean the form carries no signal at this weight) |
Q4 verdict: KILL after Dose 1. No Dose 2 (Dose 1 exactly control = neutral, not
directionally positive). Next per plan: C2 census measurement (probe_b_census).

## Experiment E (Lane E v7 C2-census) — line-strength measurement — 2026-09-29
Probe (`retaliator/examples/probe_b_census.rs`, measurement-only, no eval
change): corpus = h2h gate lines (10 games, variant vs shipped) + league
genuine rows (skip0 both colors, variant vs scoutbase), BOTH sides' closes
logged (win control). Per close (Connect, gain >= 2.0): gain; touches =
max(source-degree-before, target-degree-after); bank-rate = own score delta
over next 6 own-actions / gain; survived = own area at game end >= 50% of
post-close peak; dose + result + color + side. JSONL rows + stratified
summary. NOTE: first run voided (cross-game trajectory slice bug —
closes resolved against later games' trajectories); fixed (per-game
close/snap ranges, game lines to stderr) and rerun. n=523 unique closes per
dose (1046 pooled); n>=200 gate MET. Dose 0.25 games byte-identical to
control (Q4 never flips), so the dose covariate is vacuous — analysis pooled.
| predictor | win stratum (n=548) | loss stratum (n=498) | verdict |
|---|---|---|---|
| gain [2,4.6)/[4.6,6)/[6,10)/[10+) survival | 0.246/0.273/0.397/0.192 | 0.252/0.286/0.270/0.338 | NO — [10+) flips worst-in-wins to best-in-losses; C2's loss-corpus negative slope does not replicate with win control (survived-mean gain 9.16 vs farmed 10.63, far weaker than C2's 5.62 vs 11.57) |
| touches 2/3/4+ survival | 0.331/0.239/0.175 | 0.273/0.253/0.368 | NO — sign flips across strata (negative in wins, positive in losses); confounded with gain (higher touches <-> smaller gains: 12.9/11.0/7.6 win meangain) |
| bank-rate survived vs farmed | 24.37 (n=288) vs 21.88 (n=758) pooled | same pooled | WEAK — small positive gap, but post-hoc (measured after the fact); unusable at pick time without a predictive model |
Census verdict: with win control, NONE of gain/touches/bank-rate predicts
survival consistently — every candidate flips sign across strata. No
"strength" term meets the spec gate (a predictor only counts if it flips a
pick; nothing here earns one). C2's H1 caveat confirmed load-bearing: the
loss-corpus negative gain slope was a corpus artifact (in losses the enemy
eats big ones and lets small ones stand), not a structural law.

## Experiment E (Lane E v7 Q12) — reinforce-vs-prevent thick-line bonuses — 2026-09-29
Term (`eval_phases.rs` `Thick{r,p}` + `thick_best_move_with_avoid`): two
sub-terms, one dose variable each, swept against each other (reinforce-only
vs prevent-only, 2-dose max). REINFORCE (r): bonus r x hz (binary, the
shipped DENSE form) for OUR non-breaking first actions that EXTEND our 2+
touch lines (source already a shared node, degree >= 2 — the Q2-extend form)
or ANCHOR to them (target within Chebyshev 1 of one of our shared nodes —
topological, vs DENSE's head-count). PREVENT (p): bonus p x (pairs
contested) x hz (the Deny form) for OUR non-breaking first actions landing
within Chebyshev 3 of either endpoint of a foe near-thick frontier pair
(both endpoints foe wall-ends, degree exactly 1, no edge between them,
geometrically closable in one move — the foe close's legality is NOT checked,
it is not foe's turn; geometry is the proxy, stated not hidden). CONTACT
still bans the free graze on top. Full-horizon units, deterministic tiebreak
by move index. Dose via `E_THICK="r,p"`; (0,0) = OFF = the doom-OFF control
(OFF-identity verified: `E_THICK="0,0"` gauge byte-identical to control,
6/6 blue +53.5 x3 / red +69.3 x3). Gates per dose: h2h >= 6/10, league >
-9.9% (no row < -300%), gauge 6/6, thick-life (variant share UP *and*
shipped-side completions DOWN, dose-vs-control; `probe_b_thick`: share =
mean shared-node fraction of own nodes, comp = own moves raising own shared
count). HEADWIND (stated before running): the C2 census finds touches do not
predict close survival with win control — a pure touch bonus has no survival
mechanism behind it.
| dose (E_THICK) | h2h W/L | league margin % | gauge W/L | thick-life dose vs control | verdict |
|---|---|---|---|---|---|
| Q12D1 reinforce-only (1,0) | 6/10 PASS (blue 2/5 -3.7%: None +45.4 W, 4864 -12.5 L, 5589 -29.3 L, 11723 -35.9 L, 9199 +14.0 W; red 4/5 +1.4%: None -47.4 L, 4864 +17.4 W, 5589 +23.2 W, 11723 +3.9 W, 9199 +10.0 W) | -12.6% FAIL (skip0 -7.4/+38.3, skip10 +15.2/-63.6, skip20 +15.2/-63.6, skip30 +21.3/-55.9; worst -63.6% > -300%) | 6/6 PASS (blue +66.2 x3 R=23 G=16, red +69.2 x3 R=4 G=2 — term fires vs greedy, holds) | variant share 0.761 vs 0.744 UP (+0.017, mechanism fires) BUT shipped-side comp 416 vs 416 FLAT (no denial without the prevent half) — FAIL (needs both) | FAIL (passes h2h + gauge, fails league + thick). Per-row deltas vs E0: collapse line as red improves (skip10/20-red -94.5 -> -63.6) but as-blue collapses (skip0-blue +29.0 -> -7.4, skip10-blue +29.0 -> +15.2, skip30-blue +38.9 -> +21.3): thickening as blue is slow — scoutbase out-expands us. The h2h pass does not travel. |
| Q12D2 prevent-only (0,1) | 2/10 FAIL (blue 1/5 -24.7%: None +25.8 W, 4864 -37.4 L, 5589 -59.9 L, 11723 -35.9 L, 9199 -16.3 L; red 1/5 -44.6%: None -130.5% blowout L, 4864 -1.9 L, 5589 +8.3 W, 11723 -7.7 L, 9199 -91.1 L) | -58.2% FAIL (skip0 -3.6/+23.1, skip10 -3.6/-174.1, skip20 -3.6/-174.1, skip30 +3.5/-133.4; worst -174.1% > -300%) | 6/6 PASS (blue +70.7 x3 R=15 G=12, red +69.3 x3 R=4 G=2 byte-identical to control) | shipped-side comp 410 vs 416 DOWN (-6, denial fires weakly) BUT variant share 0.737 vs 0.744 DOWN (own lines thinner) — FAIL (needs both) | KILL (worse than control on h2h, league, and own share; only gauge holds). Mechanism: contesting drags our moves onto foe wall-ends — contact tempo loss the CONTACT penalty (1.0 x hz) cannot outbid (p=1.0 x pairs x hz wins the tiebreak toward the graze). The -6 foe completions cost -48.3pp of league. |
Q12 verdict: KILL after the tasked 2-dose max. Neither half passes its gates;
no joint dose (would combine a league-losing term with a catastrophic one —
no mechanistic path to a joint pass). Reinforce thickens our lines (+0.017
share, h2h 6/10) but loses the as-blue expansion race; prevent weakly denies
foe thickness (-6 comps) but sacrifices our game to do it. The census
headwind held: thickness is not survival.

## Experiment E (Lane E v7 Q8) — line-strength census rerun + KILL verdict — 2026-09-29
Reran `probe_b_census` control dose (E_DENY unset): 12 games (10 h2h + skip0
both colors vs scoutbase), n=523 closes (win 274 / loss 249), n>=200 gate MET.
Replicates the §C2-census control half byte-for-number (win gain buckets
0.246/0.273/0.397/0.192; touches 0.331/0.239/0.175). New: gain x touches joint
split — big+high-touch closes survive 12.9% in wins (n=31) vs 40.9% in losses
(n=22); small+low-touch 36.2% wins vs 23.5% losses. Every candidate (gain,
touches, bank-rate) flips sign across win/loss strata. No shape-family labels
in probe output (no corridor/blob/thicket/loop classifier in-tree); joint
gain x touches proxy is the closest the data allows, and it also flips.
| census (control, n=523) | win stratum | loss stratum | verdict |
|---|---|---|---|
| survival predictor check | all slopes flip vs loss stratum | mirror image | KILL — strength does not predict survival with win control; no pricing attempt per task spec |
Q8 verdict: KILL. No reinforce-strong/abandon-weak term priced (nothing clears
the census gate). Caveat: control dose means both h2h sides play the shipped
policy — line diversity limited to one policy's closes; a multi-policy corpus
could in principle differ, but the sign-flips are large (12.9% vs 40.9%) and
unlikely to unflip.
