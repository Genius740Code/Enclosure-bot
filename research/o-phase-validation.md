# v7 Phase-Trigger Validation (Lane O probe `opp_phase.rs`) — 2026-09-29

**Status: measurement only.** `PhaseTracker` from `research/v7-phase-spec.md`
implemented harness-side (turn-end snapshots, no eval/search changes).
Deterministic, engine legality only (`game.play().unwrap()`, 24/24 games
completed, no illegal moves). Grid per foe line: skip {0,10,20,30} ×
us-color {blue,red} = **8 games (n=8)**, us = retaliator `search::best_move`.

**Line substitution (recorded):** the brief named a "capybara" line; no
capybara mimic exists anywhere in-tree (grep over
retaliator/bots/harness/vendor empty). Third line is **v1base**
(Scout-shaped normal foe) as the quiet FP control. Foe lines:
A collapser = scoutbase rush-closer (CONTEST positive control),
B gbstyle = book wall-builder (WALL-RACE positive control),
C v1base = quiet control. Raw logs: `research/logs_lane_o_phase.txt`
(P0 spec triggers, P0diag +rates, P1 retune).

**Deviations/approximations:** (1) `fresh_foe` = shielded edges touching a
foe node (Edge carries no owner; mirrors search's `near_fresh_enemy`).
(2) Close = foe `MoveKind::Connect` with mover area gain > 0; big close =
gain ≥ 8.0 (GB-anchored MIN_CLOSE). (3) Priority from BUILD:
DENY > CONTEST > WALL > BANK. (4) P1 log-text bug (cosmetic only):
WALL exit message prints "1.5", logic uses the P1 const 0.5.

## P0 (spec-as-written): MISFIRE — 0/24 games leave BUILD before BANK

| line (n=8) | BUILD dwell | CONTEST | WALL | BANK entry | big closes w/ C/W lead |
|---|---|---|---|---|---|
| collapser | 3–6 | 0 turns | 0 turns | t=6–9 (area_sum 25–45) | 0/61 |
| gbstyle | 3–9 | 0 turns | 0 turns | t=6–19 | 0/38 |
| v1base | 3–6 | 0 turns | 0 turns | t=6–19 | 0/120 |

Three compounding causes, all measured (P0diag max trailing 4-turn rates):
1. **WALL `build ≥ 3` is 3× the physical ceiling — unreachable by
   construction.** Foe acts 2 of every 4 turns (mover alternates per
   2-action turn, `mover_after`), so the 4-turn trailing build rate caps at
   ~1.0. Observed max exactly **1.00 in 18/24 games, never above**
   (per-line maxima: 1.0/1.0/1.0; skip30 games 0.33–0.5).
2. **CONTEST `close ≥ 0.75` never matches the rush it was anchored to.**
   Collapser early peak 0.5–0.67 (8/8 games), reaching 0.75 early in 0/8
   (the 3 games touching 0.75+ peak at t=35/45/56 — endgame compounding,
   two excluded by the turn<40 gate anyway). The spec rationale ("closes
   ~every turn early") is falsified: the rush closes ~0.5/turn early plus
   6–15 scattered *small* closes/game.
3. **BANK `area_sum ≥ 25` fires t=6–19 on opening loops in 24/24** and is
   sticky, masking every other phase. Plus MIN_DWELL=4 from BUILD blocks
   the turns-1-2 signature the spec targets (dwell hits 4 only at turn 4,
   by which the 4-turn rate already dilutes).

## P1 (single allowed revision): dwell-bypass BUILD exit, CONTEST 0.75→0.5, WALL 3.0→1.0, BANK +turn≥30 guard

| line (n=8) | BUILD | CONTEST dwell | WALL dwell | BANK dwell | DENY | big closes w/ lead, mean lead |
|---|---|---|---|---|---|---|
| collapser | 10–16 | 5–24 (mean 14.9) | 0 | 17–32 | 0–5 (3/8 games) | 55/61, **18.8 turns** |
| gbstyle | 2–23 | 0–7 (mean 1.8) | 0–29 (mean 8.9) | 0–32 | 0–46 (5/8) | 20/38, 25.6 turns |
| v1base (quiet) | 10–39 | 7–34 (mean **16.9**) | 0 | 0–32 | none | 116/120, 24.0 turns |

CONTEST entries: 13 collapser (8/8 games, first entry t=2–29; 9/13
followed by a big close within 12 turns) vs **10 on the quiet line**
(8/8 games, 8/10 followed — v1base banks 120 big closes total, so any
entry is "followed"). Exit-lag (CONTEST-active turns with trailing rate
< 0.4): 42/119 collapser, 34/135 v1base (~30%).

## Per-trigger verdicts

- **CONTEST: KILLED (non-discriminative, both revisions).** P0 0.75 never
  fires on the rush (0/8 early); P1 0.5 fires everywhere — quiet-line
  dwell (16.9 turns/game) meets/exceeds rush-line (14.9), 10 FP entries in
  8 quiet games. v1base also closes small loops at ~0.5/turn from turns
  2–3; no threshold in [0.4, 0.75] separates the distributions (P0diag:
  rush maxima 0.5–1.0, quiet maxima 0.5–0.75, overlapping). Do not gate
  eval/search on foe close-rate.
- **WALL-RACE: PROVISIONAL VALIDATE at 1.0 (P0 3.0 KILLED as unreachable).**
  P1: 3/8 gbstyle entries (t=2,3,7 — book start; dwells 19–29, exits on
  build halving), **0 entries in 16/16 non-wall games (FP 0)**. Recall
  limited to unshifted books (skip20/30 shift the wall phase out of the
  early window). Needs a recall study, not a threshold change.
- **BANK-RACE: VALIDATED as endgame marker with the turn≥30 guard**
  (P0 unguarded version MISFIRES). P1 entries t=30–45 in 23/24 games
  (one direct BUILD→DENY), dwell 17–32. Note the turn gate does most of
  the work — area_sum ≥ 25 is already met at t=30 in nearly all games.
- **DENY (300/150): VALIDATED.** Fires only on real swings: 3/8
  collapser, 5/8 gbstyle, 0/8 v1base. No churn observed (entry→exit
  hysteresis holds; no game re-entered DENY after exit).
- **Anti-thrash dwell-4: counterproductive from BUILD, fine between
  active phases.** P1 bypass caught the rush at t=2 with 14-turn lead to
  the first big close (collapser blue skip0: entry t=2, first big close
  t=16). Keep bypass-from-default; keep dwell elsewhere.

## Q10 follow-up: phases differ by color (DENY only)

Mean dwell by (line, us-color), B/C/W/K/D: collapser blue
10.8/15.5/0/23.5/**3.8** vs red 14.5/14.2/0/24.8/**0.0**; gbstyle blue
12.5/3.5/5.8/12.2/**19.5** vs red 12.2/0/12.0/21.2/**8.0**; v1base blue
10.0/19.0/0/24.5/**0** vs red 19.2/14.8/0/19.5/**0**. DENY is a winner's
phase: 6 Blue-side entries vs 2 Red-side; as Red we never led by +300 on
collapser/v1base (leads go negative). CONTEST/WALL/BANK are roughly
color-symmetric (WALL gbstyle 5.8 vs 12.0 is small-n noise: 1 vs 2
entries). Lane E/S consequence: DENY-gated weights will engage almost
only as Blue in these matchups — any DENY response must be validated
Red-side separately.

## Handoff (Lane E/S)

- Do NOT implement CONTEST gating on close-rate (killed, both revisions).
- WALL-RACE at `build ≥ 1.0 AND area < 1.0` is the only validated early
  detector (recall 3/8, FP 0/16) — usable for WALL-gated weights with a
  recall caveat; BANK (turn≥30) + DENY (300/150) validated for endgame.
- Determinism note: skip-shifted blue collapser games converge
  byte-identically (rs=1139/1139/1139), as in prior lanes.
