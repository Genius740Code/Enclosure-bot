# Track R — Robustness Findings (bot-tourney.js vs anything)

## TOP-LINE SUMMARY
- Hardest opponent: **blob — beats bot-tourney 2/2 at 300ms (4225–981 as Red, 3021–2340 as Blue); as-Red loss CONFIRMED at 2000ms/turn (4225–1342)**. As-Blue loss flips at 2000ms but only to +5.9% (still sub-10% P0-candidate). Next: neck-hunter 2/2 (confirms Track B), aggro-as-Blue 1/2, uniform-random-as-Red 5/10 (2.2x structural Red edge swamps the skill gap). Bot's record vs these four: 16W–10L with Blue the weaker chair (6–7 vs 10–3). Junk/turtle/sprawl/sandbag all crushed; bot-as-Red is 10–0 vs noise.
- Bot record vs anything that fights or races: blob 0–2, neck 0–2, aggro 1–1, random 15–5 (5–5 as Blue, 10–0 as Red) — 16W–10L combined, with the Blue chair the weaker side (6–7 vs 10–3).
- No replay mismatches anywhere (0 illegal moves returned by any side in 34 games + self-play + 3 confirmations).

## Method
- Challenger: `chall-random.js` (rejection sampling over (own-node, dx/dy) pairs
  with seedable mulberry32 RNG — accepted samples are uniform over legal moves;
  `setSeed(n)` resets; runner sets seed per game), `chall-junk.js` (longest
  zero-area-gain edge, avoids contact; takes least-area move only if forced),
  plus all six existing Track-B archetypes.
- Runner `runR.js`: bot-tourney.js at `--budget 300` (ms/turn) vs challengers,
  both colors; legality verified by replaying every returned move through
  `engine.js applyMove` on live state (mismatches logged + counted).
- NOTE on budget: tournament play uses 8000ms/turn; 300ms is a timebox
  compromise (~44s/game). bot-tourney's strategy is identical at any budget
  (time-boxed 2-ply -> 1-ply -> first-legal); lower budget only narrows the
  2-ply shortlist. Key results re-confirmed at higher budget where flagged.
- Deterministic archetypes: 2 games (bot blue / bot red). Random: 10 seeds/color.

## Results table
| Opponent | Bot=Blue (bot–chall) | Bot=Red (bot–chall) | Beats bot? |
|---|---|---|---|
| random as Red (bot Blue, 10 seeds) | 5W–5L, bot 1129–1610 vs chall 739–1386 | — | **YES — P0** |
| random as Blue (bot Red, 10 seeds) | — | 10W–0L, bot 2130–4100 vs chall 391–998 | No |
| junk (deliberately bad) | 2W–0L, bot 2535–5575 vs chall 0–0 | No |
| blob (1-ply max-area) | 0W–2L: Blue 981–4226; Red 2340–3021 | **YES — P0-2** |
| turtle (passive) | 2W–0L, bot 1203–3985 vs chall 393–417 | No |
| neck-hunter | 0W–2L: Blue 736–802; Red 373–705 | **YES — P0-3** |
| sprawler | 2W–0L, bot 1128–2253 vs chall 703–785 | No |
| aggro | SPLIT: Blue wins 538–282; Red loses 481–633 | **YES (as Red) — P0-4** |
| sandbag | 2W–0L, bot 1203–3981 vs chall 498–537 | No |

