# Lane B: phase-aware eval — mission report (2026-10-01)

Branch: `lane-b-eval` (worktree `/tmp/opencode/game-lane-b`). Baseline: the
shipped `search.rs` (byte-identical to the main session's working checkout,
verified by blob hash; doom ON, league control −20.0%). New code:
`retaliator/src/eval_phases.rs` (appended section: pure term functions +
`replay_adjusted`) and `retaliator/examples/probe_eval.rs` (census /
cutline / league / gauge / h2h modes). No rig internals, `search.rs`, or
`lib.rs` were touched; every earlier lane-b verdict stands.

## Mission

Phase-aware evaluation and long-run area maximization. Key hypothesis to
quantify with the engine: **cutting enemy area AND gaining own area in one
line beats pure gaining, because the score gap swings twice — measure the
true banked-score swing of cut-vs-build lines over the remaining events,
and check whether the current symmetric 0.5 beyond-horizon weight is right,
or denial deserves more (or less).** Terms to adapt as pure functions:
one-move loop-closing potential (all closable pairs), mobility,
cuttable-edges vulnerability, frontier/node differential, phase gates with
per-phase area weights.

## Validation instrument

`replay_adjusted` re-scores `search::analyze`'s top-8 candidates with the
shipped arithmetic (sign·evaluation + claim/denial-split horizon extension
+ fresh-wall poison − doom max-pop) plus gated extra terms. **Fidelity:
199/199 decision points** reproduce the shipped pick exactly over the
8-game league corpus (census mode prints this before any flip count).
All flip counts below are therefore real term effects, not drift.

## The KEY measurement — cut+make vs make-only, true banked-score swing

`probe_eval cutline`: 8 league-style games (skip 0/10/20/30 × both colors,
shipped vs scoutbase); at spread shipped-side turn starts (every 3rd in
[16,96), every one in [98,118), positions deduped — the harness's blue
games converge across skips), enumerate ALL breaking first actions (top 24
by value) + top-8 gaining firsts, each with its best second action by the
shipped static value; classify the two-action lines; take the best
**cut+make** (destroyed ≥ 0.5 ∧ gain ≥ 0.5) and best **make-only**
(destroyed < 0.5 ∧ gain ≥ 0.5) line by static value; **play both out to
game end with the shipped search for both sides** (fixed policy — the
ecosystem the site actually rates); margin difference = the TRUE
banked-score swing. Pricing model: shipped price = A + w·B, where
A = mover-relative leaf value (banked score + area×min(ev,12), already
includes destroyed×min(ev,12)) and B = destroyed×(ev−12)×0.5 — the
beyond-horizon denial credit, linear in the denial weight w (shipped w=1
on both claim and denial, i.e. the symmetric 0.5).

**Middlegame (n=19, ev 13–45):**
- True swing mean **+71** final points (95% CI ±91), cut+make better in
  **12/19**. The double-swing is real: a line that both banks own area and
  zeroes enemy area ends the game ~70 points better than the best pure
  claim of the same turn.
- Priced at shipped w=1 the swing is only +42; true − priced = **+28**
  (positive in 11/19). The shipped pricing *underprices* the cut+make line.
- Implied denial weight: **w* median +2.08** (trimmed mean +2.33, p25 −1.30,
  p75 +6.79). Point estimate: **denial deserves ~2× its current
  beyond-horizon credit** (effective 1.0 instead of 0.5 per unit destroyed
  past the horizon). Caveat: per-point w* is chaotic (two playouts from
  one-line-different positions diverge — the w* tail is butterfly
  effect, not signal), so the median/trimmed values carry the claim, not
  any single row.
- The shipped bot already picks the cut+make line in 12/19 midgame points
  (the within-horizon symmetric pricing already prefers it) — the
  underpricing costs it the other ~half.

**Endgame (n=14, ev ≤ 12):**
- True swing mean **+2** (95% CI ±18), 6/14 positive — a coin flip.
  **Endgame denial is worthless**: with ev ≤ 12, B ≡ 0 (the
  beyond-horizon extension is structurally inert) and the true swing is
  ~0. Cutting enemy area in the last 22 actions does NOT convert to final
  margin under the shipped policy (their rebuild or the reduced event
  count absorbs it). This matches findings-endgame's reach census and
  extends it: even *available*, *area-destroying* cuts don't pay late.
- Phase-gate verdict: the shipped formula's own phase gate (the ev−12
  term) is structurally CORRECT — denial counts only while ev > 12 and
  the measurement says it should. No extra phase gating is warranted by
  this data; the only open phase question is the midgame magnitude.

**Answer to the mission's question:** denial deserves MORE, not less —
point estimate 2× — but the census shows the lever barely moves play
(deny2 flips 2.5% of picks, deny0 6.5%): the top-8 candidates rarely
differ in beyond-horizon destroyed area. The exploitable underpricing
lives *within* the horizon in *which* cuts get chosen (the flat treatment
of enemy-area-destroyed vs enemy-structure-damaged), which is the Q7b
break-taxonomy / CUT-YIELD program, not a horizon_extension weight change.

## Term census (n=199 decision points, fidelity 199/199)

Single-term pick flips (one term on at a time, vs the shipped pick):

| term | flips | share | prior evidence |
|---|---|---|---|
| close2 (closable-pair potential ×2) | 58 | **29.1%** | lane-b2's close-SIZE weight (different term) REJECTED 1.0–3.0 |
| mob0.05 (mobility diff ×0.05) | 28 | 14.1% | lane-b SKIPPED mobility (no disease measured then) |
| vuln4 (cuttable-edges diff ×4) | 18 | 9.0% | area-based VULN term REJECTED (this is the count cousin) |
| claim0 (drop beyond-horizon claim credit) | 17 | 8.5% | — |
| close0.5 | 17 | 8.5% | — |
| deny0 (drop beyond-horizon denial credit) | 13 | 6.5% | direction check: should HURT per w*≈2 |
| node3 (node-count diff ×3) | 7 | 3.5% | — |
| deny0.5 | 6 | 3.0% | — |
| deny2 | 5 | 2.5% | kill-0 fail (<10%) despite w*≈2 |
| frontier2 | 3 | 1.5% | frontier_nodes ≡ node_count (see below) |

Phase means ([Blue, Red]): middlegame (n=78): area [31.5, 22.4], closable
[10.9, 11.5], mobility [454, 392], cuttable [7.4, 8.2], nodes [16.3, 15.5].
Endgame (n=22): closable [22.2, 23.2], cuttable [13.8, 12.9], mobility
[601, 572]. Opening (n=3): all terms near zero — the terms are inert before
the first loops close, exactly where the Blue chair lives.

**frontier finding:** the geometric frontier approximation (any on-board,
unblocked direction) equals node count at 100% of sampled positions —
every node always has one free direction; it carries no information and is
redundant with node_count. Killed as a term; kept as a documented negative.

## Playing gates (league n=8 vs scoutbase + gauge vs greedy, probe_league pattern)

<!-- GATE_ROWS -->

## What to merge, what failed

<!-- VERDICTS -->

## Scoreboard rows appended

See `research/league-scoreboard.md` (this branch) for the gate rows.
