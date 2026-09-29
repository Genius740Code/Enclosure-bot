# C2 v7 gate review — where doses die (2026-09-29)

Tips read: `origin/lane-v7-search` @2b2c884 (S-Q14), `origin/lane-v7-eval`
@8d152f0 (E REJECT), `origin/lane-v7-opp` @1799112 (O contest-gen). Gates:
h2h = 5 openings x 2 colors = 10 vs v1 (pass >=6/10); league = scoutbase
skip 0/10/20/30 x 2 colors = 8, pass AVG > control with no row < -300
(diff); gauge = greedy 6 (or 8 in eval-lane probes), pass 6/6.

## 1. Where each dose died

### Lane S (budgeted depth-3 search, lane-v7-search)
| dose | h2h | league | gauge | dies WHERE |
|---|---|---|---|---|
| control (capped d3, 2s) | 2/10 (blue 2/5, **red 0/5**) | -96.4% | 6/6 | h2h + league; **red-only collapse** (red rows -130..-326, blue rows +37/+37/-28/+45) |
| S0 (aspiration mover-fix) | 2/10 (identical W/L) | -108.1% (red worse) | 6/6 | same place; fix verified real but NOT causal |
| S-Q14 (killer+history) | value-exact 0 mismatches | nodes +2.5%/+5.4% totals (gate >20% FAIL) | n/a | perf gate; -77% only where cutoffs exist, totals dominated by unprunable t=13 |

### Lane B/E (eval terms, lane-v7-eval; control E0 = doom-OFF = 5/10, -9.9%, 6/6)
| dose | h2h | league | gauge | dies WHERE |
|---|---|---|---|---|
| B-1 vuln ON (1.0) | 3/10 | -25.5% | 8/8 | everywhere (worse on all gates incl. blue chair -13.2pp) |
| B-2 doom 0.5 | **6/10** (only ship-threshold pass) | **-27.3% (worst)** | 8/8 | league + collapse rows -137.7% — non-monotone vs doom 0.0 |
| B-3 flat mem 12/20/40 | 4/10 each | -6.8/-9.3/-11.7% | 8/8 each | h2h (-1 everywhere); league non-monotone in memory |
| B-3 ScaledAll A mem6 | 4/10 | **+1.2%** (collapse FIXED -270/-270/-306) | 8/8 blue +76.0 | h2h only (-1: red 11723 flips on -36 diff) — session's one big positive, kept as near-miss |
| B-3 ScaledAll B mem 6/12/20/40 | 3/2/3/2 | -25.4/-31.8/-47.6/**-70.7%** | 8/8 | everywhere; monotone-catastrophic in memory |
| E1 rescue (1,1) | 6/10 | -24.9% | n/g | league; skip0-blue +29.0 -> -30.3 |
| E2 create-only | 5/10 | n/g | n/g | h2h (control-level, never flips) |
| E3 extend-only | 6/10 | -25.9% | n/g | league; EXTEND carries the damage |
| E4 half-extend (0.5,0) | 6/10 | -20.6% | n/g | league; monotone-negative dose-response 0.0->-9.9, 0.5->-20.6, 1.0->-25.9 |
| E5 v2 non-breaking (0.5,0) | 6/10 | -15.3% (best of sweep) | n/g | league; 4 rows back to control, skip0-blue still -36pp |
| E6 v2 quarter (0.25,0) | n/g | -19.1% | n/g | league; single-row flip skip0-blue +29.0->-46.0 |

### Lane O (opp-mimic, lane-v7-opp)
- Collapser repro: skip=0 as Blue 1-7 (-11.1%), 7/8 losses — dies on **genuine
  (skip=0) blue**, the one place the scoutbase league control wins.
- CAPTURE_W ablation: -20.5pp if removed (critical defense term).
  FRESH_PENALTY/DENSE_BONUS/HORIZON_WEIGHT removal HELPS vs rush-closer
  (+5.8/+6.9/+3.7%) — terms that hurt in exactly one matchup family.
- Contest-gen n=32: Blue 8-0 +17.8% but **conversion 0/32** (wall always
  closes); cut volume identical (~35-38/game) so the win is selection.
- Lane D tournament: scout-style most dangerous (13-11 overall); v2+avoid
  PROVEN vs farmer (6-2 -> 8-0) but REGRESSES vs sac (7-1 -> 6-2).
- D2: sac takes 7/32 (3 off shipped v3); blob 0/32.

## 2. Patterns across lanes

P1. **Red is where search doses die; skip0-blue is where eval doses die.**
S-control/S0 red 0/5 with blue rows winning; E-doses pass h2h 6/10 yet
collapse league via skip0-blue (+29 -> -30..-46). Different gates, mirror
failure: each lane's dose fixes its home gate and breaks the other color's
genuine game.

P2. **The scoutbase skip=10/20-red collapse is the universal graveyard.**
Baseline -111.2%, doom-OFF -94.5%, B-3-form-A -270 diff (fixed), E-doses
-99..-120, S-rows -130..-392. Every dose is scored first by whether it
moves these two rows; only form-A mem6 ever got both inside -300, at the
cost of one h2h game.

P3. **Non-monotone single-row attractors.** Doom 0.0/0.5/1.0 league
-9.9/-27.3/-20.0; E skip0-blue +29.0/-46.0/-7.4/-33.9 across 0/0.25/0.5/1.0;
B-3 league vs memory -6.8/-9.3/-11.7 (flat) and -25.4/-70.7 (scaled). Dose
sweeps cannot be interpolated — one early pick-flip swings +/-40pp.

P4. **h2h-pass + league-fail is the modal outcome** (B-2-0.5, E1, E3, E4,
E5, B-3-form-A). The v1 h2h and the scoutbase league punish opposite
things; no tested dose beats control on both jointly except doom-OFF
itself (5/10, -9.9%).

P5. **Gauge never fails** (6/6 or 8/8 everywhere, incl. S control). It
screens for crashes/byte-identity only — no discriminative power; do not
cite it as support.

P6. **Q2 REJECTED mechanism-confirmed**: life probe shows lifetime +11%,
bpb +11%, breaks flat at E4 — the unbreakable bonus fires directionally
but steers into scoutbase-punished lines. Mechanism metrics passing while
gates fail = the bonus prices the wrong thing (endpoints, not survival).

## 3. Open questions for lanes A/B/E
- Q1 break-fix valuation (charge only ~2 missed events, not x12 doom):
  untested; the one eval direction with mechanism + gate alignment pending.
- B-3 root cause (avoid slice carries no COUNTS; per-edge cut-counter form
  needs API change) — blocks the only league-positive direction (form A).
- S red-collapse: aspiration ruled out; Q14 kept (strictly positive per
  second); Q16/Q6 pending. Red concessed 2000-4000 to 2-ply baseline as
  Blue — suspect evaluation sign/perspective or width, not ordering.
- O handoff: contest-gen Blue-only (conversion unsolved 0/32); CAPTURE_W
  must not be touched by Lane B adjustments (FRESH/DENSE/HORIZON are the
  approved dials vs rush-closers).
