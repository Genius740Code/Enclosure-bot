# BEAM-SEED setup-aware re-run (external session, 2026-09-30) — GO FIRMS UP

Validated: 1440/1440 plies legal, exact-rational final scores 12/12
(BigInt shoelace; bit-exact). Authoritative codec.

## Result

| | baseline | setup-aware | forward-only t+1..t+4 |
|---|---|---|---|
| outside top-8 | 633/1440 = 44.0% | 845/1440 = 58.7% | 881/1440 = 61.2% |

Baseline reproduced EXACTLY (per-game range, blue/red split, 4.4% zero-best —
validates filed `83ea216`; lenient 29.2% vs filed 29.4%, 2-ply tie detail).
Setup-attributed share = 73/633 = 11.5% (forward-only 6.8%). Setup-credit
rescues 73 misses but evicts 285 others — diffuse credit inflates non-played
candidates. Not a window artifact.

## Verdict: GO stands, caveat FALSIFIED on both halves

Setup explains 11.5%, not "much"; adding setup-credit moves the rate the wrong
way. Residual 88.5% of misses are not close/break/setup-value moves at all —
the better case for seeding. D1 sizing: do NOT spend effort on this priority
term as specified (poor ranking signal). Limits unchanged: played-move
containment, not screen-argmax; bot-vs-bot moves, not outcome argmax.
