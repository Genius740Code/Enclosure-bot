# TOURNEY PLAYBOOK (locked pre-tournament)

## Run this
`bot-tourney.js` — `bestTurn(state, 8000)`. Time-boxed copy of bot.js's 2-ply:
never throws, never illegal (all candidates engine-verified), always answers
(2-ply -> 1-ply -> first-legal -> timeout). Smoke-tested: 61ms early turns,
~3s by turn 9, hard-capped at budget. On 1min+15s clock this never flags.

## Verified engine facts that change play (Track C, confirmed live)
1. **Cross-color nesting DOUBLE-pays.** Enemy loop inside your loop banks for
   BOTH (outer keeps full area incl. their interior, they bank theirs too).
   "No double count" is same-color only. => Don't build inside enemy shadow
   expecting denial; DO build there yourself — it still pays you.
2. **Touch = break.** Endpoint touches, T-junction landings, moving off a
   coincident node ALL cut the whole enemy edge. Shared enemy corner = two
   breaks = your move throws. => Keep 1+ unit clearance on valuable edges;
   graze enemy frontier for free breaks (no separate attack move exists).
3. **Invincibility = current-turn edges only** (my live probe; starters never
   get it). One-turn grace during the immediately-following opp turn,
   then gone. => A break threat must be executed the very next turn or it
   evaporates. Your own older edges are ALWAYS cuttable.
4. **Timeout burns clock** (`moveNumber += actionsRemaining`) and never
   removes enemy edges. A wiped player can only timeout; game continues.

## Strategy (holds per Track B: turtle lost 2120-417 / 4709-409)
- Close a small loop by turn 2-3. Every turn of delay compounds against you.
- Petals, not one blob: single-neck enclosures die to one cut.
- As Red (2nd player, ~2.2x scoring edge observed): press expansion harder.
- As Blue: rounder shapes, faster close, take free graze-breaks.
- Vs chaos/aggro/junk moves: do NOTHING special. Bot replans from live state
  every turn — no opening book to break. Junk moves cost THEM tempo; keep
  banking. Only intervene if bot starts timing out (then lower budget to 3000).

## Still running while you play
Tracks A (bot tuning) + B (blob/neck/sprawl/aggro/sandbag) keep self-playing.
I will only ping you mid-tournament with a merge-worthy patch + numbers.
