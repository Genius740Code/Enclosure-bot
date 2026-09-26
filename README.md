# Enclosure-bot

A heuristic bot for the 2-player board game **Enclosure** (19×19, enclose
territory, score compounds every turn), built against the game's real rules
engine, plus tooling: local play UIs, a live-site connector, and a full
empirical research log (self-play matchups, archive mining, verification).

## How it plays (the strategy in 30 seconds)

- Score banks **every turn for both players**, so area closed early pays out
  for the rest of the game. Close a small loop by turn 2–3.
- Breaking is free when piggybacked on expansion you wanted anyway — graze
  the opponent's frontier. Pure break-chasing loses tempo (proven in
  `findings-robustness.md` P0-1…P0-4, and the retaliation gap in
  `findings-retaliate.md`).
- Petals, not one blob: a single-neck enclosure dies to one cut.
- Red (second player) has a ~2.2× structural edge; Blue must close fast.

## Files

| File | What |
|---|---|
| `engine.js` | Real rules engine (extracted from the site bundle — do not modify logic) |
| `bot.js` | Base heuristic bot (2-ply greedy + opponent denial) |
| `bot-tourney.js` | Tournament pick: time-boxed wrapper, adaptive clock, never throws/illegal |
| `botX-*.js`, `botA1-D1.js`, `botF-*` | Experiment variants (quiescence, transposition, criticality, ponder, retaliation) |
| `chall-*.js` | Sparring archetypes (turtle, blob, neck-hunter, sprawler, aggro, sandbag, random, junk) |
| `connector.js` | Live-site player (WebSocket: auth → subscribe/join → revision-checked moves) |
| `play.js` / `play.py` + `bridge.js` | Terminal / tkinter practice UIs (rules via the JS engine) |
| `server.js` + `play.html` | Browser practice UI, incl. bot-vs-bot watch mode |
| `runB.js`, `runR.js`, `run-match.js` | Self-play harnesses (all moves replay-verified legal) |
| `verifyC.js` | 22 engine zero-divergence checks |
| `mined-fetch.js`, `mined-analyze.js` | Site archive miner (needs `ENC_USER`/`ENC_PASS`) |
| `findings-*.md`, `TOURNEY-PLAYBOOK.md` | Research log + tournament playbook |

## Quick start

```bash
npm install        # ws for the connector
node play.js blue  # terminal: play the bot

python3 play.py    # desktop UI (needs node alongside)

node server.js 8901  # browser UI at http://localhost:8901
```

Bot vs bot watch: open the browser UI → **Watch bot vs bot**.

## Live play (your own account, your own risk — check event rules first)

```bash
export ENC_USER=you ENC_PASS=yourpassH1
node connector.js  # subscribes to tournament mt5x4j77, plays automatically
```

Never open the game page on the same account while it plays (the site
routes live updates to one connection). Credentials stay in env vars —
nothing secret is committed (see `.gitignore`).

## Key results (all empirical, via `engine.js` self-play)

- Passive/turtle play loses ~5× under cumulative scoring.
- Red second-player edge ≈ 2.2× in mirror matchups.
- Break-heavy archetypes (neck/blob/aggro) beat pure greedy expansion —
  retaliation valuation is the open patch (`findings-retaliate.md`).
- 104 archived human games replay bit-exact; winners out-build
  (max territory 16.8 vs 6.5), breaks are symmetric (5.3 vs 4.9).

## License

MIT — see `LICENSE`.
