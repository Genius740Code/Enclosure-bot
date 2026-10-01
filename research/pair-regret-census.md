# Pair-regret census (external session, 2026-10-01) — GO (partial: 5/12 games)

Q: on OUR double turns, does the jointly-best pair beat greedy-then-replan?
Kill-0: mean regret <2.0 area, or >80% doubles ~0 regret. Measured: mean **9.68**
(4.8×), ~0 in **2.2%** (3/136, 37× under bar). GO, both criteria fail wide.

Per-game (doubles, mean, max): 16840108 29/11.28/41.50; 1c69056c 30/10.75/26.57;
21fc2945 23/6.56/13.12; 30c7653b 30/7.07/21.50; 9c3b27a5 24/12.64/35.43.
Flat across phases (open 10.44 / mid 9.15 / late 9.34) and colors (blue 10.28 /
red 8.91). 75% of turns exceed 5.0.

Decomposition regret = gap1 + gap2: gap1 = −1.49 (best static 1-ply WORSE than
played — PV already sees further); gap2 = +11.17, positive 136/136 (100%) —
almost all value is JOINT, unrecoverable by single-move improvement. Optimal
pair structural 136/136.

Method: exact search (9.16M pair evals, triple-validated: legality 275k/0
mismatch, evaluator 156k/0 err, spur-prune 48 turns/0 diff; first level
deliberately unpruned). D = ownArea − oppArea delta over turn. Sizing: ~9.7 ×
~29 doubles ≈ 263–330 area/game; 66%/45% of losing margins in close games.
Cost: 67k evals/double turn (~15–50s) vs 2s budget — D1 must be two-level beam
(~24–32 × ~40 ≈ 1.3k evals ≈ 0.3–1s); gate D1 by recall-of-pair-opt on these
136 states. Caveat: static area-delta proxy, not outcome A/B; gap1<0 shows
1-ply area-delta is a poor move-1 heuristic (consistent with BEAM-SEED).

PARTIAL: 5/12 games (CPU contention); resumable (results.jsonl keyed
game|turnIdx); ~3h for remaining 7.
