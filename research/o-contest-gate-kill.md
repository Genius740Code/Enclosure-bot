# Lane O — contest-gen GATE RESULT: KILL (2026-09-29)

**Dose D1** (probe `retaliator/examples/opp_contest_gates.rs` @ tip+1098d9d,
release, deterministic, engine legality only): blue-only contest-set
generation (CUT_SITE + OCCUPY within Chebyshev 2 of foe fresh/shielded wall,
force-pick on full-horizon merit, BONUS=0) when mover is Blue; Red plays
unmodified `search::best_move`. Red cells therefore == control by
construction (verified byte-identical in all 5 h2h + 4 league red games).

## 4-cell gate table (dose vs local control, same binary)

| gate | control | DOSE-D1 | pass bar | verdict |
|---|---|---|---|---|
| h2h vs v1 (10) | 3/10 (blue 0/5, red 3/5) | 3/10 (blue 0/5, red 3/5) | >=6/10 | **FAIL** |
| league scoutbase (8) | +9.9% (blue +24.2%, red -4.5%) | -6.2% (blue -7.9%, red -4.5%) | AVG > control, no row <-300 | **FAIL** (-16.1pp vs control) |
| gauge greedy (6) | 6/6 | 6/6 | 6/6 | pass (non-discriminative, cf. C2 P5) |

Per-game notes:
- H dose flips Blue picks (None blue 934-1055 -> 1110-1389; 4864 blue
  1071-1359 -> 1394-1398; 9199 blue 1262-2027 -> 929-1047) but W/L
  identical 3/10. Our score rises AND foe's rises more — contest moves
  donate tempo against a non-wall opponent.
- L dose blue rows: skip0/10/20 +30.6 -> -17.2 (triple identical,
  deterministic convergence), skip30 +5.0 -> +20.2 (lone counter-flip,
  insufficient). No row < -300 (min -17.2): no collapse, pure value loss.
- G dose still 6/6 but Blue margin shrinks 62.8 -> 37.9 (-25pp, same sign
  as the league-blue damage).

## KILL reading

Contest-gen's 8-0 Blue (+17.8%) was matchup-specific: it prices
construction-site fights that only exist when the foe builds big walls
(collapser). Against v1 (no wall book) and scoutbase (floor-0 farmer) the
same picks are anti-value as Blue (-32.1pp league-blue swing) while Red is
untouched. This is the mirror of C2 P1: a dose that fixes the lane's
showcase matchup and breaks genuine games. "Contest-and-out-bank" does not
generalize beyond wall-building foes.

## Handoff

- Do NOT port contest-set generation to search.rs (no port spec written).
- The approved Blue dial remains Lane E's FRESH_PENALTY (price the occupy,
  not the generation).
- Opp-mimic track closed except the site-game census prediction test
  (v7-gb-census-spec.md), run separately.
