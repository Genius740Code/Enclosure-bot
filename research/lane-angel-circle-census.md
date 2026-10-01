# ANGEL-CIRCLE census — lane-v9-cutyield (v9 roadmap lane 7) — 2026-10-01

**Lane:** 3 (cheap, analysis-only — the roadmap assigns lane 7 to "analysis lane
first, dose only after kill-0"). **Probe:** `retaliator/examples/probe_angel.rs`
(+ `examples/support/gamejson.rs` shared loader). Read-only, no src edits.

**ANGELWASM GAMES NOT AVAILABLE YET** (v8 evals queued 2026-09-30, pairs=2, not
landed). This report: definition + probe ready + preliminary baseline on the
46-game local corpus. Re-run the probe on the eval games when they land for the
AngelWASM-specific census.

## Definition

A **circle** = after an opponent close (a Connect gaining real area, > 0.5),
the opponent's enclosed territory contains at least one of our nodes. Trapped
nodes can only expand inside the enemy wall (crossing an enemy edge is
illegal), which is the encircle-and-bank shape the roadmap targets.

Territories are the engine's own face computation (union of closed loops),
ported from the site JS engine (`engine.js` `nv`/`_0`/`A0`/`O0`): planar graph
per connected component, directed-edge outer walks with positive signed area,
components nested inside another's face dropped. **Port validation: 92 checks
(final edge sets vs engine `area()`), 0 mismatches** — the probe sees exactly
what the engine's areas see.

## Census (46 local games, 5520 actions)

| | count |
|---|---|
| opponent closes | 899 |
| opponent closes gaining real area | 425 |
| **circles** (a close trapping our nodes) | **4** in 3 games |
| circles / gaining closes | 0.9% |
| circles / game | 0.09 |

Circles are RARE in the local corpus — and all 4 came from the two
wall-building bots:

| opponent | games | circles | circles/game |
|---|---|---|---|
| neck | 4 | 3 | 0.75 |
| blob | 4 | 1 | 0.25 |
| aggro / random / sandbag / sprawl / turtle / junk | 38 | 0 | 0.00 |

## Shape (the 4 circles)

| game | action | gained area | our nodes inside | opp edges | deg≥2 (thickness) | phase | outcome |
|---|---|---|---|---|---|---|---|
| neck-botIsred | 44 | 9.7 | 2/8 | 16 | 11 | mid | OPP WINS |
| neck-botIsblue | 62 | 12.1 | 4/9 | 17 | 12 | mid | OPP WINS |
| neck-botIsred | 96 | 3.8 | 2/12 | 32 | 20 | late | OPP WINS |
| blob-botIsred | 96 | 21.0 | 3/30 | 40 | 19 | late | **WE WIN** |

- **Size**: mid loops (8-30 area) ×3, small (<8) ×1, big (≥30) ×0. The circle
  is not one giant sweep — it is a modest loop closed AROUND a small cluster
  of ours (2-4 nodes, all four circles).
- **Phase**: mid (31-80) ×2, late (>80) ×2, early ×0 — circles come AFTER the
  opening, once our cluster exists to trap.
- **Thickness**: avg 15.5 opponent nodes with degree ≥ 2 (shared-node walls —
  near-unbreakable per E3). The circles are embedded in thickets, not lone
  petals.

## What breaks it

| measure | count |
|---|---|
| circles with ≥1 cuttable boundary edge at the first our-turn | **4/4** (avg 2.5 cuttable edges, max destroyed 7.6) |
| circles where we cut a circle edge | 4/4 |
| **circles where the opponent's bank fell back (area below pre-circle)** | **0/4** |

The decisive number: **we cut a circle edge in all 4 circles and the bank
never fell back in any of them.** Cutting the circle does NOT un-bank the
score — scores bank at the scoring event; a later cut only removes future area
value. (One circle, neck-botIsblue @64, had its single boundary edge
UNBREAKABLE at the first our-turn — no legal cut could reach it; the wall was
a shared-node thicket edge.)

Consequences for the dose path:

1. **Nothing AFTER the close works.** The bank is permanent; contesting the
   circle post-close is donated tempo — the same failure that KILLED the v8
   W-lane corridor contest (league −27/−30%, threshold-insensitive).
2. **Only prevention (before the close) can matter.** A prevention dose would
   need to detect the circle 2-4 moves early and outbid our best area move —
   the M9 threat-detection discipline. Its kill-0 (detectable signature in
   losses) has NOT run; on this corpus the event is rare (4/46 games, 0.9% of
   gaining closes), so any prevention dose fires rarely locally.
3. **Being circled correlates with losing** (circled games 1/3 = 33.3% win vs
   uncircled 30/43 = 69.8%) — n=3, far below the n>=8 law; directional only.

## Verdict: analysis complete, dose NOT justified on this corpus

The roadmap's kill-0 for lane 7 (dose only after a census that shows the event
is common enough to price) fails locally: 4 circles in 46 games is below the
n>=8-events bar for any dose decision, and the only actionable lever
(prevention) belongs to the M9 discipline whose kill-0 hasn't run. The
post-close lever is measured DEAD (bank never falls back — 0/4).

**Next:** re-run `probe_angel` on the v8 eval games when they land
(AngelWASM a2d70517 is the priority target — "circles us" is AngelWASM's
signature per the roadmap). If AngelWASM circles at a meaningfully higher rate
(and the circles are detectable pre-close), open a prevention-dose kill-0 with
the M9 discipline. Until then: no dose, no engine change.

## Caveats

- Corpus = old-bot local games vs challenge bots; zero AngelWASM/VladNet/GB
  games. All AngelWASM-specific claims await the eval games.
- The circle boundary-edge test maps face-boundary stretches back to opponent
  edges (T-junction walls are made of edge parts; a both-endpoints test
  measured 0 of 5 on circle@44 and was replaced by the stretch test).
- The trap test is the engine's own territory semantics (union of closed
  loops) — validated 92/92 against `area()`.
