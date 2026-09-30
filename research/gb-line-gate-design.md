# GB-line gate design (external session, 2026-09-30) — method (a) + Stompy controls

## PREREQUISITE defects (fix before ANY ON measurement)

1. init_opp_close_tracker assumes strict alternation; game runs two-action
   turns (B,R,R,B,B,R — 59 non-alternations/game). Wrong-side attribution 2/3
   games both directions. Fix (~3 lines): current_player = pos.to_move()
   before apply_unchecked; drop the to_move param.
2. Tracker never fed in h2h path (called only from best_move_routed;
   h2h_base drives analyze_with_avoid directly) — ON arm structurally
   unreachable in gates; OPP_MOVES stays 0. Fix at HARNESS level: gb_gate.rs
   drives best_move_routed (single-threaded; process-global atomics).

## Recommendation: (a) counterfactual replay + (b) Stompy controls

gb_gate.rs from autopsy.rs (exact replay) + h2h_base (BUDGET=4096, chairs):
arms RECORDED (bit-exact or abort) / OFF / ON, 3 GB + 2 Stompy games, mirrored
block reported separately. Per row: gb_like fires, deny_flip_count,
bank_completed, our area at bank ply, final scores, opp_moves_rejected.
Kill bars: mechanism null (bank completes, area not ≥1 cell above OFF) /
payoff null (blocked but margin not better) / harness void (rejections >15 or
RECORDED inexact — fix once, then park) / control leak (Stompy flips cap the
production claim). Keep bar: blocked ≥2/3 AND margin better, rejections <15 —
still counterfactual-only, no Elo claim; phase-4 falsifier stays open.
(c) scripted mimic REJECTED (circular, uninterpretable null, contested shape).

## Dose-design flaws flagged (for wire v2)

DENY fires on 58–88 eligible cells (any border-near-enemy first action), not
the open-segment rule — measures "border-adjacent good vs low-closers", not
the prep-book denial. Same weight tier as REMOTE_BONUS (go FAR from enemy):
partial cancel. Narrow to open-segment test or accept proxy semantics.
