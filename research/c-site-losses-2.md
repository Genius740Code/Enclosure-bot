# Lane C2 — Site-loss autopsy follow-up, 2026-09-28 (c-site-losses-2)

Branch `work-c2` (from `origin/lane-c-autopsy` @ `88e31fb`), code base `eb100e9` (V5-1).
ANALYSIS ONLY — no `retaliator/src/*` changes; this file is the only repo artifact. Continues
`research/c-blunder-catalog.md` (§3.5 found the deployed Riposte v3 being farmed by AngelBot).

## 0. Scope, method, and what "eval" means here

- **Window:** every game the API could page back from 2026-09-28 01:20Z to 2026-09-27 10:38Z
  (800 of 1736 total games; `GET /api/games?limit=100` + cursor). 114 involve a Riposte-family
  bot. **53 distinct finished losses** vs rivals were fetched and replayed (self-play between our
  own versions excluded; the two 13:01Z AngelBot losses `4f0c9650`/`3910c73c` already autopsied in
  the catalog are referenced, not redone). Byte-identical dedup check: **0 duplicates** (unlike
  the probe_league §2.3 double-count, every site game here is distinct — the site forces a
  different opening each game).
- **Replay:** the mandated `cargo run --release --example autopsy -- GAME.json COLOR` for all 53
  (transcripts in `/tmp/opencode/sitegames/autopsies/`), plus a custom analyzer crate kept
  OUTSIDE the repo (`/tmp/opencode/sitegames/analyzer`, path-deps into this worktree, no repo
  files touched) that logs per action: move id, kind, both areas/scores, the cut edge, scoring
  deltas — and `search::evaluate` before each of our actions. Game JSONs: `/tmp/opencode/sitegames/losses/`.
