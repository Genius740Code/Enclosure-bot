# V5 continuation prompt (GLM 5.3 Flash subagents)

Copy everything below the line into a fresh session rooted at
/home/genius74o/game (repo: Enclosure-bot, master pushed).

---

You are continuing the Riposte project: a Rust WASM bot for the Meridian
territory game at constellation.blueshrimp.uk (19x19 edge-drawing, 120
actions, both sides bank enclosed area every turn, one cut per move max,
last-turn edges shielded, 20s/move, hard kill ~23s browser / 25s server).

## Objective (V5)
Beat v1 head-to-head (currently 3-10, 0-5 as Blue), then Great Barrier
(1907). Site Elo is the only judge. Local leagues gate every upload.

## Codebase map
- `retaliator/src/search.rs` — the bot (2-ply WIDTH-8, do NOT rewrite blindly)
- `retaliator/src/lib.rs` — site ABI + replay (collects cut-avoid points)
- `retaliator/examples/` — probes: `probe_v3all` (v1/v2 round robin),
  `probe_league` (scoutbase), `gauge` (greedy), `autopsy` (site-game replay),
  `probe_collapse`/`probe_traj` (trajectories), `probe_breaks*` (targeting),
  `probe_blueopen*` (openers); `support/` has v1base/scoutbase baselines
- `research/` — findings + plans (read `plan-v3.md`, `plan-v4.md`,
  `rival-analysis.md` FIRST); `vendor/meridian-bot-kit` is the site engine

## Lineage verdicts (do not relitigate without new numbers)
- v1 (asymmetric break-first): the baseline that beats everything newer 7-3.
- v2 (symmetric horizon, avoid-routing, patience): loses v1 7-3.
- v3 (+capture-exposure, Blue opener D10-F7): beats v2 7/10,
  loses v1 7-3. Opener is 12/12 as Blue (+20-41%).
- v4 (+fresh avoidance, do-nothing filter, density, two-front, doom
  discount, selection-level avoid): genuine games 2-0, greedy 6/6.
- REJECTED with numbers: GB 26-move book (-48pp), center-pull bias (-16pp),
  fixed 3-ply minimax (-58%), room weight 1.5 (noise). This eval is not
  minimax-stable; depth must come from real alpha-beta, not naive deepening.

## Iron rules
1. All eval terms in full-horizon points (area x min(events,12)) or they
   silently do nothing (proven twice). No invented units.
2. Deterministic: tiebreak by move index. Legality via engine only.
3. Every claim needs n=8+ games, both colors, vs v1 + scoutbase + greedy.
4. One file per track, never two lanes in one file. Commit + push per step.

## Open problems, ranked
1. **Blue chair 0-5 vs v1.** First-close timing/placement as Blue.
2. **Red collapse line** (`probe_collapse`): single 30+ pop ~t=60 + erosion.
   Needs depth or vulnerability pricing, not weights.
3. **Lane A (search):** negamax alpha-beta + iterative deepening + TT
   (Position hashes free) + time management from `limits.moveTimeMs`
   (soft 40%/hard 80%, easy-move fast path, panic on fail-low). Spec in
   `plan-v3.md` §2. This is the highest-EV structural work.
4. **Lane B (eval):** legal-cuts-only threat/vulnerability terms, mobility,
   potential-area machinery for empty-space building, cut+make gap math.
5. **Lane C (autopsy):** site losses (v1 0-4 + GB games in eval records via
   GET /api/evals) into blunder catalogs with action numbers.

## Subagent protocol (GLM 5.3 Flash, max 3, parallel)
Launch at most 3 background subagents with model `nvidia/z-ai/glm-5.3-flash`,
one lane each (A=search_deep.rs, B=eval_phases.rs, C=analysis-only). Shared
context: this prompt + `research/SUBAGENT-BRIEF.md`. Each lane appends rows
to `research/league-scoreboard.md` (date, variant, margins, verdict) and a
`research/<lane>-*.md` note. You merge only gated winners, rebuild the wasm
(`cargo build --release --target wasm32-wasip1`, verify exports +
8MB limit + live execution), copy to Downloads, push.

## First tasks this session
1. Check `research/league-scoreboard.md` and lane notes for prior output.
2. Reproduce baselines: `probe_v3all`, `probe_league` (8 games), `gauge`.
3. Pick the top open problem and run exactly one experiment end-to-end
   (idea -> implement -> measure -> merge-or-reject with numbers).
