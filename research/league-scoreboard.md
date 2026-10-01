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

## Experiment V5-2 — probe_deep depth-3 UNBOUNDED re-confirmation — 2026-09-28
Hypothesis: re-measure whether depth-3 alpha-beta (unbounded `analyze()`, XBot ordering) is league-viable at 8 games (lane protocol).
Run: `probe_deep 3 8` on lane-a-search working tree — KILLED after 27 CPU-minutes with **0 of 8 games completed** (not one game finished; ≈27s+/deep-move). Confirms Lane A's prior verdict ("depth-3 over budget 1.5–22× site move time") at unbounded width: dead on cost, never mind league.
Verdict: **REJECTED (cost — re-confirmed)**. Only the width-capped deep path (benchcap 0.00–0.16s/move, Lane A2(d)) remains viable; its league gate is the open item (parallel session probing `benchbudget 2000` in game-s workspace — do not duplicate).

## 2026-10-01 — Lane A: width-capped deep search, league gate CLOSED — 2026-10-01 (lane-a-search)

Config: `retaliator/src/search_deep.rs` restored from `0b1c0ba` (Lane A2(d)) onto
lane-a-search: negamax alpha-beta over actions (negation across turns only —
within a turn the mover maximizes), depth-1 TT entries (path-independent parts +
line extension, DefaultHasher, cleared per move), iterative deepening under
`Budget` (Ms native-CPU clock / Nodes deterministic), width caps
`Width{root,inner}` 8/6, engine-only legality (`has_legal_move` verified 596/596
vs engine movegen), eval + root priority verbatim from `search.rs` (v3+v4
terms), all terms in full-horizon points. Gate config:
`Budget::Unbounded` + `DEFAULT_WIDTH` + depth 3 (deterministic; the game-s
parallel probe died — workspace gone — so this lane ran it).
- league (scoutbase, 8 games): AVG margin **-8.2%** (baseline -20.0%, +11.8pp).
  As Blue +28.7% avg (3W/1L: +48.5, +48.5, -5.5, +23.1); as Red -45.0% avg
  (1W/3L: +15.0, -76.4, -76.4, -42.1 — skip=10/20 red are the SAME converged
  line, so the Red sample is 3 unique lines). Worst row -76.4% > -300% bar.
  League bar AVG > -9.9%: **PASS**. Collapse line improved (-111.2% -> -76.4%)
  but persists.
- gauge (greedy, 6 games): **6/6** (+69.8% blue / +73.0% red, deterministic
  repeat of 2 lines) — baseline 6/6, no regression.
- h2h (vs `search::best_move`, 4 openings x 2 colors = 8 games): **6/8**
  (blue 3/4 avg +15.0%, red 3/4 avg +14.3%). Narrowest win: 9199 red 979-975.
- Cost: depth 3 `Width 8/6` = 0.00-0.06s/move wall over 20 bench positions
  (benchcap) — 300x under the 20s site limit. Unbounded width stays dead
  (27 CPU-min, 0/8 games completed).
- Determinism: repeated runs give identical lines (league x2, gauge x3).
Verdict: **MERGE-READY (search core)** — depth pays once the search is real
alpha-beta with width caps at turn-start roots (their ~b^2 leaf-parent trees
were the cost disease; 8/6 makes depth 3 tens of ms). Red-side league weakness
(-45.0%, 1-2 unique lines) is the follow-up: collapse line needs Lane B
vulnerability pricing or deeper search. Main session merges into search.rs;
stashed WIP (TT+ID+timed variant with unified root-relative extension,
stash@{0}) kept for reference.
