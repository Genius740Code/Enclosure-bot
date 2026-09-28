# V6 OVERNIGHT PROMPT — Riposte bot for Meridian (constellation.blueshrimp.uk)

You are the main session. Goal: ship **v6**, clearly stronger than v5 on **site Elo**.
Run up to 4 concurrent subagents (background=true), respawn instantly on finish/fail.
Work 15h straight, checkpoint every 3-5h. No asking the user — only report.

## Models
- Reasoning/analysis/autopsy/opponent-modeling: `nvidia/z-ai/glm-5.3` (smart) or
  `nvidia/z-ai/glm-5.3-flash` (fast+smart). Flash rate-limits under concurrency:
  if HTTP 429, move lanes to full 5.3 and/or stagger launches.
- Implementation/search code: Nemotron (fast, huge context, strong Rust).
- Rule: commit + push PER STEP so 429s/deaths never lose work; each lane in its
  OWN worktree (`git worktree add /tmp/opencode/game-<lane> <base>`), never
  checkout branches inside the main repo or another lane's worktree.

## Repo
`https://github.com/Genius740Code/Enclosure-bot`, branch `master`.
Worktrees live outside the repo. Bot code: `retaliator/src/search.rs` (SHIPPED BOT),
`retaliator/src/lib.rs` (site ABI + replay with avoid routing — deployed path).

## Where v5 stands (tag `v5`, commit `9c1f043` + pickmix doc `2deb3c7`)
- v5 = v4 + doom discount OFF (`DOOM_W` 1.0 -> 0.0, provenance comment in search.rs).
- Rig numbers: league -9.9% vs -20.0%, v1 h2h 5/10 vs 4/10, collapse -55.9% vs -111.4%,
  gauge 6/6. Costs: as-Blue 2/5 -> 1/5; ties (not beats) v1.
- Site bot "Riposte v5" id `3dff983a-171c-45b6-b899-bad54648f13e`, version hash
  `10071d190f56e927db21aac0a97aa596f4faa1637bba8716c83dfaa161ec4b03`.
  Early site signal MIXED/NEGATIVE: 0-3 vs v4, 0-2 VladNet, 1-1 Scout, 0-2 GB —
  small sample; if it stays negative, consider reverting doom-off in v6.
- Baselines: `probe_v3all` (v1/v2 round robin, 5 openings x 2 colors),
  `probe_league` (scoutbase, 8 games), `gauge -- 6` (greedy). Run from `retaliator/`.
- Scoreboard: `research/league-scoreboard.md` (append-only, one row per variant).

## Iron rules (non-negotiable)
1. All eval terms in FULL-HORIZON points (area x min(events,12)) or they silently do nothing.
2. Deterministic: tiebreak by move index. Legality via engine only.
3. Every claim n>=8, both colors, vs v1 + scoutbase + greedy.
4. One file per track; NEVER two lanes in one file. Lane branches only; main session
   alone merges to master. NEVER touch `search.rs`/`lib.rs` from lanes.
5. Never relitigate REJECTED ideas without new numbers: GB 26-move book (-48pp),
   center-pull (-16pp), naive depth-3 (-58%), room weight 1.5 (noise), vulnerability
   term (worse everywhere), close-size weight 1.0-3.0 (all worse than off).

## Diagnosed diseases (evidence: research/c-blunder-catalog.md, c-site-losses-2.md)
- Area-lifetime asymmetry (not cut volume): we out-close losers 1736-930 but our
  area dies (mean 30.7 vs 49.8). Rivals bank; we rebuild.
- Farm cycles run 20-40 actions (CUT_MEMORY 6 outlasted); re-closes re-popped by
  identical move ids. Penalty 36 < 54-198 can't flip a farmed re-close.
- Blue chair: not timing (ruled out), not action-1 (90-game sweep: no first move
  fixes it) — problem is close-survival/farming after act 12.
- 47% of our cuts pop nothing (break bonus mispriced). Mega-blob swallow: enemy
  banks 0 for 40+ actions then closes one 65-256 loop while our eval peaks +1099.
- Harness gap: local probes bypass the deployed avoid path (see
  research/harness-gap-avoid-path.md). Plan: route gates through answer() via shared
  examples/support/answered.rs — apply ONLY when no lane is mid-measurement.

## V6 work items (highest EV first; one variable at a time, ablation each)
1. **Think longer (Lane A track, `search_deep.rs`, branch `lane-a-search`)**.
   Status: cheap leaf eval (7s->0.1s) + TT done, value-exact. Open: turn-start roots
   cost 12-40x (same-mover ply passes beta=INF) -> fix root window (parity-aware
   aspiration) -> iterative deepening with soft per-move cap (target <=2s native).
   Then XBot ordering (captures>cuts>closes). Gates only when affordable. Keep `avoid`.
