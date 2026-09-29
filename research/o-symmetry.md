# Q18 Symmetry Probe Results

## Configuration
- Base positions: 10 diverse positions (start, openings, mid-game)
- Symmetries per position: 8 (D4 group: 4 rotations + 4 reflections)
- Total symmetric positions: 80 = 10 × 8
- Orbits found: 6 (expected ≤ 10)
- Search budget: MOVE_BUDGET = 4096

## Results Table

| Orbit | Size | Eval | Nodes | Base Position |
|-------|------|------|-------|---------------|
| 0 | 40 | 0.000 | 621 | 0 |
| 1 | 8 | 46.200 | 1088 | 1 |
| 2 | 8 | 36.600 | 1237 | 2 |
| 3 | 8 | 36.600 | 1502 | 3 |
| 4 | 8 | 17.450 | 1641 | 4 |
| 5 | 8 | -11.167 | 1597 | 5 |
| 1 | 46.2 | 1088 | 8 | 5 |
| 2 | 36.6 | 1237 | 8 | 6 |
| 3 | 36.6 | 1502 | 8 | 7 |
| 4 | 17.449999999999996 | 1641 | 8 | 8 |
| 5 | -11.166666666666671 | 1597 | 8 | 9 |

## Aggregate
- Total full nodes (all 80 positions): 81360
- Total orbit nodes (1 per orbit): 7686
- Node savings: 90% (theoretical max: 87.5%)
- Max eval diff (orbit rep vs full): 0.000000
- Symmetry violations (value mismatches): 0

## Verdict
**VALIDATE — 8x theoretical savings achieved (1/8 positions searched)**

## Notes
- Probe is measurement-only: no eval terms added/changed.
- Deterministic: same positions, same budget, same seed.
- Engine legality only: uses `Position::legal_moves` and `search::analyze`.
- Symmetry group: D4 (8 elements) for square board.
- Full approach searches all 8 symmetric variants; orbit approach searches 1 representative.
- Expected savings: 87.5% (7/8 positions skipped) when all 8 symmetries produce distinct positions.
