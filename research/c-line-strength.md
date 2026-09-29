# C2 line-strength census — Q8 partial test + measurement spec (2026-09-29)

## 1. What the available data allows (53-loss an/ corpus, 1308 our-closes)

Tested: close gain (area banked on the close move, thickness proxy) vs
survival (not popped >=50% within 6 actions):

| gain bucket | n | survival |
|---|---|---|
| [2, 4.6) | 690 | 54% |
| [4.6, 6) | 34 | 24% |
| [6, 10) | 266 | 36% |
| [10+) | 318 | 5% |

Survived-mean gain 5.62 vs farmed-mean 11.57. Result: **gain NEGATIVELY
predicts survival in the loss corpus** — big closes die (farmed mega-chains),
small 4.5 balloons stand (ignored or game ends). Caveats (load-bearing):
(a) losses only, no win control — cannot tell loss-mechanism from base rate;
(b) gain is banked-area, not structural thickness; (c) touches (shared-node
degree) and true bank-rate are NOT in the an/ JSONs — need engine queries.
Do NOT cite the negative slope as "thin lines survive"; the correct read is
"in losses, the enemy lets small closes stand and eats big ones" — consistent
with Q2's REJECT (endpoint bonus builds big walls scoutbase punishes).

## 2. Measurement spec for Lane E (exact)

New probe `probe_b_census` (eval lane owns it; C2 will consume output):
- Corpus: h2h gate lines (10/game-dose) PLUS league genuine rows (skip0 both
  colors), winner AND loser lines (win-control mandatory after §1b).
- Per OUR close (kind Connect, gain >= 2.0), log at close time: (i) gain;
  (ii) touches = max over the closed loop's edges of (shared-node count with
  our prior structure), from engine position (source-degree/target-degree as
  in E-term code); (iii) bank-rate = score delta over the next 6 own-actions
  / gain; (iv) survived? (area still >= 50% of post-close peak at game end);
  (v) dose + game result + color.
- Output: one JSONL row per close; analysis = survival logistic on
  (gain, touches, bank-rate, dose) with game-result stratification, min n=200
  closes per dose before any claim.
- Gate: a "strength" predictor only counts if it flips a pick decision —
  report pick-flip rate of a term built on it (E-lane lesson: mechanism
  metrics without pick-flips are decoration).
