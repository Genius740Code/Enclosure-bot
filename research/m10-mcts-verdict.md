# M10 MCTS pilot verdict (external Claude session, 2026-09-30) — KILL

Setup: 20 mid-game positions (actions 12-59) from 3a414aad. UCT + progressive
widening, 2,200 sims/move. Root = beam top-8 + own candidates (16). Rollouts:
8-ply best-of-3 on `evaluate` + sigmoid leaf. Opponent: fixed 2-ply beam
`analyze(pos, 4096)` (NOT the stronger timed beam). Each position played twice
(pilot Blue + pilot Red). Cost: ~0.9s native/move, ~2.8s at 3x WASM — fits the
4.8s hard cap (budget OK, strength fails).

## Result (13 of 20 positions, 26 games)

Pilot 13/26 = **50.0%** vs >50% bar. 9/13 positions colour-decided splits (1-1,
uninformative). Informative positions 2-2 (pilot won pos 3, 9; lost 12, 13).
Pilot pick matched beam first choice only ~31% (deviates, doesn't convert).
Setting favours beam (opponent IS the beam, so its reply model is exactly right;
vs real top bots M7 match was 7.5-32.5%). Uncalibrated: 8-ply rollouts, leaf
sigmoid scale 40, small sample (±1 game = noise). Positions 14-19 unfinished
(see /tmp/m10_run2.log on that box, not committed).

## Verdict: KILL

MCTS with `evaluate`-only leaves gets no better information than beam minimax
over the same static eval, and does nothing for the M7 reply-model gap (39.5%
match). Effort goes to the M7-gated opponent-aware reply model (M6 rival strat
files), not MCTS. Revisit only with a learned/opponent-conditioned rollout
policy. M10 CLOSED.