- **Numbering:** site move lists are engine-numbered — action n here = engine action n
  (the catalog's §0 "probe-n = engine − 1" caveat does NOT apply to site games).
- **Eval convention:** `evaluate()` is Blue-lead (positive favors Blue); all evals below are
  sign-flipped to OUR perspective. Eval = the current master brain (V5-1). **Version fidelity,
  measured with `best_move_with_avoid` on the deployed avoid-reconstruction (lib.rs::replay
  semantics, CUT_MEMORY 6): deployed riposte v4 ≡ master `eb100e9` — 59–60/60 move matches in
  every v4 game tested (446956a1 60/60, a3f5f01c 60/60, d9a0fbe3 60/60, a4b5e81b 59/60,
  db7641e1 59/60).** So source-level findings speak *exactly* for the v4 losses. Deployed v3
  diverges (36–48/60 on AngelBot/VladNet games), v1 48/60 — for those games the eval trajectory is
  "our current engine's view", not the deployed brain's.

## 1. Headline scoreboard (rivals' record vs ALL our bots, finished games in window)

| rival | W–L vs us | note |
|---|---|---|
| Great Barrier | **0–19** | mega-loop; 3 more losses on v1 at 11:12, 10 at 12:09, 6 on v2 |
| AngelBot WASM | **0–7** | farming + bank-sacrifice; most efficient exploiter (§3) |
| VladNet | **0–6** | farm cycle; finished the two v3 games the catalog saw live |
| medium capybara net v5 | **0–4** | swept v4 4–0; low-tempo farmer (§4.6) |
| xmybot | **0–2** | mega-blob, beat v4 by 4949 and 819 (§4.1) |
| Great Barrier 2.0 | **0–2** | bigger loop than GB (130 vs 109) |
| small baby search bot v4 | **0–2** | beat v2 via opening tempo |
| Atlas v2 | 1–1 | won the v4 game with a loop that lands act 41–51 |
| Scout (site builtin) | 2–2 | out-closed v2 43–14 (§4.5) |
| Flux-2 | 1–3 | opening-tempo kills as blue (§4.3) |
| bowot | 1–3 | close losses via endgame farm grind (§4.4) |
| Roxbot | 1–3 | early-area kills |
| Flux | 5–1 | only rival we beat; its 1-pt win over v2 is §4.4 |
| Atlas v1 | 4–0 | we win |

**Aggregates over the 53 losses:** mean area **30.7 vs 49.8** (them), peak area 62.6 vs 95.0.
Close chains ≥ +4.5 popped within 3 actions: **929/1522 = 61%**. Our closes 1736 vs their 930
(we out-close 1.9:1 and still lose — closing is not the currency, area-lifetime is). Our cuts
1515, of which **706 = 47% popped nothing** (enemy area didn't fall). Cuts suffered 1890.
Enemy close+cut-in-one-move (own gain ≥ 4.5 with a break): **421 across 50/53 games** — worst:
Scout 28, Flux-2 28, AngelBot 22/21/19. We also play close+cut combis (702) but ours are mostly
balloon re-closes that register a break (the §1.5 `[40]` pattern), not durable double-duty banking.
**First-close timing: we close FIRST in the losses** — ours avg act 4.0 vs theirs 5.6 (median
4/6); re-confirms catalog §1.2 that first-close is not the discriminator. Eval-flip histogram
(last our-action with eval ≥ 0): 25 games never durably ahead (flip at 119), 11 flip at act 49
(the GB window), 3 at act 5, 2 at 13 — early-death games are all blue-side (§4.3).

**Answer to the shared cross-cutting question:** every rival that beats us — transformer
(VladNet), alpha-beta (AngelBot), rule bots (GB family, Flux, Roxbot, bowot), the builtin Scout,
and neural capybara — wins on the same **area-lifetime asymmetry**: their ground survives, ours
recycles. New: two rivals (xmybot, GB family) mostly skip farming entirely and win by building
one invincible mega-territory (§4.1) — a mechanism the catalog did not name.

## 2. Master table — all 53 finished losses (site actions; areas at game end)

cols: 1stCls = first close act ours/theirs; cls = closes ours/theirs (gain >= 2.0); cuts =
inflicted/suffered; wst = our cuts that popped nothing; dd = enemy close+cut moves; farm = our
>= +4.5 chains popped within 3 actions / total; meanA = time-average area ours/theirs; endA =
final area ours/theirs; flip = last our-action with eval >= 0; peak/min = our eval extremes.

| id | created | ver | clr | opponent | us-them | 1stCls | cls | cuts | wst | dd | farm | meanA | endA | flip | peak | min |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| aefa3637 | 09-27T10:38 | v1 | red | AngelBot WASM | 1247-3291 | 3/4 | 31/26 | 32/43 | 9 | 19 | 11/20 | 20/53 | 17/33 | 7 | 6:64.8 | -2615.9 |
| e8fca105 | 09-27T10:38 | v1 | blue | AngelBot WASM | 1437-3017 | 5/6 | 25/25 | 32/40 | 6 | 18 | 13/21 | 23/49 | 34/43 | 1 | 1:0.0 | -1914.6 |
| 3120f6da | 09-27T11:12 | v1 | blue | Great Barrier | 2590-4448 | 5/6 | 42/10 | 17/33 | 8 | 1 | 23/39 | 42/72 | 38/117 | 49 | 36:852.4 | -2071.3 |
| 4975d4e6 | 09-27T11:12 | v1 | red | Great Barrier | 2380-4564 | 3/4 | 34/16 | 19/33 | 8 | 9 | 13/26 | 39/74 | 29/117 | 51 | 38:843.0 | -2815.5 |
| 16a589c2 | 09-27T11:12 | v1 | blue | Great Barrier | 3050-4344 | 5/6 | 43/8 | 25/32 | 18 | 0 | 29/42 | 49/70 | 49/113 | 49 | 36:923.6 | -1491.0 |
| a3702c60 | 09-27T12:09 | v1 | red | Great Barrier | 2555-4238 | 3/4 | 39/10 | 23/32 | 15 | 1 | 25/38 | 41/69 | 34/112 | 51 | 38:845.6 | -1771.2 |
| bb4dc0dd | 09-27T12:09 | v1 | blue | Great Barrier | 2507-4401 | 5/6 | 37/10 | 22/32 | 15 | 4 | 19/36 | 40/71 | 62/112 | 49 | 37:726.4 | -2168.6 |
| 57c9fd39 | 09-27T12:09 | v1 | red | Great Barrier | 2555-4220 | 3/4 | 39/9 | 24/32 | 16 | 1 | 25/38 | 41/68 | 34/112 | 51 | 38:851.0 | -1753.2 |
| 7b84b2db | 09-27T12:09 | v1 | blue | Great Barrier | 2475-4371 | 5/6 | 40/10 | 22/34 | 12 | 3 | 22/34 | 40/71 | 58/118 | 49 | 37:880.5 | -2132.8 |
| 02023cd5 | 09-27T12:09 | v1 | red | Great Barrier | 2380-4559 | 3/4 | 34/16 | 19/33 | 8 | 9 | 13/26 | 39/74 | 29/117 | 51 | 38:845.6 | -2810.5 |
| 276ab448 | 09-27T12:09 | v1 | blue | Great Barrier | 2507-4401 | 5/6 | 37/10 | 22/32 | 15 | 4 | 19/36 | 40/71 | 62/112 | 49 | 37:726.4 | -2168.6 |
| 21281ebe | 09-27T12:09 | v1 | red | Great Barrier | 2555-4220 | 3/4 | 39/9 | 23/32 | 15 | 1 | 25/38 | 41/68 | 34/112 | 51 | 38:851.0 | -1750.2 |
| d7ef1de4 | 09-27T12:09 | v1 | blue | Great Barrier | 2501-4359 | 5/6 | 43/8 | 21/34 | 16 | 1 | 30/43 | 41/71 | 42/114 | 49 | 37:811.8 | -2044.1 |
| 35e64aae | 09-27T12:09 | v1 | red | Great Barrier | 2555-4220 | 3/4 | 39/9 | 23/32 | 15 | 1 | 25/38 | 41/68 | 34/112 | 51 | 38:851.0 | -1750.2 |
| fdc6d61e | 09-27T12:09 | v1 | blue | Great Barrier | 2511-4529 | 5/6 | 32/16 | 23/33 | 11 | 9 | 15/28 | 40/72 | 59/123 | 49 | 37:854.0 | -2425.1 |
| 5bbee7e7 | 09-27T12:33 | v2 | red | VladNet | 1860-2417 | 3/4 | 40/15 | 34/41 | 25 | 4 | 24/37 | 31/40 | 16/39 | 55 | 27:199.2 | -926.5 |
| fa1df9b9 | 09-27T12:33 | v2 | blue | VladNet | 1576-2313 | 5/6 | 39/9 | 34/41 | 27 | 1 | 30/38 | 26/38 | 30/34 | 57 | 24:216.4 | -1272.2 |
| 200389aa | 09-27T12:33 | v2 | blue | VladNet | 1048-1995 | 5/10 | 37/15 | 34/42 | 22 | 2 | 16/23 | 18/33 | 9/32 | 61 | 29:346.9 | -1177.7 |
| 80ea1b0e | 09-27T12:33 | v2 | red | VladNet | 1691-2389 | 3/4 | 45/10 | 35/42 | 30 | 2 | 30/41 | 28/39 | 29/48 | 35 | 27:189.6 | -1047.4 |
| dfa99037 | 09-27T12:33 | v2 | blue | Great Barrier | 2456-4432 | 5/6 | 39/13 | 22/33 | 14 | 6 | 24/37 | 39/72 | 37/128 | 49 | 37:776.0 | -2290.2 |
| f76a0178 | 09-27T12:33 | v2 | red | Great Barrier | 2869-4257 | 3/4 | 38/12 | 24/32 | 16 | 4 | 21/36 | 46/69 | 40/113 | 55 | 38:851.0 | -1572.8 |
| 4a2e3a23 | 09-27T12:33 | v2 | red | Great Barrier | 2706-4307 | 3/4 | 35/15 | 23/32 | 11 | 9 | 19/33 | 44/70 | 41/108 | 55 | 38:851.0 | -1856.7 |
| b1542a7e | 09-27T12:33 | v2 | blue | Great Barrier | 2748-4514 | 5/6 | 36/13 | 21/34 | 13 | 8 | 22/33 | 44/73 | 37/112 | 61 | 36:834.8 | -2003.1 |
| b64a562b | 09-27T13:00 | v3 | red | VladNet | 1320-1840 | 3/4 | 39/15 | 25/39 | 17 | 3 | 22/32 | 21/30 | 9/39 | 79 | 59:369.8 | -728.3 |
| 6d96df35 | 09-27T13:00 | v3 | blue | VladNet | 1257-1686 | 5/6 | 46/12 | 36/44 | 23 | 2 | 31/38 | 20/27 | 14/35 | 49 | 33:437.6 | -687.9 |
| 1c7b1aa1 | 09-27T13:01 | v3 | red | AngelBot WASM | 502-997 | 3/4 | 22/26 | 46/50 | 18 | 22 | 8/17 | 8/16 | 0/0 | 19 | 6:68.8 | -895.4 |
| a4b5e81b | 09-27T19:51 | v4 | blue | medium capybara net v5 | 751-1113 | 5/6 | 28/9 | 33/42 | 21 | 2 | 17/25 | 12/18 | 17/22 | 17 | 8:105.6 | -435.1 |
| 76074a41 | 09-27T19:51 | v4 | red | medium capybara net v5 | 849-1289 | 3/4 | 26/18 | 30/41 | 12 | 8 | 9/19 | 13/21 | 16/24 | 23 | 15:172.0 | -626.1 |
| b4dd04bf | 09-27T19:51 | v4 | blue | medium capybara net v5 | 945-1187 | 5/6 | 30/15 | 37/41 | 18 | 5 | 12/19 | 15/19 | 23/24 | 25 | 8:91.2 | -392.3 |
| 446956a1 | 09-27T19:51 | v4 | red | medium capybara net v5 | 748-1214 | 3/4 | 26/13 | 31/42 | 19 | 1 | 10/21 | 12/20 | 19/47 | 23 | 11:119.9 | -556.2 |
| 6c4c2a2c | 09-27T23:27 | v4 | red | Atlas v2 | 2641-4634 | 3/4 | 37/12 | 23/31 | 13 | 3 | 17/31 | 43/75 | 35/129 | 51 | 46:1099.3 | -2101.3 |
| 0ac7f8b4 | 09-27T23:27 | v2 | red | Great Barrier | 2869-4257 | 3/4 | 38/12 | 24/32 | 16 | 4 | 21/36 | 46/69 | 40/113 | 55 | 38:851.0 | -1572.8 |
| 89acb586 | 09-27T23:27 | v1 | red | Great Barrier 2.0 | 2617-4662 | 3/4 | 36/10 | 26/33 | 19 | 4 | 22/36 | 41/76 | 36/117 | 51 | 38:773.4 | -2271.1 |
| 4ba0ff86 | 09-27T23:27 | v1 | blue | Great Barrier 2.0 | 2799-4939 | 5/6 | 37/16 | 25/34 | 12 | 7 | 23/35 | 44/80 | 41/124 | 49 | 37:659.2 | -2409.5 |
| 001a0f49 | 09-27T23:27 | v2 | blue | Great Barrier | 2679-4331 | 5/6 | 44/6 | 18/34 | 16 | 0 | 27/38 | 43/70 | 42/116 | 49 | 37:733.2 | -1803.7 |
| 37946f6f | 09-27T23:43 | v3 | red | AngelBot WASM | 594-1569 | 3/4 | 31/26 | 37/49 | 13 | 15 | 13/23 | 10/25 | 13/28 | 3 | 3:-0.0 | -1211.3 |
| abaea489 | 09-27T23:43 | v1 | blue | Roxbot | 1340-1982 | 5/6 | 28/28 | 37/37 | 10 | 17 | 14/20 | 21/32 | 20/13 | 13 | 12:59.4 | -944.1 |
| 83f11a1e | 09-27T23:43 | v2 | blue | Scout | 1136-1275 | 5/6 | 14/43 | 41/37 | 2 | 28 | 2/13 | 18/20 | 16/15 | 45 | 29:136.1 | -351.1 |
| a3f5f01c | 09-27T23:43 | v4 | red | xmybot | 2929-7878 | 3/4 | 13/3 | 2/4 | 1 | 0 | 1/10 | 48/128 | 60/257 | 55 | 54:1413.4 | -5034.1 |
| 4929b951 | 09-27T23:43 | v3 | blue | AngelBot WASM | 180-344 | 5/6 | 24/27 | 45/54 | 15 | 21 | 10/15 | 3/4 | 4/8 | 53 | 29:74.5 | -389.1 |
| e6bf571b | 09-27T23:43 | v4 | blue | xmybot | 3603-4422 | 5/6 | 20/5 | 1/11 | 0 | 1 | 1/14 | 59/70 | 56/211 | 89 | 72:2397.3 | -880.9 |
| db7641e1 | 09-28T00:02 | v4 | blue | Flux-2 | 1060-2489 | 4/6 | 21/31 | 34/39 | 7 | 20 | 9/17 | 17/41 | 6/16 | 5 | 5:61.8 | -1830.4 |
| abb62022 | 09-28T00:02 | v1 | blue | Scout | 1444-1513 | 5/14 | 31/29 | 38/37 | 11 | 12 | 19/29 | 23/24 | 20/25 | 101 | 49:288.3 | -249.2 |
| f9c819ed | 09-28T00:02 | v3 | blue | bowot | 1658-1663 | 4/10 | 34/26 | 34/36 | 9 | 8 | 22/32 | 27/27 | 19/22 | 117 | 101:172.2 | -199.0 |
| 20bf5fac | 09-28T00:02 | v3 | red | bowot | 1548-2203 | 3/4 | 31/30 | 32/38 | 7 | 14 | 18/30 | 25/35 | 22/71 | 11 | 11:6.0 | -906.3 |
| d9a0fbe3 | 09-28T00:02 | v4 | red | Flux-2 | 1498-1615 | 3/4 | 13/43 | 40/37 | 1 | 28 | 1/11 | 24/26 | 20/5 | 67 | 59:30.7 | -687.2 |
| 4bf39c48 | 09-28T00:02 | v2 | red | Flux | 2641-2642 | 3/4 | 37/17 | 23/33 | 14 | 2 | 18/34 | 43/43 | 46/68 | 95 | 38:613.2 | -88.1 |
| ab1b58f5 | 09-28T00:38 | v2 | red | small baby search bot  | 869-1354 | 3/4 | 28/28 | 41/36 | 12 | 10 | 18/27 | 14/22 | 7/14 | 11 | 6:68.8 | -698.3 |
| d5408f1f | 09-28T00:38 | v3 | blue | Flux-2 | 1527-1734 | 4/6 | 23/37 | 42/32 | 10 | 18 | 11/18 | 25/28 | 49/20 | 5 | 5:25.8 | -809.7 |
| 69f08058 | 09-28T00:38 | v2 | blue | small baby search bot  | 1129-1779 | 4/6 | 29/26 | 37/40 | 13 | 14 | 18/25 | 18/29 | 16/30 | 5 | 5:25.8 | -922.4 |
| 95d14ddf | 09-28T01:00 | v4 | red | Roxbot | 2117-3014 | 3/4 | 27/23 | 30/30 | 11 | 8 | 10/24 | 35/49 | 34/38 | 55 | 11:124.1 | -1108.9 |
| a5ab11d0 | 09-28T01:00 | v1 | blue | bowot | 1374-1666 | 5/18 | 24/32 | 34/36 | 4 | 18 | 12/21 | 22/26 | 14/15 | 29 | 28:22.1 | -577.2 |
| f0d17dcc | 09-28T01:00 | v4 | blue | Roxbot | 1241-2000 | 5/6 | 36/18 | 39/37 | 17 | 9 | 20/31 | 20/33 | 19/27 | 13 | 12:3.4 | -1026.4 |

## 3. Focus Q1 — does AngelBot (alpha-beta) exploit the same area-lifetime asymmetry?

**Yes — and it is the most efficient exploiter we face (7–0 vs us in the window).** Five new
AngelBot losses autopsied (plus the two in the catalog): v3 games `1c7b1aa1` (997–502),
`37946f6f` (1569–594), `4929b951` (344–180); v1 pair at 10:38Z `aefa3637` (3291–1247),
`e8fca105` (3017–1437). Signature matches the catalog's VladNet fingerprint exactly:

- **Every >= +4.5 close of ours is answered within 3 actions** (farm rates 8/17, 13/23, 10/15,
  11/20, 13/21) with **reused pop ids**: `1c7b1aa1` runs a *rhythmic 12-action cycle* from act
  48 to 81 — ids 14942, 12114 (x3), 10600 (x3), 12475, 17102, 14882 in the same rotation; the
  balloon farm cycle of catalog §2.4, live on site against the alpha-beta engine.
- **Its own ground is durable; ours is recyclable**: mean area 16–53 (AngelBot) vs 3–23 (us);
  in the 10:38 games its web grows to 64.7–86.3 area while ours oscillates 13–31.
- **Close+cut double-duty**: 19–22 such moves per game (its Connect lands a big close AND cuts
  us in the same action).

**Two NEW refinements, not in the catalog:**

1. **Bank-sacrifice tempo trade.** AngelBot *lets* us demolish its already-banked loops for
   zero score recovery, then re-closes bigger. In `1c7b1aa1` we land +15.0 [43], +22.8 [46],
   +37.8 [50] (its area 22.7 -> 10.0, later 0.0 at acts 90–100) yet the score gap only widens
   (220 us vs 386 them at 40 -> 924 vs 471 at 110): score is cumulative, cutting a banked loop
   recovers nothing, and it re-opens 47.5 area by act 60 while we spend actions demolishing.
   Our engine pays us a break bonus for these worthless demolitions (47% of all our cuts across
   the 53 losses pop nothing). `4929b951` is the pure form: both sides at ~0 area all game
   (mean 2.8 vs 4.1), our 45 cuts deal real pops (19.9, 25.2, 19.5, 24.1) — and it wins
   comfortably on the bank, 344–180.
2. **Never lets a balloon start (as the stronger side).** In `37946f6f` we were never ahead
   after act 3 (peak eval −0.0); every close from act 3 was popped within 1–3 actions, so we
   never built the +600–850 eval peaks that GB gives us before killing us. Against AngelBot the
   loss has no balloon phase at all — just a continuous grind (eval −111@10, −50@30, −372@50,
   −807@70, −1051@90).

**Conclusion:** the disease is ours, not theirs — confirmed across engine families
(transformers, alpha-beta, rules, builtin, neural nets). AngelBot additionally *weaponizes the
cumulative-score rule*: it banks early and freely trades area-for-tempo afterwards.

## 4. Focus Q2 — NEW signatures not in c-blunder-catalog.md

### 4.1 S1: the mega-blob endgame swallow (the biggest new finding)

xmybot beat v4 **7878–2929** (`a3f5f01c`) and **4422–3603** (`e6bf571b`); the same mechanism
decides GB 0–19, GB2.0 2–0, Atlas v2's win, and Flux's 1-point win — but xmybot is the pure case:

- Enemy keeps area near **0** for 40+ actions while drawing one giant loop far from our lines.
  Then it closes: `a3f5f01c` 3.6 -> **180.3** (act 60) -> **256.9** (act 80, a quarter of the
  board) and holds it; `e6bf571b` 10.8 -> 116.9 (act 80) -> 214.5 (act 110); GB: 0 -> 108
  (acts 41–51) held forever; GB2.0: -> 130.5 by 61; Atlas v2: -> 126.8 by 61; Flux: -> 68.5
  by 81 and we were **+416 on score at act 81 and lost by 1**.
- **The eval calls it a rout for us until the moment it closes.** Our peaks: +1413 at act 54
  (`a3f5f01c`, enemy area 3.6!), +2397 at act 72 (`e6bf571b`), +1099 at 46 (Atlas v2), +924 at
  36 (GB 16a589c2). The crashes after the swallow: −5031, −881, −2101, −1491. In `e6bf571b` we
  led the SCORE 2048–505 at act 80 and still lost by 819.
- **Our cuts are structurally useless against it**: 2 cuts suffered in `a3f5f01c`, 11 in
  `e6bf571b` (2.2–9.0 each) — the loop boundary is simply out of reach of our placement radius
  (Chebyshev 3), and we never routed toward it. 47% wasted-cut rate is the mild version; here
  cutting is *impossible* — the counter is to contest the territory BEFORE it closes.
- **Why the eval is blind:** `worth(player) = score + area × min(events_left, 12) +
  room × 0.4 × min(events_left, 1)`. Enemy area ~0 for 40 actions (priced ~0), enemy room
  (convex-hull) priced 0.4 × once — no horizon compounding, no closure-progress, no term for
  "their open web is 8 actions from enclosing 200". Our own area × 12 dominates, so the eval
  reports +1000–2400 while the game is lost. **Depth cannot fix this**: the enemy's build is a
  20–40-action plan; a 2–3-ply reply search never contains the closing move. It must be an
  eval/routing fix.

### 4.2 S2: wasted-cut tempo bleed (measured at 47%)

Across the 53 losses we played 1515 cuts; **706 (47%) left enemy area unchanged** — we paid the
break bonus and lost the tempo for nothing. Extremes: vs Scout 41/41 cuts wasted (83f11a1e), vs
Flux-2 34/34 and 40/40 (db7641e1, d9a0fbe3). Two causes, both priced wrong: (a) invincible
thickets — cutting one exposed edge doesn't drop the region (V4d2 confirmed at scale), and
(b) **already-banked ground** — AngelBot's bank-sacrifice above. This upgrades catalog
hypothesis #3's "worthless cut paid a bonus" from one anecdote to 47% of our cut tempo.

### 4.3 S3: blue opening tempo hole (all three act-5 flips are blue)

As Blue we get one forced first action plus a site-chosen opener (never D10-F7 — catalog §0),
and against fast-opening rivals the game is over by act 11: `db7641e1` (Flux-2, eval flips at
**act 5**, enemy 51–58 area by act 31, ours never recovers), `d5408f1f` (Flux-2, flip 5),
`69f08058` (small baby, flip 5, enemy area 19.5 vs our 9.0 at act 11), `abaea489`/`f0d17dcc`
(Roxbot, flip 13), `a5ab11d0` (bowot, flip 29). The red ladder (16013/9512/14873/9458 = +4.5
x4 by act 19) is stable in every game — the hole is blue-specific and the §0 opener-response
experiment (a no-op bug in probe_bluechair) is still the untested counter.

### 4.4 S4: endgame bank-race grind (close losses decided in the last 20 actions)

`f9c819ed` (bowot, 1663–1658): score tied at acts 81, 91, 101, 111 (1534–1534!) — then bowot's
farm tempo (22/32 of our closes popped; ids 10598, 497, 5096 reused) takes the last 5-point
margin. `4bf39c48` (Flux, 2642–2641): we led +416 at act 81; Flux's 66–69 living area out-banks
our 38–52 recycled ground across the final 10 scoring events and it wins by 1. `b64a562b`
(VladNet, 1840–1320): **we led +42 at act 80**; VladNet's 36–39 stable area banks ~200/10
actions while our 5–13 re-close farms bank ~60; we lose the last 40 actions by 520. When
scoring events remaining are few, the living-area differential is the whole game — and our bot
keeps playing the opening-style farm game instead of denial.

### 4.5 S5: the builtin Scout out-closes us 43–14

`83f11a1e` (v2, 1275–1136): Scout plays 28 double-duty close+cut moves, holds 15–32 area, and
ALL 41 of our cuts bounce (47% average is bad; 100% is a shape problem). We out-first-closed it
(act 5 vs 14) and still lost the banking race. Our DENSE_BONUS thicket exists but Scout's
closing shape banks more per move. Losing 2–2 to the site baseline is the clearest regression
alarm in the set.

### 4.6 the capybara sweep (named game 446956a1, deep dive in §5)

A neural "medium net" swept v4 4–0 with the *anti-balloon* discipline: it never lets any close
of ours survive (10–17 farmed chains per game from act ~17), while holding a modest 16–24 area
web that we cannot touch (our wasted cuts 12–21/game), growing it to 36–47 late. All four games
are low-tempo grinds (finals 751–1289); our eval peaks are tiny (+91–172) and the flips come
early (acts 17–25). It is AngelBot's refinement 2 with a neural net — the correct farming
strategy executed patiently.

## 5. Per-game deep dives (farm chains with move ids; "us" = our bot)

### 5.1 `446956a1` capybara (Blue) 1214–748 riposte v4 (Red) — the named loss

We open the standard red ladder (+4.5 [3] id 16013, [7] 9512, [11] 14873, [15] 9458, [19]
15899) and reach our best position at act 20: area 22.5 vs 19.0, score 112–73, eval +120.
**Divergence: acts 21–40** — capybara dismantles the ladder with 11697 [21], 1249 [24], 1610
[25], 14549 [28]+[40]+[57] (x3), 7329 [32], 1914 [33], 93 [36], 16715 [37], 12079 [41]+[64]+[89]
(x3); our area 22.5 -> 4.5, eval +119@10 -> −9@30 -> −334@50, never positive again (flip 23).
From there the farm cycle: **id 17112 pops us 4x** ([52], [72], [92], [104]), 12079 3x, 14549
3x, 2639 x2, 2332 x2, 14566 x2, 14511 x2, 4859 x2 — cycles spanning **20–40 actions**, far
beyond CUT_MEMORY 6. We re-close the same loops with the same ids: 16013 ([3], [51]), 16070
([31], [58]), 9572 ([46], [70]), 17096 ([50], [83]), 15293 ([71], [94]), 8755 ([74], [114]).
Capybara's own web: area **19.0 flat from act 20 to 90** (untouched by our 31 cuts, 19 wasted),
then it finally grows to 46.6 by the end. Area-lifetime: them 19+ alive for 100 actions; us
oscillating 4–22. **Hypothesis -> B** (farm-cycle memory + survival pricing; §6 H1).

Sister games, same shape: `a4b5e81b` 751–1113 (flip 17, farmed 17/25, every close >= +4.5 from
act 17 popped; ids 12394, 14994, 2357, 15660, 537, 161, 15263, 7737, 15298 in 1–3-action
chains); `b4dd04bf` 945–1187 (flip 25, farmed 12/19); `76074a41` 849–1289 (us Red, flip 23,
farmed 9/19, our +23.5 [86] popped by 15749 [88], +14.9 [43] by 1650 [44]).

### 5.2 AngelBot (v3) — details in §3; chains for the record

`1c7b1aa1` 997–502 (us Red): pop rotation 14942/12114/10600/12475/17102/14882 acts 48–81;
our re-closes 16736 [55]/[67]/[79], 15348 [59]/[71] feed the same cycle; our big demolitions
+15.0 [43], +22.8 [46], +37.8 [50] recover zero bank; both areas end 0.0, score decided by
AngelBot's early banking (220–386 gap at act 40 -> 471–924 at 110). Final areas 0/0.
`37946f6f` 1569–594 (us Red): never ahead (peak −0.0@3); pops 12059 x2, 14834 x2, 146 x2,
15610 x2, 12385 x2; our re-closes 16676 [43]/[54], 15237 [99]/[119]; our rare big lands
(+14.8 [42], +27.1 [62], +32.3 [86]) are rebuilt within 10 actions (AngelBot area 24 -> 36).
`4929b951` 344–180 (us Blue): mutual-zero cut war, mean areas 2.8/4.1, 21 dd moves by
AngelBot; our 45 cuts pop it repeatedly (19.9, 25.2, 19.5, 24.1) — the bank-sacrifice game.
**All three -> B** (H2/H3); A secondary on the [50] position of 1c7b1aa1 (does depth-3 see
that demolishing banked ground is worthless?).

### 5.3 VladNet finishes the two v3 games the catalog saw live

`b64a562b` 1840–1320 (us Red): **we lead +42 at act 80** (1076–1034). Every balloon is popped
in 1–2 actions: +23.2 [47] by 14568, +37.1 [66] by 431, +50.2 [74] by 4873, +48.7 [78] by 868;
id 452 pops us 3x ([89], [101], [117]). VladNet's area never leaves 31–39; ours ends 8.9.
Eval peak +370@59, flip 79, min −728. The last 40 actions lose the bank race 520 — S4.
`6d96df35` 1686–1257 (us Blue): balloon ladder popped all game — +23.8 [32] by 2702, +21.2
[45] by 842, +29.4 [53] by 2707 (x3 popper: acts 27, 30, 55), +13.7 [61] by 902, +12.4 [65] by
4829 (x2), +37.7 [73] by 12372, +14.1 [77] by 12046, +16.7 [81] by 7375. VladNet 33–36 stable.
Peak +438@33, flip 49. **Both -> B** (H1 survival + H4 endgame).

### 5.4 xmybot — the blob swallow (S1; §4.1)

`a3f5f01c` 7878–2929 (v4 Red): enemy area 0.0/1.4/2.6/3.6/3.6 at acts 10–50 -> 180.3 @60 ->
255.0 @80 -> 256.9 to the end; our area 9 -> 60.0 (flat, alive); cuts suffered 4, ours 2.
Eval +1413@54 -> −5031@110. `e6bf571b` 4422–3603 (v4 Blue): our best build on record — area
13.5 -> 86.5 by act 80, score 2048–505 (+416-level lead like 4bf39c48) — then 10.8 -> 116.9
@80 -> 205.4 @100 -> 214.5; eval +2397@72 -> −870@120. **Both -> B** (H5 closable-potential
term), A confirms depth blindness at a3f5f01c pre-60 / e6bf571b pre-80.

### 5.5 GB family / Atlas v2 — mega-loop at act 41–51 (S1)

GB cluster (v1/v2, 19 games, 11:12Z + 12:09Z + 12:33Z + 23:27Z): identical arc every game —
GB area 0–5 through act 31, loop lands 56.5 @41 -> 108 @51, held 108–127 forever; our eval
peaks +726–923 at acts 36–38 (blue, flip 49) / +845–851 at 38 (red, flip 51); our closes 32–43
vs GB's 8–16; our cuts 17–25 (wasted 8–18); farmed 13–30. Best GB loss score 3050 (16a589c2,
our peak area 89.5 — still −1294). `89acb586` GB2.0 4662–2617: loop -> 130.5 @61, peak +773@38,
flip 51, min −2271; pops 1511 x2, 9972 x2, 14447 (−33.0 [80] on our +4.5+[33.0] pair), 84
(−54.5 [120]). `6c4c2a2c` Atlas v2 4634–2641: loop 9.0 @41 -> 65.2 @51 -> 126.8 @61; peak
+1099@46, flip 51, min −2101. `001a0f49` v2 4331–2679: same, peak +733@37. **All -> B (H5).**

### 5.6 Flux-2 — opening tempo + double-duty banking

`db7641e1` 2489–1060 (v4 Blue): **flip at act 5**. Flux-2 holds 51–58 area from act 31; our
+35.0 balloon [45] popped instantly by 1942 [46]; 20 dd moves (11613, 16787 x3, 15244 x2,
14987, 17150, 11733...); our 34 cuts ALL wasted. `d9a0fbe3` 1615–1498 (v4 Red): the closest
v4 loss — we even end with MORE area (20.2 vs 3.8) and popped Flux-2 down to 4.0 at act 91;
it wins purely on 28 dd banks (acts 48–73: +18.0, +10.0, +28.5, +21.2, +33.0, +16.7...).
`d5408f1f` 1734–1527 (v3 Blue): flip at act 5, 18 dd, farmed 11/18. **-> A** (H6 blue tempo)
with B secondary on dd pricing.

### 5.7 Scout, bowot, Flux, Roxbot, small baby

`83f11a1e` Scout 1275–1136 (v2 Blue): §4.5; we even led 633–582 at act 51. `abb62022` Scout
1513–1444 (v1 Blue): close; peak +288@49, flip 101. **-> B** (H2/H7 shape).
`f9c819ed` bowot 1663–1658 (v3 Blue): §4.4; tied at 111 actions; re-close 16970 x4 (33, 68,
84, 113) feeds poppers 6198, 4807, 5096 x2, 4032, 1580, 10303, 497 x2, 15223. `20bf5fac`
bowot 2203–1548 (v3 Red): never ahead after act 11; bowot steady 34–52. `a5ab11d0` bowot
1666–1374 (v1 Blue): bowot first-closes at act 18 (!) yet banks smoothly; every close of ours
from [41] popped within 2 (9983, 555, 500, 15414, 12354, 15603, 440, 9827, 10244). **-> B (H4).**
`4bf39c48` Flux 2642–2641 (v2 Red): §4.4; our +47.8 [59] popped by 11922 [60] in one action,
+22.5 [51] by 2308; Flux's loop 35 @41 -> 68.5 @81 held to the end. **-> B (H4).**
Roxbot: `abaea489` 1982–1340 (v1 Blue, flip 13, 17 dd), `f0d17dcc` 2000–1241 (v4 Blue, flip 13,
farmed 20/31), `95d14ddf` 3014–2117 (v4 Red, flip 55, Roxbot 54–73 area by act 31). **-> B (H1).**
small baby: `69f08058` 1779–1129 (v2 Blue, **flip 5**, enemy 19.5 vs our 9.0 at act 11),
`ab1b58f5` 1354–869 (v2 Red, flip 11). **-> A (H6).**
VladNet v2 cluster (5bbee7e7, fa1df9b9, 200389aa, 80ea1b0e): textbook farm cycle — our area
oscillates 0–60 (fa1df9b9: 59.8 -> 9.0 -> 42.9 -> 4.5 -> 49.9 -> 0.0 -> 65.2 -> 0.0) while
VladNet sits 32–51; flips 35–61. **-> B (H1, already catalog #3/#4).**

## 6. Ranked hypotheses (one per loss group; addressee first, confirm target second)

| # | Losses (ids) | Divergence | HYPOTHESIS | Lane: test / confirm target |
|---|---|---|---|---|
| H1 | capybara 4x; VladNet v2 x4, v3 x2; bowot x3; Roxbot x3; Scout 2x; small baby (red) | flips 11–79; 61% of our >= +4.5 closes popped within 3 actions; pop ids reused across 20–40-action spans (17112 x4, 12114 x3, 2707 x3, 452 x3) — far beyond CUT_MEMORY 6 | Farm-cycle memory + close-survival pricing are both too small on site: the avoid set forgets cycles that run for dozens of actions, and re-closes on known farm ground still outrank fresh ground (catalog #3/#4, now confirmed in rated play at 20–40-action cycle lengths) | **B**: extend farm memory (per-edge cut counters with decay >= 30 actions or no decay) + scale the rebuild penalty to the candidate's own gain; gate with the avoid-reconstructing harness (catalog §3.6). Confirm: at 446956a1 act 51 (re-close 16013) the counter must make any fresh-ground move outrank it; at f9c819ed act 113 (re-close 16970 x4) same |
| H2 | AngelBot 5x; 4929b951; Scout 83f11a1e; all 53 (47% average) | flips 1–19 vs AngelBot; 706/1515 cuts pop nothing (Scout game 41/41) | Wasted-cut pricing: the break bonus pays for cuts that unbank zero enemy area — invincible thickets (a) and ALREADY-BANKED ground (b), the bank-sacrifice trade AngelBot weaponizes (cutting banked loops recovers nothing) | **B**: weight the break bonus by enemy area actually unbaked (rival-analysis steal #3), i.e. cut value = enemy area lost that is NOT yet banked; thicket detection per V4d2 legality. Confirm: 1c7b1aa1 acts 43–50 must rank the demolitions near zero; 83f11a1e the 41 bounces must rank below any gaining move |
| H3 | b64a562b (+42 @80, lost by 520); f9c819ed (tied @111, −5); 4bf39c48 (+416 @81, lost by 1) | flips 79/95/117 — all late; enemy living area 36–69 stable while we farm 5–13 re-closes | Endgame bank-race blindness: when events_left is small and enemy living area > ours, fresh small closes are worth ~nothing; the game is denial (cut/contest enemy living ground) — findings-gamedata P0-3 confirmed in bot play | **B**: endgame term — when events_left <= ~10, price enemy living area x remaining events as the threat it is and boost denial moves. Confirm: b64a562b act 80 position must prefer attacking VladNet's 36-area web over the +4.5 re-close it actually played; 4bf39c48 act 81 same vs Flux's loop |
| H4 | xmybot 2x; GB 19x; GB2.0 2x; Atlas v2; Flux 4bf39c48 | eval peaks +1413/+2397/+1099/+659–923 6–20 actions before a 65–256-area enemy close; crashes to −5031/−881 after | Enemy closable-potential is invisible: room is priced 0.4x once (no horizon, no closure progress); an enemy that banks ~0 area for 40 actions while drawing a giant loop reads as "we are crushing" | **B**: enemy web-closure term — estimate the area the enemy's open web is about to enclose (room x closure-progress, or their best close's gain via one movegen scan like max_pop but for THEIR close) and price it x min(events_left, HORIZON); plus routing: when enemy room >> ours, march nodes into their hull early. Confirm: e6bf571b act 72 (+2397) must drop to <= 0; a3f5f01c act 54 (+1413) same; 6c4c2a2c act 46 (+1099) same. **A**: confirm at e6bf571b act 72 that depth-3 still sees nothing (the closing move is 8+ actions out) — if so, this is B's term alone |
| H5 | AngelBot 1c7b1aa1 (+37.8 demolition [50] while score gap widens) | the pop lands, the bank doesn't move | Bank-aware cut valuation is part of H2; listed separately for A's depth check: does any feasible depth fix the tempo trade? | **A**: at 1c7b1aa1 pre-[50], run depth-3: if the demolition still wins locally (it banks a graze), the fix is B's H2 pricing, not depth |
| H6 | db7641e1, d5408f1f, 69f08058 (all flip @ act 5); abaea489, f0d17dcc (flip 13); a5ab11d0 (flip 29) | as Blue vs fast openers we are behind on area by act 11 (9.0 vs 19.5) and never recover; red ladder is stable in every game | Blue opening tempo hole: 1 forced action + site-chosen opener (never D10-F7, catalog §0) vs rivals that reach 19–33 area by act 11 | **A**: fix the probe_bluechair opener-response no-op (§0: inject only when to_move()==Blue) and test whether a D10-F7-class response or an area-first blue book recovers the act-11 deficit; replay db7641e1/69f08058 openings as the harness positions |
| H7 | Scout 2x (43–14 out-closed, 41/41 cuts bounced, 28 dd) | flip 45/101, mild evals | Durability shape gap vs the builtin's thicket: we have DENSE_BONUS but Scout's closes bank more per move and its region is uncuttable | **B**: boundary-density requirement on closes (2-touch edges, catalog #2's per-close survival scan) — Scout's shape is the reference; beat Scout 6+/8 before any rival work |

Priority call: **H4 (blob) and H2 (wasted cuts) are the new money** — H4 lost us the two biggest
score deficits on record (−4949, −819 with a winning bank) and 21 of 53 games; H2 is 47% of our
cut tempo. H1/H3 extend catalog #3/#4 with site evidence. H6 is the only A-owned item with a
cheap experiment attached.

## 7. Caveats, artifacts, and handoffs

- Eval trajectories are the CURRENT engine's view; they are exact for v4 losses (59–60/60
  deployed-move matches) and approximate for v3/v2/v1 (36–48/60). The 25/53 "flip at 119" games
  include garbage-time positives — treat flip acts 119 as "never durably ahead".
- Survival was also measured directly: median answer time of a >= +4.5 close is 3 actions for
  BOTH sides — the asymmetry is not answer speed but what the answer leaves behind (their refill
  is durable, ours re-inflates): mean area 30.7 vs 49.8, peak 62.6 vs 95.0.
- Master-table numbers are from the replay harness at /tmp/opencode/sitegames (game JSONs
  `losses/`, per-action logs `an/`, autopsy transcripts `autopsies/`, analyzer crate
  `analyzer/`). Nothing in the repo was touched except this file.
- **Lane E (FYI):** 22 games stuck `waiting` in the window vs Spearman (2), Kinetic (4),
  john.fun (4), Space Bunny (10+) — opponents never came online; plus unfinished VladNet v3
  pair (949c4aae, e17f71c9) and AngelBot bbd9da1e (1615–402 in progress at fetch time).
- Ratings snapshot (site): Scout 1315, Riposte(v1) 1460, v2 1370, v3 1489, v4 1402 — v4 is
  NOT stronger than v3 on site; the capybara sweep (4–0 vs v4) and the Scout split (2–2 vs v2)
  both landed on the newest deployment.
