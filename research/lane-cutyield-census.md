# CUT-YIELD census — lane-v9-cutyield (v9 roadmap lane 3) — 2026-10-01

**Lane:** 3 (cheap, analysis-only). **Probes:** `retaliator/examples/probe_cutyield.rs`
(+ `examples/support/gamejson.rs` shared loader). Read-only, no src edits.
**Engine for probes:** master 34baf47 (v8 stack: mesh8 + doom-ON + tie-break D1),
fixed node budgets (deterministic; never `analyze_timed`).

**Kill-0 question (roadmap):** replay flip census ≥10% of zero-yield cuts →
proceed to dose (ZERO_CUT_PENALTY=1.0 × hz); else KILL (Lane-E disease).

## Definition

A **zero-yield cut** = a first action that breaks an enemy edge while
destroying 0 enemy area AND gaining 0 own area (both against a 1e-9 area
tolerance): the cut costs a tempo and produces nothing on either axis. The
broken edge supported no enclosure and the placed edge enclosed nothing.

## Corpus

46 local sparring games (repo root `chall-*.moves.json` / `R-chall-*.moves.json`),
5520 actions, recorded by the pre-Riposte JS bot line. **v8 eval games (vs GB /
VladNet / AngelBot-WASM) have NOT landed yet** (queued 2026-09-30, pairs=2) —
the probe accepts site-format JSONs (`moves` id array) and can be re-run on
them when they land. The roadmap's "113 zero-yield cuts" figure does not match
this corpus (we count 214); different corpus or stricter definition — the flip
RATE is the decision input and it fails the gate by an order of magnitude
either way.

## Census (46 games)

| | cuts | zero-yield | zero-yield share | destroyed from cuts | avg destroyed/cut |
|---|---|---|---|---|---|
| **us** | 566 | **214** | **37.8%** | 521.6 | 0.92 |
| **opponent** | 342 | 42 | 12.3% | 721.9 | 2.11 |

Our zero-yield cuts destroy 0.92 area on average vs the opponent's 2.11 — our
cut portfolio is nearly half junk by count and carries less than half the
denial value per cut.

**Zero-yield rate by outcome (our cuts): wins 177/356 = 49.7%, losses 37/210 =
17.6%.** Zero-yield cuts are a DOMINANCE ARTIFACT, not a loss driver: they
happen most in games we win big (vs random/sprawl/turtle — cutting dangling
edges for nothing while the board is already ours). In losses (vs aggro/neck)
our cuts mostly carry real destruction. A blanket ZERO_CUT_PENALTY would
mostly punish winning positions.

## Per opponent (us cuts / zero-yield | opp cuts / zero-yield)

| opponent | us cuts / zy | zy share (us) | opp cuts / zy |
|---|---|---|---|
| random (20 g) | 261 / 123 | **47.1%** | 11 / 5 |
| junk (2 g) | 44 / 35 | **79.5%** | 0 / 0 |
| sprawl (4 g) | 11 / 8 | 72.7% | 7 / 1 |
| turtle (4 g) | 7 / 5 | 71.4% | 0 / 0 |
| aggro (4 g) | 91 / 24 | 26.4% | 159 / 26 |
| neck (4 g) | 111 / 14 | 12.6% | 160 / 10 |
| blob (4 g) | 39 / 4 | 10.3% | 4 / 0 |
| sandbag (4 g) | 2 / 1 | 50.0% | 1 / 0 |

vs weak/passive bots (random, junk, sprawl, turtle) roughly HALF our cuts are
zero-yield; vs the aggressive wall bots (aggro, neck) the share collapses to
13-26% — against real opposition our cuts get real targets.

## Flip census (209/214 measured; per-game cap 16, one game capped 16/21)

Method: at each recorded our-side zero-yield-cut decision point, two
continuations run from the same position —

- **CONTROL**: the plain current-engine best move (budget 4096);
- **DOSE-CF**: the best NON-zero-yield alternative (the dose's intent,
  "zero-yield cuts are not playable"; budget 4096, walk candidates best-first,
  production-value fallback if all 8 searched candidates are zero-yield).

