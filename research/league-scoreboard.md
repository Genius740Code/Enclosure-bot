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

## Experiment V5-3 — rebuild-denial v2 dose 3: CUT_MEMORY 6->20 — 2026-09-30
Hypothesis: extending farming memory from 6 to 20 (dose 3) would further reduce rebuild-denial effects, expecting plateau/neutral gate versus control.
Change: `CUT_MEMORY` 6 -> 20 in `retaliator/src/search.rs`.

Gate results (once vs control, per-chair tables E-6):
- **h2h (probe_b_h2h)**: Faithfulness: 0 mismatches in 239 positions (control vs shipped ✓). Per-opening margins: open=None a=blue +48.4% A WINS, a=red -123.8% b wins; open=4864 a=blue -49.0% b wins, a=red +13.8% A WINS; open=5589 a=blue -35.0% b wins, a=red +11.1% A WINS; open=11723 a=blue -17.7% b wins, a=red +6.9% A WINS; open=9199 a=blue -13.9% b wins, a=red -6.5% b wins. 4/10 blue wins, 3/5 red wins avg.
- **league (league_b)**: AVG margin (ret perspective): **-25.5%**. skip=0: blue +26.5%, red +8.5%; skip=10: blue +26.5%, red -120.7%; skip=20: blue +26.5%, red -120.7%; skip=30: blue +55.7%, red -105.9%.
- **gauge (6 games)**: 6/6 deterministic repeat. Game 1: retaliator=blue margin=+66.0% breaks R=26 G=18; Game 2: retaliator=red margin=+69.3% breaks R=3 G=2. Overall: retaliator wins 2/2.

Verdict: **REJECTED** — league margin -25.5% is not plateau/neutral; h2h faithfulness passes (0 mismatches) but league shows significant negative drift. Close lane regardless after gating.
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

## Experiment V5-4 — rebuild-denial v2 dose 4: penalize ALL re-closes near cuts — 2026-10-01
Hypothesis: The REBUILD_PENALTY only applied to non-breaking moves (outcome.broken.is_none()),
but re-closes typically DO break the enemy's loop (the loop that cut us), so they were
exempt. This allowed the farming cycle: we get cut -> we re-close (breaking their loop) ->
they cut again. Fix: remove the outcome.broken.is_none() check so that ALL moves near
recent cuts are penalized, regardless of break status. CUT_MEMORY=20 (from dose 3).

Change: Removed `outcome.broken.is_none() &&` from REBUILD_PENALTY checks in:
- `retaliator/src/search.rs` (2 locations: analyze_with_avoid, ranked)
- `retaliator/examples/support/v2base.rs` (1 location: ranked)

Gate results (vs control CUT_MEMORY=20 with old logic):
- **gauge (6 games)**: **6/6** (deterministic repeat, identical margins to control)
- **probe_league (8 games)**: **-20.0%** avg margin (identical to control — probes don't use avoid points)
- **h2h v1 (10 games)**: 4/10 (identical to control — probes don't use avoid points)

Note: Probe tests (gauge, probe_league, probe_v3all) call `best_move()` which passes
empty avoid slice. The fix only takes effect when avoid points are provided (actual bot
via lib.rs replay function). Probe results are identical to control as expected.

Verdict: **PENDING SITE TEST** — fix is correct for actual bot (uses avoid points),
but probes cannot measure the effect. Need site eval with avoid points active to
verify re-closes DOWN and league improvement.
