# PREFIX-ABORT tabulation (external Claude session, 2026-09-30) — PROVISIONAL

Source: 12 loss game JSONs on `origin/lane-v8-autopsy` (tip `e1b82d3`), replayed
through repo `engine/engine.js` (`applyMove`, stdlib only). 12/12 replays clean
(120 moves, final areas match JSON `state.areas`). Prefix idx k = after first k
moves. Foe = non-Riposte-v7 colour.

## Losses arm (complete, 12/12)

| game | winner | foe-area@4 | foe-area@6 |
|---|---|---|---|
| 16840108 (v7 blue vs v6 red) | foe | 0 | 0 |
| 1c69056c (VladNet blue vs v7 red) | foe | 0 | 4.5 |
| 21fc2945 (v7 blue vs v6 red) | foe | 0 | 0 |
| 30c7653b (GB blue vs v7 red) | foe | 0 | 0 |
| 3a414aad (v7 blue vs VladNet red) | foe | 4.5 | 4.5 |
| 9c3b27a5 (v7 blue vs Stompy red) | foe | 0 | 0 |
| ad65f054 (v7 blue vs GB red) | foe | 0 | 0 |
| b0ac4141 (Stompy blue vs v7 red) | foe | 0 | 9 |
| b4462808 (v7 blue vs v6 red) | foe | 0 | 0 |
| c20c750b (v7 blue vs v6 red) | foe | 0 | 0 |
| c3c8cf5b (v7 blue vs v6 red) | foe | 0 | 0 |
| d2d4b4fd (v7 blue vs GB 2.0 red) | foe | 0 | 0 |

Fire-rate: X<4.5 catches 3/12; 4.5<=X<9 catches 1/12; X>=9 catches 0/12.
9/12 games have foe-area 0 at both prefixes — weak early signal.

## Wins arm (MISSING — blocks certification)

The 5 v7 wins have no JSON in `research/games/` (none on any of 25 remote
branches). Named wins: `f81e3b7e`, `2f6343a3` (v7 Red beats v6, per
`research/c-v8-losses.md` Cluster 4); other 3 unnamed. Wins-arm fire-rate UNKNOWN.

## Verdict: PROVISIONAL LEAN-KILL, not certifiable

Kill-0 bar read as KILL-conditions (kill if fires in >=50% of wins OR catches
<2 of 12 losses): X>=4.5 is KILLED (<=1/12); X<4.5 catches 3/12 (survives the
losses floor barely) but needs the wins arm. Substance: 9/12 zeros = foe area
is not a useful early-abort signal. NEXT: fetch win JSONs via public site API
(`GET /api/games`, `GET /api/games/ID` — public, no auth), re-run same replay,
extend table, certify.
