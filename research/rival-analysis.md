# Rival analysis for Riposte — 2026-09-27

Source: live site data (API + engine replay with the current bot-kit rules).
Ratings/records as of 2026-09-27. All cited games are FINISHED, 120 actions each.

## 1. Bot ladder (GET /api/bots)

| Bot | Rating | Record (W-L) | Kind | Hardware | Description (verbatim gist) |
|---|---|---|---|---|---|
| Great Barrier (`0c659d88`) | 1907 | 169 games | wasm | — | Fixed opening book (first 26 moves of a won game, mirrored by color) + enhanced Scout search (2-ply negamax, light alpha-beta): larger node budget, greater first-move breadth, explicit bonus for cutting exposed enemy edges |
| AngelBot WASM (`622b7d84`) | 1613 | 52 games | wasm | — | WASM version of AngelBot |
| xmybot (`a1fd81c6`) | 1591 | 16 games | wasm | — | "run, wall, wall, capture, bother", LLM-implemented |
| VladNet (`45cf9f59`) | 1584 | 46-34 (80) | external | 7700x | 2.9M-param transformer (5 blocks, width 256), 354k self-play games |
| AngelBot (`95b5bb5a`) | 1568 | 20-13 (33) | external | i9-13900hx, rtx 4070 mobile | Alpha-beta search engine |
| Stompy (`004e4f67`) | 1527 | 25-21 (46) | wasm | — | "Stronger than Scout" (Vlad's, site bot) |
| john.fun (`35d158bc`) | 1493 | 22 games | wasm | — | Complete-turn heuristic search, iterative deepening, alpha-beta/PVS, transposition table, 24 first actions × 24 continuations, 20 plans, 3 s budget |
| Riposte (`bd367686`) | 1492 | 4 games | wasm | — | (ours) |
| spacebot (`5bc57cbc`) | 1482 provisional | 0-2 | external | desktop CPU | Beam search over two-line turns + opponent cut-threat model; eval weighs territory, reachable space, sealing |
| small baby search bot v4 (`e733cdfe`) | 1456 | 34 games | wasm | — | MCTS 16/8/6 + heuristic `(score_me−score_opp) + min(turns_left,6)·(area_me−area_opp)` |
| Scout (`scout`) | 1385 | 18-48 (66) | wasm (site) | — | "Closes loops and cuts yours" |
| FoibBotV1 (`bd58f27b`) | 1433 provisional | 16 games | external | Ryzen 9 3900X | Pruned 2-ply: rank turns by area swing, pick best after opponent's best reply |

Head-to-head (finished bot-match games): Great Barrier beats VladNet 8/8 tracked
(~4300–4400 vs ~2500–2900 both colors), beats AngelBot WASM 3/3
(`3d3f8cb8` 4309 vs 3343; `65095072` 4560 vs 3542; `feec4693` 4344 vs 3457),
beats Riposte 4/4, beats Stompy (`46de1e77` 4464 vs 2740).
VladNet beats Riposte 3/3 (`f61a06ec` 2512 vs 1242; `6b66f7db` 1701 vs 1215).

## 2. Move decoding used

Move ID = `direction*361 + source`, source = `(y+9)*19 + (x+9)`,
direction index 0..47 ordered by dy then dx skipping (0,0).
Point: col `A`–`S` = x+9, row = y+10 (D10 = (−6,0)). Verified: 16058 = D10-D13.
Turn shape: action 0 solo (blue), then alternating 2-action turns
(red actions 1–2, blue 3–4, …). All games below replayed legally in
`meridian-engine` (kit revision current); per-player stats use the engine's `to_move`.

## 3. Per-game replay stats (MoveKind::Connect = loop close; broken = edge cut)

| Game (blue vs red) | Final area B / R | Final score B / R | B: 1st close / closes / cuts | R: 1st close / closes / cuts |
|---|---|---|---|---|
| `51e28dbb` GB vs VladNet, GB wins | 108 / 50.8 | 4296 / 2926 | B: act 15 / 10 / 26 | R: act 2 / 32 / 26 |
| `f87e6390` GB vs VladNet, GB wins | 119.2 / 55.3 | 4402 / 2785 | B: act 15 / 13 / 23 | R: act 2 / 30 / 22 |
| `cbc2c196` VladNet vs GB, GB wins | 44.3 / 109.8 | 2676 / 4352 | B: act 4 / 31 / 24 | R: act 13 / 10 / 25 |
| `3d3f8cb8` GB vs AngelBot WASM, GB wins | 108 / 36.3 | 4309 / 3343 | B: act 15 / 11 / 24 | R: act 6 / 14 / 26 |
| `65095072` AngelBot WASM vs GB, GB wins | 87.7 / 120.6 | 3542 / 4560 | B: act 12 / 15 / 25 | R: act 13 / 13 / 17 |
| `46de1e77` Stompy vs GB, GB wins | 61.0 / 116.2 | 2740 / 4464 | B: act 4 / 33 / 25 | R: act 13 / 13 / 25 |
| `f61a06ec` Riposte vs VladNet, VladNet wins | 7.0 / 51.8 | 1242 / 2512 | B: act 4 / 28 / 26 | R: act 2 / 14 / **41** |

Captures (node steals) are rare everywhere (0–4/game). Avg extend length
≈ 2.8 for every bot (everyone plays mostly length-3 edges). Cuts are
background noise at ~22–26 per side per game — EXCEPT VladNet cut Riposte
**41 times** in `f61a06ec` (see §5).

## 4. Per-bot style

### Great Barrier (1907) — big-loop banker + fixed book
- Opening (both colors, byte-identical across games): as Blue
  (`51e28dbb`, `f87e6390`, blue acts 2–13):
  `D10-E12 D10-F13 E12-G15 F13-H16 G15-I18 H16-J19 I18-J19 D10-F7 D10-E8 E8-G5 F7-H4 G5-I2`
  — two long diagonal walls NE (to J19) and SE; as Red (`cbc2c196`, `65095072`
  red acts 1–13, identical): `P10-O12 P10-N13 O12-M15 N13-L16 M15-K18 L16-J19
  K18-J19 P10-N7 P10-O8 O8-M5 N7-L4 M5-K2 L4-J1` — the exact mirror.
  (Only the very first action varies: `D10-F11` vs `D10-C7` vs `A10-C11` as blue.)
  Matches its description: 26-move book, mirrored by color.
- Loop timing: does NOT rush — first close at action 13–15, after ~12 wall
  actions. Then closes steadily (~12 actions apart: `[15,28,40,52,…]`).
- Only 10–13 closes/game but 108–121 area → **≈10 area per loop** (vs
  VladNet ≈1.6, Riposte ≈0.25). Builds few huge loops, banks them, never
  rebuilds small ones in place.
- Aggression: ordinary cut volume (23–26); the edge is *which* cuts it picks
  (its stated cut bonus), not how many.

### VladNet (1584) — small-loop popper with a memorized red opening
- As Red its first 13 actions are IDENTICAL in `51e28dbb` and `f61a06ec`
  despite different blue play (`P10-S13 S13-S10 S13-P11 P10-S7 S10-S7 S7-P9
  P10-M13 M13-P11 P10-M7 P9-M7 M13-O10 O10-M7 M7-P4`): a memorized/learned
  opening line, hugging its own edge (S- and M-files, short hops).
- Closes its first loop at action 2 and keeps popping: 30–32 closes as red,
  31 as blue (`cbc2c196`), but only 44–55 area. Many tiny banked loops.
- Handling cuts: absorbs them by volume — loses ~25 edges cut/game like
  everyone, but with 30+ small loops the damage per pop is small. Against
  Riposte it shifted to 41 cuts: it actively demolishes a rebuilder.

### AngelBot WASM (1613) — middleweight search, bigger loops than the poppers
- As Blue (`65095072`): `A10-C11 A10-B13 B13-D16 D10-G13 G13-E10 G13-E16 …`,
  first close action 12, 15 closes → 87.7 area (≈5.8/loop). As Red
  (`3d3f8cb8`): `S10-S13 S13-P16 P10-M13 M13-P16 …`, first close action 6,
  14 closes → 36.3 area. Respectable but GB doubles it both colors.

### Stompy (1527) / Scout (1385) family — rush closers
- Stompy as Blue (`46de1e77`): first close action 4, **33 closes** → 61 area
  (≈1.8/loop). Same disease as VladNet, milder than Riposte. Scout's code
  (below) explains it: the eval pays `area × min(events_left, 12)`, so closing
  anything scores immediately and search never waits for big loops.

### Riposte (ours, 1492) — rebuilds in place, gets farmed
- As Blue (`f61a06ec`): first close action 4, 28 closes → **7.0 area**
  (≈0.25/loop). 28 loop-closes banked essentially nothing: popped loops are
  rebuilt where they died and popped again. VladNet's 41 cuts in that game
  (vs the 22–26 background rate) is the fingerprint of farming a rebuilder.

## 5. How the best handle being cut (the anti-rebuild lesson)
- Baseline cut rate is ~25 Steals per side per game for ALL bots — being cut
  is normal play, not an emergency. Nobody's cut count is low, including GB's.
- What differs is exposure per pop: GB risks ≈10-area loops but only holds
  ~10 of them and closes steadily; poppers hold 30+ ≈1–2-area loops so each
  cut costs little. Riposte holds ≈0.25-area loops and still loses — because
  it spends ~28 actions/60 closing loops that never survive to a scoring
  event, while the opponent converts those actions into walls + cuts.
- VladNet's 41 cuts vs Riposte prove the exploit is real and targeted: when
  the opponent rebuilds in place, the best reply is to keep cutting the same
  ground. Routing away (new ground the cutter hasn't scouted) is the counter.

## 6. Scout's exact eval + search (`scout/src/search.rs`, bot kit)
1. `evaluate = (scoreB−scoreR) + (areaB−areaR)·min(events_left,12) + 0.4·(roomB−roomR)·min(events_left,1)`, room = convex-hull area of own nodes.
2. 2-ply over ACTIONS (not turns): try first actions, then best reply (opponent's if turn ended, else own 2nd action); candidate value = value after reply.
3. Budget 4096 positions; first actions = longest-edges-first, capped at budget/2, dedup'd by `repeats_a_connection` (skip A→B if B→A tried).
4. First-action priority = mover-signed value + `loop_bonus` (only if a 2nd action remains): room gained capped by the largest triangle the move could close × events left — keeps triangle-closable moves ahead of dead long edges.
5. Keep top 8 by priority, split remaining budget across their replies, sort candidates by mover-signed value. Drawn positions valued 0.

## 7. Steal-able ideas for Riposte (ranked by expected value)
1. **Anti-rebuild routing (highest value).** After one of our loops is cut,
   forbid/penalize re-closing in the same bounding box (e.g. within Chebyshev
   distance 3 of the cut point) for N turns; force the 2-ply to extend walls
   elsewhere instead. Directly fixes the 41-cuts-farmed failure in `f61a06ec`
   and the 0.25-area-per-close stat. Cheap: a move filter + eval penalty.
2. **Big-loop bias: delay first close, build walls first.** GB spends ~12
   actions walling (length-3 diagonals toward the far corner/edges) before its
   first close at action ~13–15 and gets ≈10 area/close. Add Scout-style
   `loop_bonus` triangle planning but ALSO a small penalty for closing before
   (say) action 10 or while own convex-hull room is growing fast — stops the
   action-4 tiny-triangle habit shared with Stompy.
3. **Cut-threat term in eval (spacebot's idea, GB's cut bonus).** At each leaf,
   count own exposed edges (cuttable next action) minus opponent's, weighted
   by the area each shields; prefer moves that cut high-value enemy edges
   (GB's stated bonus) and avoid leaving own big-loop edges exposed. Our cuts
   currently "hit stray edges while rivals pop banked loops" — rank candidate
   cuts by (enemy area unbanked if cut) instead of treating all cuts equal.
4. **Fixed 26-action opening book (GB's free rating).** Hard-code the mirrored
   diagonal-wall development (Blue: `D10-E12 F13…J19` + `D10-F7…I2`; Red:
   mirror) for the first ~13 own-actions, then hand over to search. It is
   public, proven over 169 games at 1907, and costs zero search budget.
5. **Eval: banked-area × time + room, with pruning (Scout + baby-bot).**
   Replace/extend current eval with `(scoreΔ) + (areaΔ)·min(events_left,12) +
   0.4·(roomΔ)` (Scout constants), longest-first ordering,
   `repeats_a_connection` dedup, width-8 2-ply — then spend saved nodes on
   reply breadth for cut-heavy positions (GB: "greater breadth at first move").
   Baby-bot's variant (`min(turns_left,6)·areaΔ`) suggests capping the horizon
   tighter late-game; test 6 vs 12.
