# BEAM-SEED census (external Claude session, 2026-09-30) — SURVIVES kill-0

Method: replayed all 12 `research/games/` JSONs on `origin/lane-v8-autopsy`
through a stdlib Node reimplementation of the vendored Rust rules + repo
`engine/engine.js` territories. Validated: 12/12 final scores match, 0 illegal
plies, `broken` matches on all 1440 plies. At every ply (~499 legal moves avg)
scored priority = close-gain + break-gain, top-8 by (priority desc, move id asc).

## Result

Outside-top-8 rate: **633/1440 = 44.0%** strict (423/1440 = 29.4% lenient).
Required four: 215/480 = 44.8%. Range 37.5% (c20c750b) – 53.3% (b0ac4141).
Blue 323/720, Red 310/720. Not a tie artifact (best priority 0 in only 4.4%).
a8e03a5f MISSING from branch, not counted.

## Verdict: SURVIVES kill-0 (44% >> 10% bar) — provisional GO

Kill-0 kills the idea only if outside-rate <10% (beam already covers everything).
44% means winners routinely lie outside a close+break top-8, so seeding the beam
with outside candidates has something to find. Corroborates the POP ledger
(alternatives rank 10-106). NEXT: cost gate (median WASM move <=2.0s, max <=4.5s)
needs a Rust box — not sandboxable, runs on our lanes.

## Caveats (do not over-claim)

- Measured "played move" containment, not spec's "screen-argmax" rate; played
  moves come from bot-vs-bot games, not an outcome argmax.
- Heuristic ignores setup moves (extends preparing closes) — likely much of the
  44%. A setup-aware priority would lower the rate; re-run before sizing D1.
- JS reimplementation, not the Rust engine (no cargo in sandbox).