2. **Connect the right line (NEW Lane X, eval track, `eval_phases.rs`)**.
   Port XBot `one_move_potential` (frontier pairs one action from closing) as
   POT_W*(pot_me-pot_opp)*hz; sweep weight. Metric: missed-area-closes/game must fall.
   Spec: research/xbot-pickmix.md.
3. **Farm-cycle routing (Lane B track, `eval_phases.rs`, branch `lane-b-eval`)**.
   Control = doom OFF. Next: ScaledExempt variant (exempt shipped-exempt ground),
   then S2 break-bonus repricing, S1 closure-progress (price enemy room by build
   progress), S4 bank-race denial. Targets: v1 >=6/10, league better than -9.9%,
   no collapse loss worse than -300.
4. **Mesh opening (Lane H track, NEW probes only, branch `lane-h-opener`)**.
   Capybara center mesh as forced prefix (Blue D10-D13 D10-G8 G8-J10 J10-G13...,
   Red mirror). Smoke: GB mega-loop held to 4.5. Need full margins vs v1/v2/scout.
   If verifies: adopt prefix in v6 (port into search.rs opener section).
5. **Opponent watch (Lane D/G track, `opp_*` probes only)**. Blob/sacrifice/long-farm
   mimics on `lane-d-opp`; capybara fingerprint in research/g-capybara.md.

## Branch map (all on origin; read-only except your own lane branch)
- `master` — only main session merges here (gated winners). Tags v5, v6...
- `lane-a-search` — search track (search_deep.rs). ACTIVE work may be mid-flight.
- `lane-b-eval` — eval track (eval_phases.rs). ACTIVE work may be mid-flight.
- `lane-c-autopsy` — finished analyses (c-blunder-catalog.md, c-site-losses-2.md). READ.
- `lane-d-opp` — finished D mimics+tournament. `lane-d2-opp` — D2 blob/sac/longfarm
  mimics (may be mid-flight; adopt dont redo).
- `lane-e-siteops` — site-ops.md + harness-gap-avoid-path.md. READ, don't redo.
- `lane-g-recon` — g-capybara.md fingerprint. READ.
- `lane-h-opener` — opener probes (probe_bluefirst.rs, probe_mesh.rs). ACTIVE maybe.
- `work-*` local branches in others' checkouts are SCRATCH — ignore them.
Rules: start each lane worktree from the listed branch tip (`git log origin/<branch> -3`
first); never force-push a lane branch (append commits); never merge lane->master
except via the gated-winner checkpoint; delete no branches.

## Lane file ownership
- A: `search_deep.rs` + `probe_deep.rs`. B/X: `eval_phases.rs` + `probe_b_*`/`league_b`/`gauge_b`
  (X and B share the track: run SEQUENTIALLY, never concurrently).
- C (autopsy): `research/c-*.md` only. D/G/H: `opp_*`/`probe_mesh.rs`/`probe_match_*` + notes.
- Scoreboard appends allowed for all. Nothing else.

## Site ops (see research/site-ops.md)
- Auth: cookie `meridian_session` (user supplies). WASM limit 8MB, 3 versions/bot.
- Upload: POST /api/bots/:id/versions (Content-Type: application/wasm).
- Matches: POST /api/evals {botA,botB,pairs} (rated, server-side, ~40min/game).
- Games: GET /api/games?limit=N, GET /api/games/ID (public). Replay via examples/autopsy.rs.
- Rivals: GB `0c659d88-...`, VladNet `45cf9f59-...`, AngelBot WASM `622b7d84-...`, scout.
- Build: cargo build --release --target wasm32-wasip1; verify exports
  (meridian_abi=1, alloc, run) + live legal-move replies before upload.

## Checkpoint (every 3-5h) + shutdown
- Merge ONLY gated winners (beats v1 h2h + better league + clean gauge) into master,
  rebuild WASM, upload (new bot if v6 needs clean Elo: POST /api/bots kind=wasm),
  start rated evals (>=2 games each vs GB/VladNet/AngelBot, both colors via pairs),
  tag v6, v6-1..., push tags. Autopsy finished games into Lane C.
- Scoreboard row per upload with site results. Final: summary + open items.

## Start now
1. git pull, read scoreboard tail + this file's lane states (branches may have new
   commits from prior lanes — adopt them, don't redo).
2. Launch 4 lanes on the highest-impact open items above. Keep 4 hot for 15h.
