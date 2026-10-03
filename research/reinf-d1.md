# REINFORCE-LINES D1 — DENSE extension (shared-node walls) — verdict: KILL

**Lane:** v9 roadmap item 4 (REINFORCE-LINES, founder) · GLM subagent on
`lane-v9-reinforce-glm` → pushed to `origin/lane-v9-reinforce` (append-only).
**Base:** `530d4b4` (v8 stack-2 master line). **Kill-0:** adopted GO
(`5e43810`, research/reinf-kill0-GO.md — builds landing next to 2+ own nodes
survive 84.5% pooled / 94.6-95.7% in wins, +15.5/+67.1pp per chair).

## Dose (one variable: `REINFORCE_DENSE_W = 1.0`)

Roadmap text: *extend DENSE_BONUS toward building behind 2+ shared-node
walls, then expand far / bank behind. Structural props only. Respect
DEADWOOD_PENALTY — reward only thickets that gain or reach.*

Implementation (`retaliator/src/eval_reinforce.rs`, the Lane B snapshot
pattern — verbatim copy of shipped `search.rs` at `530d4b4` plus the term;
`src/search.rs` and `src/lib.rs` untouched on this branch):

- **Insertion:** first-action *ranking* priority, immediately after the
  shipped DENSE block (same layer DENSE_BONUS itself lives in; the shipped
  DENSE term is untouched). Full-horizon points: `+ W × structure × hz`,
  `hz = min(events_left, 12)`, so it competes with DENSE_BONUS/REMOTE_BONUS
  (≤ +12 each at hz=12), never with area × events swings.
