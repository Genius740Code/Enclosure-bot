# XBot pick-and-mix (source: /tmp/opencode/xbot/meridian_bot, user-supplied 2026-09-28)

XBot = from-scratch Rust bot with its OWN engine (board.rs/area.rs, NOT
meridian-engine) + ID negamax alpha-beta + TT + time budget + 14 eval presets.
README admits: "shallow search rarely closes loops, scores ~0", preset
tournament "uninformative". Rule fidelity vs site engine UNVERIFIED.

## Verdict: DO NOT play cross-engine (invalid). PORT IDEAS into our engine.

XBot's engine may mis-score cuts/shields (its README assumptions differ from
RULES.md rule 5). A h2h across engines proves nothing. Everything below is a
port spec for our lanes, measured in OUR gates.

## Steal 1 — one_move_potential (Lane B). HIGHEST VALUE, matches user complaint.
`eval.rs:52-70`: count own frontier-node pairs within a king-step-3 box
(single legal action could join them into a loop) = cheap "about to score"
proxy. Differs from our loop_bonus (triangle-specific, first-action only):
this is global, both sides, every node. Port form (full-horizon units!):
POT_W * (pot_me - pot_opp) * hz, sweep POT_W. Directly attacks "connects the
wrong line / misses area": a connect that joins a frontier pair gains potential
even before it closes. Ablate on/off, watch Blue chair + collapse worst.

## Steal 2 — captures > cuts > closes ordering (Lane A2). FREE.
`search.rs:34-55`: order root/child actions by captures(+100) > cuts(+50) >
own-loop closes(+30). Our search_deep ordering is priority-based; try this as
the TT/ID move-ordering seed. Measure nodes-to-depth + time/move before any W/L.

## Steal 3 — ID + TT + wall-clock budget skeleton (Lane A2). CONFIRMS DIRECTION.
`search.rs:113-167`: iterative deepening 1..12 under a Duration budget, TT
keyed by FNV-1a over sorted edges + to_move + actions_left, keep-last-good on
abort. Same architecture A2 is building on the real engine. No port needed —
use as a code-review checklist against search_deep.rs (abort keeps previous
depth's best; TT stores (depth,value)).

## Steal 4 — preset-tournament methodology (Lane D style). MEDIUM.
14 weight presets × round-robin to find the style gradient (their tournament
failed only because nothing closed). Our equivalent: sweep ONE weight at a
time in eval_phases (already our method) — plus try their "endgame_closer"
shape (potential 8.0 > area weight) as a phase-gated variant late-game.

## Explicitly DO NOT steal
- Flat area/score weights (area 6-12, score 100): no horizon math, violates
  iron rule 1 (their own scores ~0 show it).
- Their engine, movegen, area calc: unverified vs site rules.
- chaos_random / banked_greedy presets: noise.

## Execution
Next free lane slot -> Lane X (pick-and-mix): port Steal 1 into eval_phases
(one term, ablation), Steal 2 into search_deep ordering (timing first).
Files: eval_phases.rs + probe_b_x.rs on lane-b-eval; search_deep.rs ordering
only on lane-a-search. Scoreboard rows required.