Both continue with the current engine (our side, budget 1024) while the
opponent replays its recorded moves where still legal, else an engine stand-in
(fallbacks counted). A **flip** = outcome class (win/loss/tie) differs between
DOSE-CF and CONTROL — clean attribution, same continuation, one move
different.

| measure | count | rate |
|---|---|---|
| zero-yield decision points measured | 209 | (of 214 recorded; 97.7% coverage) |
| **dose targets** — current engine ALSO plays a zero-yield cut there | **24** | 11.5% of measured |
| outcome flips vs CONTROL | **2** | **1.0% of measured / 0.9% of all 214 / 8.3% of dose targets** |
| outcome flips vs RECORDED game (drift-inflated) | 19 | 9.1% of measured |
| opponent stand-in fallbacks | 2308 | (attribution caveat) |

The 2 real flips are 1-for-1 in DIRECTION:

- `R-chall-neck-botIsred` zy@27: recorded/top P10-Q8 (zy) → dose Q7-N4 →
  **dose WIN (+239) vs control LOSS (−99)** — skipping the zero-yield cut won.
- `chall-neck-botIsblue` zy@16: recorded/top D10-G10 (zy, engine agrees) →
  dose D10-G9 → **dose LOSS (−247) vs control WIN (+84)** — skipping the
  zero-yield cut LOST: the "zero-yield" cut carried denial value the
  area-only axes cannot see (its placement blocked the enemy re-make).

The vs-RECORDED number (9.1%) is inflated by engine drift: the corpus was
played by the old JS bot line, and the current engine's continuation alone
changes 19 outcomes (2308 opponent-move fallbacks). It is NOT the dose's
doing.

## Kill-0 verdict: **KILL**

Gate: ≥10% of zero-yield cuts flipping. Measured: **0.9%** of all recorded
zero-yield cuts (2/214), 8.3% of dose targets (2/24) — an order of magnitude
below the gate. Three reinforcing reasons:

1. **The current engine already avoids the disease.** v8's term stack
   (IDLE/DEADWOOD/CONTACT penalties + full-horizon valuation) already
   suppresses zero-yield cuts at 185/209 (88.5%) of the recorded decision
   points — only 24 dose targets remain.
2. **Where the dose does act, it is a coin flip.** 2 outcome flips of 24
   targets, one improving and one REGRESSING — the second proves some
   "zero-yield" cuts carry invisible denial value (placement blocks the
   enemy re-make path). A blanket penalty burns that value.
3. **Zero-yield cuts correlate with WINS (49.7%) not losses (17.6%).** The
   population the dose would tax is concentrated in dominating positions
   where the tempo is surplus anyway. Punishing it cannot recover losses.

Same disease family as PATIENCE-off (byte-identical, V5-1) and CUT_MEMORY
6→15 (byte-identical, P-dose-2): a flat penalty on a symptom that does not
drive outcomes. **Do not build ZERO_CUT_PENALTY.** If the junk-cut symptom
matters at all, the actionable residue is the 24 remaining dose targets vs
weak bots only — a much narrower price (e.g. zero-yield penalty gated on
enemy-aggression absence) would need its own kill-0; not recommended on this
evidence.

## Caveats

- Corpus = old-bot local games; v8 eval games will change the mix (the probe
  re-runs on them unchanged). The flip census needs the SAME re-run when they
  land before anyone reopens the lane.
- The DOSE-CF "alternative" is implemented as a hard ban, not the literal
  1.0×hz penalty: a cut whose adjusted lead beats the alternative by more than
  1.0×hz would not flip under the literal dose either, so the ban is an UPPER
  BOUND on the dose's flip count — the true rate is ≤0.9%, strengthening the
  KILL.
- Opponent continuation uses recorded moves where legal (fallbacks 2308 of
  ~25k opponent actions — 9%); the stand-in engine inflates the vs-RECORDED
  number only, never the vs-CONTROL attribution.
