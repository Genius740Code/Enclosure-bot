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

## 2026-09-28 — V5 (doom OFF port, DOOM_W 1.0 -> 0.0) — main session
Port of Lane B's rig result into search.rs (one constant + provenance comment).
Rig numbers: league -9.9% vs -20.0%, v1 5/10 vs 4/10, collapse -55.9% vs -111.4%.
Port gate: gauge 6/6 (deterministic repeat of 2 lines, margins +53.5%/+69.3%).
Caveats: ties (not beats) v1; as-Blue 2/5 -> 1/5; rig ran without avoid path.
Verdict: SHIP (best measured config; site Elo decides).

## 2026-09-28 — V6 (mesh8 prefix + doom-ON) — UPLOADED
Port of Lane H mesh8 (pair-turn slots, onset scan, permanent truncation,
between()-built moves) + doom discount restored to 1.0 (site lineage:
v3 1477/v4 1428 doom-ON vs v5 1349 4-36 doom-OFF).
Gates (on port): league_mesh +21.6% worst -57.6% (bar: >-9.9%, no row <-300);
gauge_mesh 6/6 (+74-75%); v1/v2/scout h2h probe-side doom-ON base
(v1 B5-0/R4-1, v2 5-0/5-0, scout B5-0/R4-1); mesh_verify + site_check green
(opening reply 16058, 12 legal site-path replies, analysis depth 2).
Site bot "Riposte v6" 853d805d-8aaa-4061-a0bf-816fedd3e4c9 (fresh provisional),
version 93cbc665b8ab5b885983bec02d2969917f421ed916d3a23a0581095616a8cedd (290011 bytes).
Rated evals queued pairs=1: v5 (6ea13d3e), v4 (a68f64de), GB (daf1be99),
GB2.0 (5e7dbc71), VladNet (ddd08831), AngelBot (0b6220f8). Site forces
action-1 per pair: prefix onset-scan delays (inj may read 7/8 there).
Verdict: PENDING site Elo.

## V7 directive — timed think (Q6-amendment, user order, NOT gated) — 2026-09-29
User directive: v7 must spend ~2s per searched move, more while volatile.
Change (`retaliator/src/search.rs`, master): `best_move_routed` now uses
`analyze_timed` — beam deepening (BEAM=8) through the same `ranked()` reply
model and selection adjustments (horizon/doom/rebuild/fresh); SOFT=2000ms
minimum, HARD=8000ms cap, VOLATILE_GAP=2.0 top-2 extension. Prefix/opener and
forced moves stay instant. Probes/analysis keep fixed node budgets.
Measurement (native, mid-game 373 legal): full 2-ply = 11ms/2436 nodes
(saturated — wider budgets change nothing); timed = 2029ms/depth-73/264613
nodes, best move legal. `site_check` OK (opening 16058, 12/12 legal, 15.5s).
Caveats: depth is machine-speed-dependent (routed-path gauge determinism no
longer holds); strength UNGATED — league/h2h numbers pending. WASM rebuild
required before upload.

## 2026-09-29 — V7 UPLOADED ("Riposte v7" 0c63610e-23a4-4a41-a701-723cdb1853f7)
Engine = v6 (mesh8 + doom-ON, same valuation) + timed think: routed moves spend
2000ms soft / 4800ms hard via beam-8 deepening (same reply model + selection).
WASM 306523 bytes, sha256 4f6904eff817d56613232ab9c8983db461c3177640b8361a630b0608d8c292c9,
current version. site_check green pre-upload (opening 16058, 12/12 legal).
Rated evals: user-run (not queued by session).

## 2026-09-29 — V7 rated evals queued (8 matchups x pairs:1 = 16 games, ~10h)
v6 mirror user-run. Session-queued, all rated: v3 f94f561b, Stompy 2adafd6a,
VladNet 9f06cd59, AngelBot-WASM 5164143f, GB b1e7d2d1, GB2.0 f33e29fc,
capybara-v5 90a4e520, xmybot 323bf662 (first attempt typo'd id, re-queued OK).
