# League scoreboard (append-only; date, variant, margins, verdict)

## Baseline — 2026-09-27 (current `search.rs`: v3 + v4 terms, D10-F7 opener)

`probe_v3all` (v1/v2 round robin, 5 openings × 2 colors = 10 games each):
- v3 vs v1: **4/10**. As Blue 2-3 (W: None, 4864; L: 5589, 11723, 9199).
  As Red 2-3 (W: None, 11723; L: 4864, 5589, 9199).
  - v3 vs v1 open=None a=blue 1322-786 A WINS
  - v3 vs v1 open=None a=red 1109-865 A WINS
  - v3 vs v1 open=Some(4864) a=blue 1243-1237 A WINS
  - v3 vs v1 open=Some(4864) a=red 1131-1218 b wins
  - v3 vs v1 open=Some(5589) a=blue 1256-1745 b wins
  - v3 vs v1 open=Some(5589) a=red 1218-1219 b wins
  - v3 vs v1 open=Some(11723) a=blue 1637-1806 b wins
  - v3 vs v1 open=Some(11723) a=red 1343-1259 A WINS
  - v3 vs v1 open=Some(9199) a=blue 1415-2120 b wins
  - v3 vs v1 open=Some(9199) a=red 1541-1578 b wins
- v3 vs v2: **7/10** (matches lineage claim).
`probe_league` (scoutbase, 8 games): AVG margin **-20.0%** (ret perspective).
Genuine games (skip=0) both wins: blue +29.4%, red +38.1%.
Collapse line persists: skip=10/20 red -111.2%, skip=30 red -111.4%
(single 30+ pop ~t=60 + erosion — needs depth or vulnerability pricing).
`gauge` (greedy, 6 games): **6/6** (deterministic repeat of 2 lines).

## Experiment V5-1 — patience-off ablation (Blue-chair first-close timing) — 2026-09-27
Hypothesis: PATIENCE_PENALTY (no sub-2.0 closes before action 12) misfires on
forced openings as Blue, delaying closes v1 snatches.
Change: `PATIENCE_PENALTY` 2.0 -> 0.0 in `retaliator/src/search.rs`.
Result: byte-identical lines in every gate — v3all v1 4/10 / v2 7/10,
league -20.0%, gauge 6/6. Term never flips a pick in any measured line.
Verdict: **REJECTED (no effect)**. Reverted to 2.0 with provenance comment.

## Experiment V5-2 — Lane A deep search: negamax alpha-beta, depth 3 (`search_deep.rs`) — 2026-09-28
Hypothesis: fixed 3-ply NAIVE deepening scored -58% league; real alpha-beta with
move ordering (turn-aware negation, ordered children, beyond-horizon extension
along the line, no doom discount) fixes it. Evaluation, D10-F7 opener and root
priority verbatim from `search.rs`, so the h2h isolates the search itself.
Change: `pub mod search_deep;` (lib.rs, one-line addition) + `search_deep.rs`
(lane-a-search state capture; one compile fix: an unbound `outcome` at line 543;
one exact optimization: `value`'s draw check short-circuits at the first legal
move instead of generating them all — 596 positions checked, 0 mismatches vs
engine movegen, ~2-3x faster: t=12 67s vs 214s, t=24 571s vs 991s wall).
Result: **NO GATE COMPLETED** — h2h W/L n/a, league margin n/a, gauge n/a.
h2h game 1 (deep=blue, open=None) left running at action 60-80/120, ~2.1 hrs CPU,
game ~50% done; league (skip 0,30) killed at ~40%; gauge killed early.
Cost (bench): ~10k-16k nodes/move, ~10-30s CPU/move mid-game rising to ~165-450s
late; a full game ~3-6 hrs CPU = 1.5-22x OVER the site's ~20s/move budget. The
brief's "1000x time headroom" is falsified for depth 3.
Verdict: **REJECTED (over budget; W/L unmeasurable on this box)**. The exact
draw-check optimization is kept on lane-a-search for future search work; depth
needs a time-budget/iterative-deepening layer (outside Lane A scope) or a much
faster box before these gates can run.

## Experiment P-dose-1 (lane-v8-pop) — POP-PRICE `DOOM_TAIL_W` 0->0.5 — 2026-09-29 — NEUTRAL (stack-or-kill pending)
Hypothesis: pricing the beyond-horizon tail (`events_left - hz_doom`) at parity with
`HORIZON_WEIGHT` lets doomed lines pay for their own horizon instead of collapsing.
Single-constant dose; no other term touched.
Change: `DOOM_TAIL_W` 0.0 -> 0.5 in `retaliator/src/search.rs` (`af35f50`,
`origin/lane-v8-pop`), 12 insertions / 4 deletions, engine-only, no harness change.
Gates (run 2026-09-29):
- gauge: **6/6 PASS**.
- league_mesh: **AVG +29.3%** (baseline -20.0%), worst row **-42.7%** — PASS per bar
  (requires >-9.9% and no row < -300%).
- h2h vs `v7base`: **Blue 0/5 Red 5/5 = 0/10** — identical to `v7base` self-h2h
  (Blue 0/5 Red 5/5), i.e. dose == control per chair. NEUTRAL, not a winner.
PRELIMINARY: figures transcribed from `research/v8-roadmap.md` (POP-PRICE entry,
lines 36-43). No `poprice-ledger.md`, league table, or h2h log is committed on
`lane-v8-pop` (only the `research/scoreboard.jsonl` placeholder, verdict
"pending", `h2h 0/10`, `league_avg N/A`). Numbers are unbacked by committed
artifacts on the branch and need re-verification before any ship call.
Lineage caveat: `af35f50` is **not reachable** from `origin/lane-v8-pop` — the
remote head was force-overwritten to `0c7352c`, a docs-only lineage off `eb100e9`
that also lacks master's P-dose-2 KILL row. The engine dose lives on local
`lane-v8-pop` = `af35f50` only. This row was appended fast-forward; no
force-push, no merge to master, no engine changes.
Verdict: **NEUTRAL** — no per-chair movement over control. Neutral single-variable
fixes are stackable, so the founder decision is stack-or-kill; gate D2
(`DOOM_PAIR_W`) only if stacking, else close the lane.