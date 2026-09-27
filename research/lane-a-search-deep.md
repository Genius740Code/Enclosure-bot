# Lane A — deep search (negamax alpha-beta, depth 3) — 2026-09-27

## Lineage and hypothesis

Fixed 3-ply NAIVE minimax measured **-58% league** — this eval is not
minimax-stable, so depth must come from real alpha-beta with move ordering.
`search_deep.rs` (from origin/lane-a-search state capture, ~589 lines) is that
test: negamax alpha-beta over actions with:

- turn-aware negation: a turn is 1-2 actions, so the value negates only across
  turns, never within one (within a turn the mover maximizes, window unflipped);
- children ordered by mover-signed 1-ply value + horizon extension (the core of
  the root priority function); complete root coverage (no truncation — that is
  what misled the naive minimax);
- the beyond-horizon extension accumulated along the searched line
  (signed, Blue-relative, events-free), applied at the leaf with the leaf's
  events factor — same accounting as the baseline's root-level extension;
- the doom discount DROPPED: with real depth the search sees the enemy's pop
  inside the tree;
- `one_edge_nodes` on a fixed array (no HashMap allocation — the eval now runs
  at every node).

Evaluation, opener (D10-F7) and root priority are verbatim copies of
`search.rs` (v3 + v4 terms), so any difference against `search::best_move`
isolates the search itself.

## Setup notes

- Wiring: `pub mod search_deep;` added to `retaliator/src/lib.rs` as a pure
  one-line addition (nothing else touched); sanctioned by the orchestrator.
- Compile fix: `search_deep.rs` line 543 referenced an unbound `outcome`
  (leftover from the state capture); removed.
- Probe: `retaliator/examples/probe_deep.rs` — modes `h2h` / `league` /
  `gauge` / `all`. `probe_league.rs` and `gauge.rs` hardcode
  `search::best_move` and are not Lane A's to edit, so the gate structures are
  replicated with `search_deep` substituted:
  - h2h: 4 openings (None, 4864, 5589, 9199) x 2 colors = 4 games each color
    vs the baseline. Both engines are deterministic, so openings are what make
    the games differ (same device as `probe_v3all`).
  - league: scoutbase opponent, skip 0/10/20/30 prelude (played by
    `search::best_move`, as in the baseline), both colors, AVG margin
    (ret perspective).
  - gauge: greedy max-area 1-ply, 6 games alternating colors.
- Iron rules held: eval terms in full-horizon points (area x
  min(events,12)); deterministic tiebreak by move index; legality via engine
  only.
- Baseline (from `research/league-scoreboard.md`): v3 loses to v1 4/10, beats
  v2 7/10, league -20.0%, gauge 6/6.

## Search cost (measured 2026-09-27, loaded 2-core box, probe progress lines)

- Early-game moves (b ~100-200): ~1-6s CPU/move. Mid-game (24 actions in):
  ~14s CPU/move average. Late-game moves rise steeply (48 directions x
  growing node count -> b^2 evals per move): ~30-60s CPU/move.
- Per game (~60 deep moves): ~25-40 min CPU; wall time 2-4x under the box's
  load (load avg ~11, a data collector at ~1.4 cores + other lanes' builds).
- DEPLOYMENT CAVEAT for the merge decision: depth 3 as-is costs ~25-40s
  CPU/move average — OVER the site's ~20s/move budget on late-game moves.
  Even a winning search needs a time-budget layer (iterative deepening from
  depth 2, or depth capped by branching) before deployment — a lib.rs /
  bot-integration concern, not a search_deep.rs one.
- Overnight scope: the box supports ~0.5-0.7 core aggregate for Lane A, so
  the full 22-game gate set (~11-16 hrs CPU) is not viable. Running the
  decisive subset: h2h 2 openings x 2 colors (4 games), league skips 0+30 x
  2 colors (4 games — skip 0 = genuine line, skip 30 = the collapse line the
  diagnosis says needs depth), gauge 2 games (relaunched when h2h/league
  land). n is below the brief's 8-per-claim minimum; verdicts are labeled
  accordingly.

## Results

(numbers appended as gates complete)
