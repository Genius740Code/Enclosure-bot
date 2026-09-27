# Sparring program brief (shared by all lanes, 2026-09-27)

## Goal
Make **Riposte** (`retaliator/`, Rust WASM bot for constellation.blueshrimp.uk,
Meridian 19x19 territory game) beat Great Barrier (1907) and VladNet (1584).
Current record: 0-7 on site. Local standing: beats greedy 6/6, parity with
Scout-class search (~+1% league), loses search-vs-search as Blue.

## Game facts (all verified in `meridian-engine`, `vendor/meridian-bot-kit`)
- 120 actions. Blue solo action 1, then 2-action turns, Red first.
- Both sides bank enclosed area EVERY turn end. Score diff is the objective.
- One cut per move max; last-turn enemy edges shielded; one cut zeroes a loop.
- Move ID = direction*361+source. Site gives ~20s/move, 30s/analysis.
- Our bot: 2-ply WIDTH-8, budget 4096 positions, ~20ms native. **1000x time
  headroom unused** (`limits.moveTimeMs` ignored).
- Determinism required-ish (seeded repeatability); tiebreak by move index.

## Diagnosed diseases (evidence in `research/rival-analysis.md`, autopsy probe)
1. Small-loop addiction: ~0.25 area/close vs GB's ~10. Closes tiny triangles
   from action 4; GB walls 12 actions then banks big.
2. Rebuild farming: VladNet cut us 41x/game (background ~25). We re-close
   where we were cut. Partially addressed: cut-avoidance (`avoid` points from
   replay, CUT_RADIUS 3, REBUILD_PENALTY 3.0) — UNPROVEN vs farmers.
3. Patience knob added, unproven: no sub-2.0-area closes before action 12.
4. GB's 26-move wall book TESTED and REJECTED (-48pp in league): our
   follow-up search can't convert its structures. Do not re-add blindly.

## Lane file ownership (no overlapping edits)
- Lane A (search): may add `retaliator/src/search_deep.rs` + benches. Do NOT
  touch `search.rs`, `lib.rs`.
- Lane B (eval/phases): may add `retaliator/src/eval_phases.rs` (weight
  tables, phase logic) + probe examples. Do NOT touch `search.rs`, `lib.rs`.
- Lane C (autopsy): analysis only. May add `research/c-*.md` + read-only
  probes. No src edits.
- Every lane: compare vs `search::best_move` baseline with
  `examples/probe_league.rs` (8 games, scoutbase) + `examples/gauge.rs`
  (greedy). Report avg margins, both colors. n=8 minimum for any claim.
- Site reference games: our losses `f61a06ec` (VladNet), `f4b7f187`,
  `2ae426c6` (Great Barrier) via `GET /api/games/GAME_ID`; replay with
  `examples/autopsy.rs`.

## Scoreboard
Append results to `research/league-scoreboard.md` (create it): date, variant,
league avg margin, gauge W-L, one-line verdict.
