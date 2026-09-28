# Lane B2: continuous close-size weight (Lane D H-B-VLAD1) x doom-off — 2026-09-28

Branch: `lane-b-eval` (worktree `game-b2`, local branch `work-b2`, push
HEAD:lane-b-eval). Rig: `retaliator/src/eval_phases.rs`; probes `probe_b_v1`
(v1 h2h, 10 games) / `league_b` (8) / `gauge_b` (8) / `probe_b_close`
(pick-flips + close sizes). All numbers from this session; deterministic bots
(control reproduces lane-b's doom-off rows exactly — see below). Ownership
rules respected: only `eval_phases.rs` + `probe_b_*` / `league_b` / `gauge_b`
+ this file + scoreboard appends.

## The term (CLOSE, H-B-VLAD1)

Lane D hypothesis: our closes bank ~0.2-0.9/close in the farmed losses while
the mimic's greedy close banks ~2-4; the eval's flat close treatment makes
tiny closes competitive. Form tested: for a first-action Connect,
`priority += CLOSE_W * (own_gain - CLOSE_T) * hz` — continuous in gain, all
game (no opening window; the difference from PATIENCE), referenced to the
threshold CLOSE_T: closes below it lose (CLOSE_T - gain) x hz, closes at or
above it gain (gain - CLOSE_T) x hz, so a 0.5-area close never outranks a
wall that leads to a 3+ area close. Full-horizon units (area x
min(events,12)); deterministic tiebreak by move index; legality via engine
only. Applied to our candidates only (first-action ranking + selection,
like REBUILD/FRESH); the opponent's reply model is the shipped skeleton's.
`CLOSE_W = 1.0` fixed; CLOSE_T swept 1.0/1.5/2.0/3.0. `ablation_best_move`
= doom OFF + close OFF (the same doom setting as the sweep) is the ablation
control; `baseline_best_move` = the shipped search (faithfulness control).

## Faithfulness (re-verified after the restructure, before any claim)

`probe_b_h2h`: control (all added terms off, doom ON = shipped) vs shipped
`search.rs`, position for position over 2 full games — **0 mismatches in 239
positions** after the CLOSE restructure (matching lane-b's 239-position
baseline). Control path is code-identical (the term is flag-gated); the
restructure added only the `close` parameter plumbing and the gated term.

## Ablation control: doom OFF + close OFF (same-session numbers)

Reproduces lane-b's doom-off rows EXACTLY (deterministic bots):
v1 h2h **5/10** (blue 1/5 avg -16.7%, red 4/5 avg +9.2%), league **-9.9%**
(blue +29.0/+29.0/+29.0/+38.9 = 4/4 wins; red +39.8, then the collapse rows
skip=10/20/30 red **-94.5% (1098/2136) / -94.5% (1098/2136) / -55.9%
(843/1315)**), gauge **8/8** (blue +53.5, red +69.3). Collapse score diffs:
**-1038 / -1038 / -472**. Logs: `research/logs/b2_control_*.txt`.

## Sweep: CLOSE_T at doom OFF, CLOSE_W=1.0 (one variable at a time)

| CLOSE_T | v1 h2h W/L (blue/red) | league margin % | collapse rows (red, margin % / score diff) | gauge W/L | verdict |
|---|---|---|---|---|---|
| off (control) | 5/10 (1/5 -16.7% / 4/5 +9.2%) | -9.9% | -94.5 -1038 / -94.5 -1038 / -55.9 -472 | 8/8 | baseline |
| 1.0 | 4/10 (1/5 -29.0% / 3/5 -0.4%) | **-32.3%** (blue 1/4: skip0/10/20 **-13.0%** losses vs control +29.0% wins) | **-124.3 -1215 / -124.3 -1215 / -17.5 -177** (two worse, one better) | 8/8 (blue +60.0, red +69.4) | worse on league + v1 + as-Blue genuine |
| 1.5 | 4/10 (1/5 -30.8% / 3/5 -1.6%) | **-33.0%** (blue 1/4: skip0/10/20 **-13.0%** losses; skip30 +9.8%) | **-124.3 -1215 / -124.3 -1215 / -17.5 -177** (identical rows to 1.0) | 8/8 (blue +60.7, red +69.4) | worse on league + v1 + as-Blue genuine |
| 2.0 | 4/10 (1/5 -23.7% / 3/5 -1.3%) | **-46.1%** (blue 1/4: skip0/10/20 **-24.7%** losses, -236 diff; league W/L 2/8) | **-160.4 -1412 / -160.4 -1412 / -17.5 -177** (worst collapse diff) | 8/8 (blue +60.7, red +69.4) | worst league of the sweep so far |
| 3.0 | 3/10 (1/5 -21.2% / 2/5 -4.5%) | **-33.6%** (blue 4/4 but weaker: +10.3/+10.3/+10.3/+15.2 vs control +29.0..+38.9) | **-153.8 -1361 / -153.8 -1361 / -45.9 -354** (all worse than control) | 8/8 (blue +70.7, red +69.4) | worst v1 of the sweep; still worse than control |

## Sweep verdict: the continuous close-size weight FAILS at every threshold

All four CLOSE_T doses are worse than the close-off control on league and v1
(dose response non-monotone in league: -32.3 / -33.0 / -46.1 / -33.6; v1
4/4/4/3 vs control 5/10). The uniform damage pattern: the as-Blue genuine
league games (skip 0/10/20) flip from +29.0% wins to -13.0%/-24.7% losses
(the (gain - T) bonus double-counts area on normal closes — our typical
close banks ~+7, so it gets +4..+6 x hz extra on top of the area x hz the
eval already counts — and the bot closes too eagerly as Blue), and the red
collapse rows get worse, not better (the term does NOT fix the tiny-close
disease it targets — the tiny re-closes are already below threshold and get
penalized, but the farmed losses come from cut geometry, not close choice).
gauge 8/8 equal at every threshold (the blob archetype is beaten either
way). The control (close OFF) is the best config of the sweep. The
one-sided form (tiny-close penalty only, no big-close bonus) is untested;
the two-sided form's bonus side dominates its behavior. See the dose test
below for the doom-0.5 interaction.
| 1.5 | (pending) | | | | |
| 2.0 | (pending) | | | | |
| 3.0 | (pending) | | | | |

Targets: league_b >=6-8 W vs scoutbase; no single collapse loss worse than
-300 score diff; v1 h2h >=6/10; as-Blue chair (problem #1) watched (doom-off
cost 2/5 -> 1/5 — does close-weight recover it?). n>=8 both colors per gate.