- **BUILD (≤ 1.0):** move lands next to 2+ own nodes (the shipped DENSE
  anchor + kill-0 census definition); credited by the *shared-node ratio*
  of that wall after the move — of our nodes within Chebyshev 1 of the
  target, the fraction carrying 2+ of our edges ("2+ touches = no legal
  cut"), with an adjacency-depth floor (0.25/0.5/0.75 at 2/3/4+ adjacent).
- **EXPAND (≤ 1.0):** source carries 2+ of our edges after the move
  (anchored in the mesh), target is REMOTE_DIST (5) from every enemy node,
  and the move gains area or grows room (≥ IDLE_ROOM).
- **DEADWOOD/IDLE respected:** zero unless the move gains area, reaches the
  enemy (within DEADWOOD_ENEMY_DIST 3), grows room, or breaks an edge; the
  exact shipped DEADWOOD condition (Connect, no gain, far) hard-zeroes the
  term so it can never buy back a move the engine classes as dead wood.
- Deterministic (no HashMap iteration in the scoring path, no RNG; shipped
  comparators + tiebreak by move index untouched).

**Replaced scaffold:** the `530d4b4` H1 term (bonus for moves near
*threatened* nodes) was swept NO-GO on `lane-v9-reinforce-sweep`
(`dc3d538`: every weight tied 12/24, identical metrics — zero pick-flips at
every weight). This lane implements the roadmap item-4 dose instead.

**Parallel attempt (post-hoc note, 2026-10-03):** while this lane gated, a
second session landed `4ec42ed` on the same branch — an independent
selection-time form of item 4 (`+W × Δdurable × hz` in
`analyze_dosed`, 3-weight sweep, own identity gate 239/239, own logs
`research/logs/r9-reinforce-*.txt`, own KILL note
`research/r9-reinforce-KILL.md`). Verdict there: KILL as well (league
monotone-negative at every weight; opening-row damage −34 to −107pp
outweighing mid/late gains; survival tripled but h2h "did not travel").
This lane's code was therefore renamed at integration to
`src/eval_reinforce_dense.rs` / `examples/probe_r_reinforce_dense.rs`
(the selection-time attempt owns the canonical file names; the committed
gate logs were produced by this code under its pre-rename path —
re-verified after rename: identity 32/32, census 4/36 at W≤1, byte-equal
key lines). The two KILLs compose into a stronger claim than either alone:
**the reinforcing direction fails in BOTH layers on this base** —
ranking-time (this lane: promotes mediocre thicket landings; 2/10 direct)
and selection-time (theirs: cedes expansion tempo that scout converts into
more banking than the protection saves). Any revival needs the
phase-gated/corpus-targeted gate both lanes point at.

## Gates (all logs: `retaliator/gate-logs/reinf-d1-*.log`)

### OFF-identity — PASS (hard gate, ran first)
`baseline_*` twins (weight 0.0) vs the LIVE shipped engine
(`retaliator::search`), 32 deterministic positions (fixed-budget lineage +
routed mesh lineage, budget 4096): **picks 32/32 byte-identical, full
candidate lists 32/32 byte-identical** (moves, evaluations bit-for-bit,
node counts). Routed/timed spot check (report-only, machine-dependent
path): 5/5 identical.

### Flip census (inertness detector — the NO-GO's failure mode was 0 flips)
36 deterministic positions, dose pick vs OFF pick at runtime weights:
W=0.25: 4/36 (11.1%) · W=0.5: 4/36 · **W=1.0: 4/36** · W=2. 6/36 · W=4:
6/36. The term is live at every tested weight (the H1 NO-GO showed 0
everywhere). Ungated context only; D1 gates at the compiled W=1.0. The
same 4 decisions flip at every weight ≤ 1.0 — shrinking W does not remove
the flipped picks, so the gate failure below is not a weight-tuning
artifact.

### h2h: dose vs control (live engine) — FAIL
10 games (skips 0..45 step 5, both colors; fixed budget 4096 both sides):
**dose 2/10 (Blue 2/5, Red 0/5)** — bar ≥ 6/10.
Pooled: dose survival **7.0%** of produced area, avg margin −54.7%,
dense-landing freq 30.3% of our moves (re-run byte-identical — free
determinism double-check). Per-game margins: −24.6, −152.7, −81.0, −152.7,
−23.6, −61.4, +35.9, −61.4, +35.9, −61.4 %.

### Suite: dose vs control, E-6 per chair (n=8 per opponent, 4+4 colors)

| opponent | dose W-L (B/R) | control W-L (B/R) | E-6 delta (B/R) | dose survival / margin | control survival / margin |
|---|---|---|---|---|---|
| v1base   | 1/8 (1/4, 0/4) | 2/8 (2/4, 0/4) | −1 / +0 | 8.6% / −55.3% | 5.1% / −39.3% |
| scoutbase| 2/8 (2/4, 0/4) | 3/8 (3/4, 0/4) | −1 / +0 | 13.9% / −34.1% | 12.1% / +2.2% |
| greedy   | 8/8 (4/4, 4/4) | 8/8 (4/4, 4/4) | +0 / +0 | 95.7% / +54.8% | 94.3% / +57.7% |

The dose never beats control on any opponent, drops one game apiece vs v1
and scout (both in the Blue chair), and craters the scout margin (−34.1%
vs control's +2.2%). It *does* raise dense-landing frequency in play
(39.5-49.8% of our moves vs control's 36.6-41.7%) — the term steers as
designed, and steering toward thicket landings loses games against
cutting opponents.

### Gauge (vs greedy 1-ply, 6 games) — PASS
dose 6/6, control 6/6 (dose outscores control: 4742-1790 / 5225-1371 vs
3433-975 / 4415-1423 — vs a non-cutting blob the thickets bank more).

### League (league_mesh structure, routed, vs scoutbase) — PASS, but ≈ control
8 rows per variant (skip 0/10/20/30 × both colors, routed through the mesh
prefix + timed deepening, the published harness):

| row | dose margin (survival) | control margin (survival) |
|---|---|---|
| skip=0 blue  | +23.9% (8.4%)   | +21.0% (13.7%)  |
| skip=0 red   | +61.0% (19.7%)  | +33.1% (26.3%)  |
| skip=10 blue | **−32.6%** (18.3%) | +59.4% (23.8%) |
| skip=10 red  | +54.1% (65.1%)  | +48.0% (57.4%)  |
| skip=20 blue | +20.6% (11.5%)  | +6.7% (10.9%)   |
| skip=20 red  | +2.3% (0.9%)    | +15.3% (10.3%)  |
| skip=30 blue | +53.5% (23.2%)  | +48.3% (64.2%)  |
| skip=30 red  | −86.8% (5.1%)   | −142.1% (0.0%)  |

**dose AVG +12.0%, worst −86.8% (PASS) | control AVG +11.2%, worst
−142.1%.** Within run-variance noise of control (the timed path is
machine-speed-dependent; the TIE-SYM row already noted same-code league
variance), with one bad dose regression (skip=10 blue: −32.6% vs control's
+59.4%) and one row where the dose out-survives control 3-5× (skip=10 red:
65.1% vs 57.4%, skip=30 red: 5.1% vs 0.0%).

## Verdict

**KILL (do not stack, do not sweep).** OFF-identity and gauge pass, the
census proves the mechanism is live (4-6/36 flips), and league clears its
bars (+12.0%/−86.8%) — but the two gates that judge the dose *against
control at the same color* both fail: h2h vs control **2/10** (bar 6/10;
Blue 2/5, Red 0/5) and the E-6 suite is dose ≤ control on every opponent
(v1 −1, scout −1, greedy +0; scout margin −34.1% vs control's +2.2%).
League AVG is within noise of control, so the one passing aggregate does
not offset the direct losses. Per the census, weights ≤ 1.0 flip the same
4 decisions — shrinking W does not remove the flipped picks — so the
failure is the direction, not the dose size.

**Reading (for the autopsy files):** the kill-0 census measured that
thicket landings *associate with winning* — but association is not
causation in either direction: the engine already reaches those landings
via DENSE_BONUS + value when they are good; adding ranking pressure toward
"any 2-adjacent or anchored-far landing that gains or reaches" promotes
the *mediocre* ones (the good ones were already being played), and against
cutters the extra clustered builds feed the rebuild-farm cycle the site
autopsy documented (96/147 closes re-closed within 2 turns of a cut).
The survival metric moved toward the dose's intent on some axes (h2h 7.0%
of produced area; vs v1 8.6% vs control 5.1%; league skip=10 red 65.1% vs
57.4%) yet wins did not follow — surviving-thicket area is not the
binding constraint; the tempo donated to reach it is.

**Status of roadmap item 4:** the dose as specified (ranking-time DENSE
extension) is dead on this lane. Any successor must change the layer
(selection-time, inside `selection_adjusted`, where the farm decisions are
actually made) rather than the weight, and must first pass a flip census
showing it flips the *rebuild-farm* positions (the 48/96 `now=same` set
from the site autopsy) — not merely 4/36 generic positions.

— Lane REINFORCE (GLM), 2026-10-03.
