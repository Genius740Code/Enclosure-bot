# v9 ideas (external session, 2026-09-30)

Provenance. Read-only session: nothing in the repo was edited, committed or
pushed. **[R]** = own replay of `research/games/<id>.json` (ref-autopsy)
through `engine/engine.js`; all 12 final scores reconcile. `engine.js` has no
node-capture rule — captures must be re-checked on Rust (flagged). Est Elo is
judgment anchored on chair gap ~200 Elo; none measured until gated.
`research/archetype-gb.md` not on the three ref branches, so
`gb-habit-replication.md` + `gb-tell-*.md` used instead. 12 game JSONs (no
a8e03a5f, no win games).

## Ranked table

| # | idea | est Elo | mechanism | dose | kill-0 (bar) | cost |
|---|------|---------|-----------|------|--------------|------|
| 1 | **K-CLASS reply hedge** (opp model) | +40 (0..+80) | Greedy reply matches VladNet 7.5% / Angel 15.8% / Stompy 22.5% / GB2 32.5% / GB 37.5% (M7 §1). Our ≥3 closes popped within one opp turn 39/60 vs VladNet while PV assumed greedy area-grab | `REPLY_CLASSES=3`: root ply-2 branches top-priority / max-our-loss / max-their-gain replies; line value = min of selection_adjusted | Actual reply in 3 classes? kill if class-union <40% pooled or VladNet <20% | ~+0 (replies already ranked); 24 lines x ~1.16ms stays in budget |
| 2 | **PROT-BLOCK: Blue seizes J11** (Blue chair) | +30 (mirror-heavy) | Red mesh entry 4 is M8-J11 @action 7; Blue owns J11 @action 5 (D10-G10, G10-J11); rule 6 makes Red's entry illegal: 9/9 forced openers [R, JS]. Red loses 5 of 8 mesh entries | `MESH_B_SEIZE` table (4 entries) + trigger "Red first turn = P10-M8, M8-J10"; one variable | Rust: block holds 9/9; Red continuation area @16 ≤10.0 in ≥8/9; our max_pop @9 ≤4.5; else kill | 0 ms |
| 3 | **CUT-YIELD: price zero-yield cuts** | +25 | Cut-actions destroyed 0 area: 44/60 VladNet, 30/49 Stompy, 17/27 GB2, 22/44 GB vs 31/155 v6; mean 2.39/1.70/3.27/3.47 vs 7.35 [R]. `ranked` exempts every cut from CONTACT/FRESH/REBUILD/IDLE | `ZERO_CUT_PENALTY=1.0` x hz on first actions with broken + 0 destroyed + 0 own gain | Pick-flip census on 12 games (Rust): ≥10% of 143 zero-yield top-bot cuts flip; 0 flips = kill | +0 (areas already computed) |
| 4 | **MIRROR-ORACLE: Blue table vs Red fixed line** (Blue; run only if #2 dies) | +20 | Red plays identical first 16 own moves in 5/5 mirrors; Blue @30 = 24–29 vs Red 37.3; score @40 = −84..−160 [R]. Red never reacts → single-agent problem solvable offline | `MESH_B_ORACLE` table (8-10 entries, actions 4..25) + #2 trigger; conflicts with #2, judge separately | Offline beam width 64 vs recorded Red line, 9 openers: Blue @30 ≥35.0 in ≥8/9; else kill | 0 ms runtime |
| 5 | **SCRIPT-LOCK: recorded-line reply oracle** (opp model, mirror-only today) | +10 | LOO: 86/89 correct where fires on v6, 0/4 on top bots [R]. Lines predict opponent, never play moves (≠ GB-book copy) | `OPP_LINES` table + MIN_SUPPORT=2; hit goes first in ranked(first=false) | Top bots 2.8% <25%: kill for top bots unless corpus ≥8 games/bot lifts coverage ≥25% @ ≥85% | O(1) lookup |

## Per-idea detail (condensed; full text in session paste)

1. **K-CLASS:** dose `REPLY_CLASSES=3` root reply pass only, argmaxes over ranked reply vector, later plies greedy, deterministic, full-horizon via selection_adjusted. Kill-0: extend replay_match.rs, 20 games/1320 turns; kill if pooled <40% or VladNet <20%. Gate: REPLY_CLASSES=1 byte-identical 20/20; league ≥+22.8%/-52.3%; site: our ≥3 closes popped ≤50% (from 65%) over ≥8 games/colour.
2. **PROT-BLOCK:** 4-entry Blue table [D10-G10, G10-J11, D10-G13, G13-J11] (J11 degree 2 = uncapturable) behind trigger; handoff to search. Kill-0: Rust 9/9 block + Red @16 ≤10.0 + our max_pop @9 ≤4.5. Gate: Blue chair vs v7base-Red ≥4/9 (control 0/5); Red stays 5/5; trigger never fires vs GB/Vlad/Stompy.
3. **CUT-YIELD:** `ZERO_CUT_PENALTY=1.0` x hz on broken + 0-destroyed + 0-gain first actions. Kill-0: Rust replay, ≥10% of 113 zero-yield cuts flip; kill if 0 or if >25% of flips let opp next-turn close ≥10. Gate: zero-yield share 63%→≤40%; gauge 6/6; site mean destroyed/cut up.
4. **MIRROR-ORACLE:** offline-solved table (NOT hand-made — hand table failed in JS sim); conflicts #2, never stack blind. Kill-0: beam-64 vs recorded Red, Blue @30 ≥35 in ≥8/9.
5. **SCRIPT-LOCK:** mirror feature meanwhile; needs Phase-4 corpus for top bots.

## Measured dead ends (do not build; all [R])

- Pre-emptive Blue cuts in mirror window: best legal destroy 0.0 at 0/55 snapshots.
- LEAD-RISK (raise DOOM_W when ahead): 33% vs 41% — no excess exposure; kill.
- RATE-GATED DOOM by opp cut rate: pre-fails ≥6/8 vs ≤1/5 bar; kill.
- Forced junk action 1 as Blue problem: full-mesh Blue still reaches 16.3–19.6 @17 — J10 loss, not junk move, costs Blue.

## Founder notes (site play + strategy, 2026-09-30)

- v8 plays better than v7 on site (confirming); Blue still huge weakness — needs better strat.
- Reinforce-lines: build unbreakable shapes (2+ shared-node walls) so they can't attack, then expand far / bank behind (GB-like, more AngelWASM-like). DENSE bonus exists (DENSE_BONUS 1.0) — extend toward enforced unbreakable-first building.
- AngelWASM circles the bot (denies expansion, banks the rest): need circle-breaker/counter-expansion answer.
- VladNet is the priority target (surgical cuts; K-CLASS #1 is the aimed fix).
- Ranking/search: rank better moves, search more moves, treat pairs together (P1) with unbreakable-awareness.
- Site games still playing — check back later for validation results.
