# Track B — Red-Team Findings (strategy)

## TOP-LINE SUMMARY
- FLAGS: TWO break-first archetypes beat bot.js BOTH colors — neck-hunter (523–236 as Red, 990–512 as Blue) and pure aggro (573–352 as Red, 893–449 as Blue). One break zeroes a whole loop; bot (break weight 1.5 vs area x2) rebuilds instead of retaliating. Patch: raise break weight ~10x, add break-back retaliation, route expansion away from enemy frontier.
- Blob (dumb 1-ply max-area) splits by color (wins as Red 3879–2412, loses as Blue 2791–5227) and sprawler/sandbag/turtle all lose: Red second-player edge (~2x) dominates all non-breaking archetypes.
- Early area compounds: sandbag (build only after move 85) loses 2120–498 / 4613–497 — late building never catches up. No scenario favors turtling or sandbagging.

## Method
- Challenger archetypes in `chall-*.js` (rule-based 1-ply greedies, no 2-ply search, no opponent score modeling — genuinely different from `bot.js` 2-ply + 0.15 opponent-denial).
- Runner `runB.js` replays challenger `bestTurn` moves through `engine.js applyMove` (verifies legality on live state), 2 games per archetype (bot blue / bot red). Engines are deterministic → scores are exact, no repeats needed.
- Full move sequences: `chall-<name>-botIs<color>.moves.json`.

## Results table
| Archetype | Bot=Blue (bot–chall) | Bot=Red (bot–chall) | Beats bot? |
|---|---|---|---|
| turtle (tiny loop, passive) | 2120 – 417 BOT | 4709 – 409 BOT | No (2 losses) |
| blob (big single polygon, 1-ply area) | 2412 – 3879 CHALLENGER | 5227 – 2791 BOT | SPLIT (color decides) |
| neck-hunter (breaks first, graze) | 236 – 523 CHALLENGER | 512 – 990 CHALLENGER | YES — beats bot 2/2 |
| sprawler (many small loops) | 2206 – 708 BOT | 4819 – 825 BOT | No (2 losses) |
| aggro (hunt enemy nodes) | 352 – 573 CHALLENGER | 449 – 893 CHALLENGER | YES — beats bot 2/2 |
| sandbag (small until move 85) | 2120 – 498 BOT | 4613 – 497 BOT | No (2 losses) |

## Per-archetype analysis
### turtle — LOSES badly both colors
Final areas: blue 67.1 vs red 13.75 (bot blue game). Turtle's chebyshev<=1 interior wiggles bank ~0.5 area/turn while bot closes real loops by turn 2–3 and compounds. Confirms: under per-turn cumulative scoring, passive/defended-small-loop play is strictly dominated; patch direction for bot.js is NOT to add turtling — current greedy expansion is correct vs passives.
### neck-hunter — BEATS bot.js BOTH COLORS (primary flag)
Scores: as Red 523–236; as Blue 990–512. Full sequences: `chall-neck-botIsblue.moves.json`, `chall-neck-botIsred.moves.json`. Opening (vs bot blue): red spears down the center (15,9)->(12,9)->(9,9)->(6,9)->(3,8), then grazes blue's frontier nodes and pops blue's loop (blue area 9→4.5 on move 11, 4.5→0.5 on move 15) while banking its own small loops. WHY: a loop is a single point of failure — one break on any loop edge zeroes its whole area, and `quickScore` weights a break at only 1.5 vs areaGain x2, so bot keeps building fragile loops near the enemy spear instead of retaliating or routing around. Both players' areas stay tiny all game (final 5.5 vs 15.8 / 31.6 vs 15.6), and the breaker wins the low-area race ~2x. PATCH bot.js: (1) raise break weight an order of magnitude (a break denies ~loop-area x remaining-turns, not 1.5 pts); (2) add retaliation — when own loop was broken last turn, prefer breaking back over rebuilding in place; (3) route expansion away from enemy frontier nodes (neck grazes them for free breaks).
### blob — SPLIT: wins as Red, loses as Blue (color effect, not archetype)
As Red: 3879 vs bot-blue 2412. As Blue: 2790 vs bot-red 5227. Trajectory shows Red ahead every single turn from move 3 in both games — dumb 1-ply max-area greedy is roughly bot-equal, and Red's structural edge (2-action reply every round + last-move advantage) decides it. WHY it matters: bot.js's 2-ply + opponent-denial buys little vs a fast 1-ply area greedy; the compute would be better spent on break-awareness. No bot.js patch strictly implied beyond the neck findings — but note bot-as-Blue lost to a ~30-line greedy, so Blue-opening play is the weakest spot.
### aggro — BEATS bot.js BOTH COLORS (confirms neck flag, independent rule set)
Scores: as Red 573–352; as Blue 893–449. Full sequences: `chall-aggro-botIsblue.moves.json`, `chall-aggro-botIsred.moves.json`. Rule ignores area entirely (minimize endpoint distance to nearest enemy node, breaks tiebreak x500) yet wins — converging on the same center-spear opening as neck ((15,9)->(12,9)->(9,9)->(6,9)->(3,8)) and the same low-area breaking war (final areas 6.1 vs 18.6). WHY: two independently-written break-first rules both beat bot 2/2, so this is not a quirk — bot.js structurally underprices denial. A break's true value is ~(victim loop area x turns remaining), often 100+, vs the 1.5 weight in `quickScore` (`bot.js:65`) and the 0.15 opponent term (`bot.js:114`). Same patch as neck, plus: when areas are being suppressed game-wide, bot's area-greedy second ply optimizes noise — it needs a "brawl mode" that switches to pure denial once an enemy spear reaches its frontier.
### sprawler — LOSES both (bot out-builds it 62 vs 21 final area)
Many small chebyshev<=2 loops near home bank steadily but each closure costs actions for ~2-6 area while bot's big polygons compound faster. Petals-over-polygon hypothesis REJECTED in this form: small loops neither defend better (still one-break fragile) nor score faster per action.
### sandbag — LOSES worst of the builders (2120–498 / 4613–497)
Turtle until move 85 then blob: ends with 497 both colors — late building never recovers 85 turns of compounding. Early-area hypothesis CONFIRMED: every turn of delay is scored area forgone forever. (Bot-blue scored 2120.058838384 identically vs turtle and sandbag — deterministic bot unperturbed by anything the passive side did.)
## Hypothesis scorecard (from mission brief)
- Early cheap loop: CONFIRMED (sandbag crushed; all winners bank by move ~10).
- Avoid necks: SUPPORTED (neck/aggro win by popping loops at single-edge points of failure; bot builds necks near its frontier).
- Petals over big polygon: REJECTED (sprawler loses; blob roughly ties bot).
- Red structural advantage: CONFIRMED (~2x same-bot scores across turtle/blob/sprawl/sandbag pairs).
- Graze opponent frontier for free breaks: CONFIRMED (both winning archetypes spear the center and fight on the victim's frontier).
### (rest TBD)
