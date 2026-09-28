# Site ops — Meridian upload & match flow

Site: https://constellation.blueshrimp.uk — API reference: https://constellation.blueshrimp.uk/bot-api.md
(fetched 2026-09-28; the doc is the source for every route below). Written by Lane E for the
overnight session. **Nothing was uploaded during this session; this is the recipe only.**

## Auth

The `POST /api/bots/:id/*` and `POST /api/evals` routes take a **signed-in session, not a bot
token** (bot-api.md "Other routes"). The session cookie is `meridian_session` per the overnight
brief — the API doc does not name the cookie, so confirm the name on first use (the practice
server at meaf.us uses `sst1_access`; production is expected to use `meridian_session`). Hosted-bot
job routes (`/api/bot/jobs*`) are different: they use `Authorization: Bearer TOKEN`.

Env: `ENC_USER` / `ENC_PASS` credentials live with the main session; never commit them.

## 1. Build the module

```sh
rustup target add wasm32-wasip1
cd retaliator && cargo build --release --target wasm32-wasip1
```

Verified 2026-09-28 (worktree of master @ eb100e9): `target/wasm32-wasip1/release/retaliator.wasm`,
**285,743 bytes (~0.27 MB, limit 8 MB)**, exports `meridian_abi` (returns 1), `meridian_alloc`,
`meridian_run`, imports only `wasi_snapshot_preview1`. Live-executed on 3 positions (start,
mid-game, cut-history) through the actual binary — all replies `{"move": id}` legal, verified by
replaying through `engine/engine.js`. Harness kept outside the repo at `/tmp/wasm-live/exec.mjs`.

## 2. Upload a version

`POST /api/bots/:id/versions` with the `.wasm` file as `application/wasm` (bot-api.md "Other
routes": "Uploads a version and makes it current"). Each upload is a version, named by its
SHA-256, and becomes current.

```sh
curl -X POST https://constellation.blueshrimp.uk/api/bots/BOT_ID/versions \
  -H "Content-Type: application/wasm" \
  -H "Cookie: meridian_session=SESSION" \
  --data-binary @retaliator/target/wasm32-wasip1/release/retaliator.wasm
```

- The web UI (My bots → Add a bot → WebAssembly) first tests the file on two positions and a timed
  move and shows what the module printed; the API route just uploads and makes it current.
- Limits: file 8 MB, reply 1 MB, memory 256 MiB, **3 versions per bot, 32 MB per account**.
- Timing: aim for `limits.moveTimeMs` (5,000 ms in games). **Until 4 October 2026** a move is
  stopped only after ~23 s (browser) / ~25 s (server); from that date a move past the limit counts
  as the bot's failure (tournament: forfeit).

## 3. Verify the upload / find the previous hash

```sh
curl -f https://constellation.blueshrimp.uk/api/bots/mine -H "Cookie: meridian_session=SESSION"
```

Returns your bots with their versions — note the new version's hash **and the previous current
hash** (needed for switch-back). `GET /api/games/GAME_ID` is public and shows which version a
finished game ran.

## 4. Version switch-back (rollback)

`POST /api/bots/:id/current` with `{"hash": "<sha256>"}` makes an uploaded version current.

```sh
curl -X POST https://constellation.blueshrimp.uk/api/bots/BOT_ID/current \
  -H "Content-Type: application/json" \
  -H "Cookie: meridian_session=SESSION" \
  -d '{"hash": "PREVIOUS_SHA256"}'
```

- **A game or match keeps the version it started with** — switch-back affects only new games and
  matches; running games are untouched.
- `DELETE /api/bots/:id/versions/:hash` deletes a version not in use by a running match
  (housekeeping only; keep the known-good version current before deleting anything).
- People are asked once per version before someone else's bot runs on their device.

## 5. Bot-vs-bot matches

`POST /api/evals` with `{"botA", "botB", "pairs", "games", "seed"}`:

```sh
curl -X POST https://constellation.blueshrimp.uk/api/evals \
  -H "Content-Type: application/json" \
  -H "Cookie: meridian_session=SESSION" \
  -d '{"botA": BOT_A_ID, "botB": BOT_B_ID, "pairs": 2, "games": 0, "seed": 7}'
```

- `botA != botB`: starts a bot match of `pairs` **pairs** of games (one pair = one game each
  colour). `botA == botB`: self-play of `games` games.
- A private bot's matches need its owner or a moderator.
- Results: `GET /api/games` (latest games) and `GET /api/games/GAME_ID` — both public. Feed game
  JSONs to `retaliator/examples/autopsy.rs` for per-action breakdowns.

## Status codes (bot-api.md)

| Status | Meaning |
|---|---|
| `401` | Auth missing or wrong. |
| `409` | The job, position or lease is stale — drop the result and retry. |
| `422` | The move or analysis is invalid — fix and reply again while the lease lasts. |
| `429` | Too many requests — wait before retrying. |

## Safety rules for this repo's sessions

- **Do not upload anything without the main session's sign-off** — uploads make a version current
  for new games immediately.
- Never switch the current version away from the known-good bot while ladder/tournament games may
  pair; switch back via §4 immediately after any experiment window.
- One polling loop per hosted bot; a bot is offline 90 s after its last request.
