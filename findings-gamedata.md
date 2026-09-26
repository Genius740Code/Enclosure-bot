# Track D — Human-Archive Mining (game data)

## TOP-LINE SUMMARY
- Sample: archive pages 1–6 (recent first), 114 games fetched, 104 real (≥10 non-pass moves, known winner), 45/45 then 114/114 replayed bit-exact through `engine.js` (replay scores == server scores to float noise). 0 fetch errors, polite pace (~1 req/2.1s).
- **Earlier first-close does NOT predict winning (resign n=67): first-closer won 37/67 (55%), avg first-close line W=8.9 vs L=9.3.** Everyone closes by ~line 10; the game is decided after.
- **Winners do NOT break more: 5.3 vs 4.9 breaks dealt, damage 72.8 vs 75.9 (tied).** 11 resign-winners + 2/7 score-games won while absorbing ≥2x the damage they dealt (e.g. `xnw0kh` took 489 dealt 102; `8f8cxp` took 548 dealt 30, won 1191–246). Breaking is symmetric human activity, not a win lever.
- **Winners out-BUILD: maxTerr 16.8 vs 6.5, territories 3.8 vs 2.2, final area 27.6 vs 11.7.** Score lead compounds: leader at line 20/40/60 won 58%/61%/71%.
- **Score is cumulative — banked points survive zeroing.** `spn59l`: Arcturus won 815–531 with 0 final area / 0 territories. `rnpvit`/`otio7i`: losers dealt 998/815 area-damage and still lost by ~600–900. Late breaking cannot erase a bank.
- Signal hygiene: 0 score-decided games in pages 1–3 (all resign/abandon); 7 full-length score games found in pages 4–6 (all 120 lines, 0 passes — highest-signal rows below). 17/67 resigners were AHEAD on score at resign time (tilt/time — resignation ≠ deserved), and 9 archive games were all-pass idle lobbies (excluded).
- Color: blue won 39/67 resign games (58%) but 53/104 overall (51%) — no structural claim; contradicts nothing, confirms nothing (Track B's Red edge came from symmetric bot-vs-greedy, not humans).

## Method
- `mined-fetch.js`: login POST `/sst1/api/access`, GET `/sst1/api/games?search=&page=N`, full games via `/sst1/api/game?id=`. Read-only (no WS, no POST except /access).
- `mined-analyze.js`: replay every `moveHistory` entry via `applyMove`/`applyTimeout` (points 0..18, `{from,to}` or `{pass:true}`). Break = opponent AREA drop on your move (scores are cumulative so score-drops never happen; first version using score-drops read 0.0 breaks/game — fixed). First-close = first line with `areas[color] > 1`. Cache: `mined-data/` (lists + 114 games + `analysis.json`).

## Score-decided games (full 120-line, highest signal)
| game | winner (color) | score W–L | firstClose W–L | breaks W–L | dmg W–L | maxTerr W–L | note |
|---|---|---|---|---|---|---|---|
| ftvu1u | myriapod (R) | 1935–1013 | 7–12 | 15–20 | 220–511 | 27–13 | closed later, won via big final area (55) + 3 terrs |
| fvojn8 | Lurex (R) | 1096–945 | 3–8 | 24–19 | 617–310 | 5–10.5 | THE breaker-win: smaller area, won by suppressing |
| kc2zfj | Yo_Fish (R) | 422–418 | 7–4 | 22–15 | 168–230 | 0.5–4.5 | razor-thin; tiny areas both sides |
| lch4ij | Electra (R) | 2196–1286 | 11–4 | 24–19 | 137–695 | 18–13.5 | loser dealt 695 dmg, lost by 900 — breaking ≠ winning |
| otio7i | Diamond (R) | 2238–1329 | 7–12 | 15–18 | 491–815 | 28–0 | loser dealt 815, zeroed winner (0 terrs), still lost 900 |
| rnpvit | thg (R) | 1356–747 | 3–5 | 20–26 | 231–998 | 15–6 | loser dealt 998 (!), lost by 600 |
| spn59l | Arcturus (B) | 815–531 | 9–3 | 16–19 | 185–485 | 0–4.5 | winner banked early, ended 0 area/0 terrs, won on bank |

## Shapes our bot never plays (human winners)
- **Giant blob**: `01smaw` (maxTerr 87, 5 terrs, 96 area), `8f8cxp` (78, won 1191–246), `s5vzi9` (60), `jmzq2u` (single 52-area blob, won while DOWN 265–428 on score — loser resigned). Bot-tourney closes greedy-small; nothing in its search builds/holds 50+.
- **Late surge**: `8f8cxp` gained 75/83 area after line 80; `at40ia` 15/27. (Rare: only 2/67.) Validates Track B sandbag-crush in general — but 8f8cxp shows a blob-finisher CAN work when the blob is invincible-walled.
- **Zero-break builder**: `l59c3y` winner dealt 0 breaks, won 270–134. Pure building beats break-spam when the spam has no building behind it.
- **Loser archetype**: break-spam without building (rnpvit/otio7i/lch4ij losers dealt 695–998 damage, all lost big). If our bot's break bonus converts building tempo into breaking tempo vs a tanker, we become this loser.

## P0s (pattern + gameID, no midgame-replay performed per mission)
- **P0-1 — Tank-and-build beats break-spam (bot-tourney vulnerability): `xnw0kh`, `uk3nkb`, `8f8cxp`, `otio7i`, `rnpvit`.** Humans absorb 3–18x the damage they deal and win by out-building (maxTerr ~2.5x). Bot-tourney inherits bot.js's break incentive; vs a human who ignores our breaks and keeps closing, we risk trading building tempo for unbankable damage. Needs a challenger test: builder-that-never-breaks vs bot-tourney (inverse of chall-aggro).
- **P0-2 — Giant-blob finisher our search never evaluates: `8f8cxp` (78-area, late surge 75/83, tanked 548).** If a live opponent grows one mega-loop, does bot-tourney's frontier-capped candidate list even generate the popping move? Flag for a targeted probe (grow blob vs bot, see if/when it breaks it).
- **P0-3 — Bank-and-deny endgame: `spn59l`.** Winner banked early, got zeroed, won 815–531 on the bank while the loser "won" the board (9 area, 2 terrs). When ahead on bank, correct play is pure denial, not rebuilding — bot-tourney's 0.15 opponent-denial term looks too weak for this endgame. Probe: ahead-on-bank snapshot, does bot rebuild (wrong) or deny (right)?
- **P0-4 (watch, not action) — fvojn8 is the ONE breaker-win template (24 breaks/617 dmg, won with SMALLER area).** Breaking wins only when it suppresses opponent income from early/mid-game, not as late damage. Any break-weight change should price early suppression, not raw damage.

## Limits / next
- Only 7 score-games; resign signal diluted by 25% ahead-resigns (clock/tilt). Next expansion: pages 7–15 hunting more `score` games; check `clocksAfter` in resign games to split flag-losses from true resigns.
- Break metric = area-drop attribution per mover; multi-edge collapses attributed to the last touch. Good enough for aggregates, not for single-move forensics.
- No WS traffic sent; no POST except /access. No live-game interference.