## P0 findings
### P0-1: uniform-random Red beats bot-Blue 5/10 (bot-tourney @300ms/turn)
Losses: s1000 (−2.6%), s1001 (−0.8%), s1004 (−22.8%), s1005 (−14.3%), s1007 (−4.8%); plus near-miss win s1002 (+2.2%). Sequences: `R-chall-random-botIsblue-s*.moves.json`. WHAT beat it: nothing clever — per-turn cumulative scoring turns the game into a race to bank the first loop, and random closes a ~20-area loop by move ~36 purely by luck (s1004: red area 19.0 vs blue 10.8 at move 36, held all game), while bot's Blue opening banks only ~10–15 area in the same span and then never breaks the enemy loop (break weight 1.5 vs area×2, same flaw Track B found vs neck-hunter). Bot's blue trajectory is byte-identical across seeds through mid-game — it plays solitaire in its own corner. Red's structural ~2x edge does the rest: bot-Blue (1130–1610) vs random-Red (740–1390) overlap, so noise wins half. (Bot-Red vs random-Blue is 10–0 by 54–88% — the bot IS stronger than random on equal footing; its Blue opening is the hole.) PATCH: faster Blue loop-closing in the first 30 moves + real break valuation, same prescription as neck-hunter.
- Control DONE (bot-tourney self-play @300ms): blue 920.8 vs red 2055.3 (Red 2.2x) — the structural Red edge under competent play. Nuance this forces: bot-Blue scored 1129–1610 in ALL random games, i.e. better than its 921 vs itself, and random-Red (739–1386) far below self-play-Red (2055). So per-color, bot >> random on both colors; the 5–5 is the 2.2x color edge swamping the skill gap, not bot playing "at random level". P0 stands (a tournament bot must beat noise from the Blue side too), but the honest patch target is Blue-opening speed + Red-denial, not general incompetence.
### P0-2: blob (dumb 1-ply max-area greedy) beats bot-tourney BOTH colors
Scores: as Red 4225.5–981.1; as Blue 3021.2–2340.3. Seqs: `R-chall-blob-botIsblue.moves.json`, `R-chall-blob-botIsred.moves.json`. WHAT: a pure area race with zero fighting — blob-Red banks 18 area by move 17, 45 by move 44, 148 by move 120, compounding every turn, and bot never once breaks blob's loop (same missing-retaliation flaw as P0-1/P0-3). As Blue, blob simply out-areas bot straight up (84 vs 71 final). A ~30-line greedy with no search and no opponent model beats the 2-ply tournament bot from both chairs. CAVEAT + CONFIRMATION: Track B found blob-as-Blue LOSES to full-strength bot.js (2790–5227), and a @2000ms/turn re-run flips this chair too: bot-Red beats blob-Blue 2942.6–2778.7 — but only by **+5.9%, still under the 10% P0 bar**, so blob-as-Blue remains a P0-candidate on margin. The as-Red loss is CONFIRMED real at @2000ms/turn (near-tournament budget): bot-Blue 1342.4 vs blob-Red 4225.5 — bot improves (981→1342) but is still crushed 3.1x; blob's banked trajectory is untouched (red score byte-identical), i.e. bot never lays a finger on blob's loop at any budget.
### P0-3: neck-hunter beats bot-tourney BOTH colors (confirms Track B vs full bot.js)
Scores: as Red 801.9–736.1; as Blue 705.0–373.0. WHAT: low-area knife fight — neck grazes bot's frontier nodes and pops bot's loops as they form (bot-Red never holds more than ~8 area all game), winning the low race ~2x. No budget-dependence concern: Track B already proved neck beats un-budgeted bot.js both colors (523–236, 990–512), and the mechanism is identical (break valued 1.5 vs area×2; no retaliation; rebuilds in place).
### P0-4: aggro-as-Blue beats bot-tourney-as-Red 632.8–481.0
Seq: `R-chall-aggro-botIsred.moves.json`. WHAT: same family as neck — aggro hugs enemy nodes and keeps bot-Red's banked area at ~5–11 all game while banking its own ~17–22. (Bot-as-Blue beats aggro-as-Red comfortably 537.9–281.5, so this is a Red-chair-only failure: bot-Red can't build under contact.) CONFIRMED at @2000ms/turn — and WORSE with more thinking time (394–785): extra search only deepens the solitaire-building lines aggro pops. This rules out any budget artifact; the flaw is strategic (zero contact-avoidance / retaliation), not computational. PATCH (all P0s): value a break at ~loop-area × remaining-turns, retaliate instead of rebuilding in place, route expansion away from enemy frontier.
