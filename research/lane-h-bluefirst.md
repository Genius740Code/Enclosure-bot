# Lane H — Blue first-move sweep (forced-action action-1 candidates), 2026-09-28

Branch `lane-h-opener` (from `master` @ `eb100e9`, V5-1 baseline). Probe-only: no
`retaliator/src/*` changes. New instrument: `retaliator/examples/probe_bluefirst.rs` (runs:
`cargo run --release --example probe_bluefirst -- v3` / `-- v1`; ~11 min each, 45 games each).

## 0. Question and method

**Question (c-blunder-catalog §0 + c-site-losses-2 §4.3 H6).** The site chooses Blue's engine
action 1 in rated play — our observed first moves are 7030 (A10-C9), 4503 (A10-C8), 5228
(D10-A9), never D10-F7 — so `blue_opener` (D10-F7, +20-41% vs v1 in `open=None` games) never
fires on site. The prior opener-response experiment (`probe_bluechair`) was a NO-OP: it checked
D10-F7 on Red's turn, so the move was always illegal (catalog §0). This probe is the fixed
version and asks: **which first action maximizes Blue's margin vs v1 on the site's forced
openings, and does any candidate fix the early flip signature?**

Design (`probe_bluefirst.rs`):
- Openings = probe_v3all's list: `None` (empty start), 4864 (A10-D8), 5589 (D10-B9),
  11723 (A10-C11), 9199 (D10-F10).
- On a forced opening: the site's choice plays as engine action 1, Red's reply [2,3] via the
  opponent, then the candidate is **forced as Blue's first own move** (engine action 4, where
  `to_move() == Blue`) — exactly catalog §0's one-line fix. On the empty start the candidate
  replaces `blue_opener` outright (engine action 1).
- Then full games with the shipped `search::best_move` for both sides. Matchups: search-vs-search
  (v3-v3) and search-vs-v1. Per opening: 2 no-force baseline rows (both colors) + 7 candidate
  rows (our side Blue). Per candidate per matchup: 5 openings = 10 Blue-side games (>= 8), plus
  the red-side baseline rows (both colors) — n=10-20 per claim.
- Eval trace: shipped `search::evaluate` (the V5-1 brain), Blue-perspective, after every engine
  action <= 12; `flip` = last OUR turn-end engine action with eval >= 0 (the c2 instrument;
  119 = garbage-time). Numbering below is ENGINE actions (1..120; B[1] solo, R[2,3], B[4,5]...).
- Candidate set (justified from the catalog):
  - **D10-F7 (1979)** — the shipped `blue_opener` (catalog §0; the control).
  - **D10-A7 (174)** — the walk-backward the search picks on direction-index ties (search.rs:54);
    isolates the forward-vs-backward diagonal choice.
  - **D10-G7 (2340), D10-G13 (17141), D10-G10 (9560)** — longest-reach moves from the
    center-facing endpoint toward the middle (H6's "area-first blue book": fast openers reach
    19-33 area by act 11 while we sit at 9.0; G10 = max-room extreme, straight; G7/G13 = long
    diagonals).
  - **A10-C9 (7030), A10-C8 (4503)** — the site's observed rated action-1s (catalog §0), short
    contact-ish develops from the back endpoint; anchors the sweep to rated reality.

## 1. Validation (all passed)

- Candidate id decode (probe prints its own indices): 1979=D10-F7, 174=D10-A7, 2340=D10-G7,
  17141=D10-G13, 9560=D10-G10, 7030=A10-C9, 4503=A10-C8 — 1979/7030/4503/9560 match the known
  site ids from probe_blueopen and catalog §0.
- **All 10 v1-matchup baseline rows reproduce `probe_v3all` byte-for-byte**: finals 1322-786,
  1109-865, 1243-1237, 1131-1218, 1256-1745, 1218-1219, 1637-1806, 1343-1259, 1415-2120,
  1541-1578; As-Blue 2-3 (W None+4864) and As-Red 2-3 (W None+11723) exactly as the scoreboard.
- **Empty start, no-force baseline ≡ forcing D10-F7 explicitly: byte-identical 2073-1294 (v3-v3)
  and 1322-786 (v1)** — `blue_opener` is exactly a forced action-1; the harness's forcing path
  is the same code path the opener uses.
- **Two invalid rows (forcing fall-throughs), excluded from the rankings**: 5589/A10-C8 and
  9199/D10-G10 are ILLEGAL at engine action 4 — `check_move` rejects A10-C8 with
  PathThroughOwnNode (the path (-9,0)→(-7,-2) passes through B9, created by the opening
  D10-B9) and D10-G10 likewise through F10 (created by 9199=D10-F10). Both rows are
  byte-identical to the no-force baseline, i.e. the catalog's no-op failure mode again —
  any future forced-move probe must assert the injection happened.

## 2. Results — ranked action-1 tables (side=blue; margin = (ours-theirs)/ours; base = no-force baseline)

### vs v1 (search-vs-v1)

| opening | ranked candidates (best first) | base |
|---|---|---|
| none | **D10-F7 +40.5% W** \| A10-C8 +22.2% W \| D10-A7 +1.7% W \| A10-C9 −14.0% \| D10-G7 −22.0% \| D10-G10 −28.9% \| D10-G13 −41.6% | +40.5% W (≡ D10-F7) |
| 4864 | **A10-C9 +6.9% W** \| D10-A7 +0.5% W (≡ base) \| D10-G7 −1.7% \| D10-G10 −16.3% \| A10-C8 −22.9% \| D10-G13 −25.3% \| **D10-F7 −41.2% (our area 0.3!)** | +0.5% W |
| 5589 | **D10-G7 +17.2% W** \| D10-A7 −13.9% \| D10-F7 −30.6% \| D10-G10 −35.4% \| D10-G13 −36.6% \| [A10-C8 illegal@4] | −39.0% L |
| 11723 | **D10-A7 −10.3% (≡ base, best; no candidate beats it)** \| D10-G10 −10.9% \| D10-G13 −21.2% \| D10-F7 −25.5% \| D10-G7 −35.8% \| A10-C9 −65.3% \| A10-C8 −67.3% | −10.3% L |
| 9199 | **D10-F7 +41.2% W** \| D10-G7 −5.8% \| A10-C8 −8.0% \| D10-A7 −14.9% \| A10-C9 −37.4% \| [D10-G10 illegal@4] | −49.8% L |

### vs v3 (search-vs-search)

| opening | ranked candidates (best first) | base |
|---|---|---|
| none | **D10-F7 +37.6% W** \| A10-C8 +11.0% W \| D10-A7 −8.4% \| A10-C9 −13.0% \| D10-G7 −15.8% \| D10-G10 −22.4% \| D10-G13 −60.9% | +37.6% W (≡ D10-F7) |
| 4864 | **D10-G10 +8.8% W** \| A10-C9 +6.9% W \| D10-G7 +4.3% W \| A10-C8 −3.8% \| D10-G13 −25.5% \| D10-A7 −31.9% (≡ base) \| **D10-F7 −49.1%** | −31.9% L |
| 5589 | **A10-C9 +37.4% W** \| D10-F7 +17.2% W \| D10-G7 −28.3% \| D10-G10 −34.1% \| A10-C8 −36.4% \| D10-G13 −38.8% \| D10-A7 −50.9% | −36.4% L |
| 11723 | **D10-F7 −6.3% (best)** \| D10-A7 −22.2% (≡ base) \| D10-G10 −34.2% \| D10-G13 −35.5% \| A10-C9 −52.2% \| A10-C8 −62.3% \| D10-G7 −78.0% | −22.2% L |
| 9199 | **D10-A7 +38.1% W** \| D10-F7 +32.4% W \| D10-G7 +25.3% W \| D10-G13 −9.7% \| D10-G10 −12.1% \| A10-C8 −32.3% \| A10-C9 −62.7% | −12.1% L |

(v3-v3 red-side rows are exact mirrors of the blue-side rows — same deterministic search both
sides — so they add no information there; the v3-vs-v1 red-side rows are genuine and match the
scoreboard, see §1.)

### Aggregates over the 4 forced openings × 2 matchups = 8 games per candidate (n=8)

| action-1 | W-L | mean margin | worst | wins |
|---|---|---|---|---|
| **D10-F7** | **3/8** | **−7.7%** | −49.1% | 5589-v3 +17.2, 9199-v1 +41.2, 9199-v3 +32.4 |
| D10-G7 | 3/8 | −12.9% | −78.0% | 5589-v1 +17.2, 4864-v3 +4.3, 9199-v3 +25.3 |
| D10-A7 | 2/8 | −13.1% | −50.9% | 4864-v1 +0.5, 9199-v3 +38.1 |
| D10-G10 | 1/6 | −20.4% | −35.4% | 4864-v3 +8.8 (9199 illegal) |
| base (search's own choice) | 1/8 | −25.2% | −49.8% | 4864-v1 +0.5 |
| A10-C9 | 3/8 | −26.5% | −65.3% | 4864-v1/v3, 5589-v3 |
| D10-G13 | 0/8 | −32.3% | −65.5% | — |
| A10-C8 | 0/6 | −32.8% | −67.3% | — (5589 illegal) |

vs v1 only (the task's metric, 4 forced openings): best mean is **D10-G7 −6.5%** (1/4, only
5589 W), then D10-A7 −9.7% (1/4), D10-F7 −14.0% (1/4), base −24.5% (1/4) — **every candidate's
mean is still negative**; D10-F7 has the best single margin (+41.2% on 9199).

## 3. The act-5..10 flip signature: NO candidate fixes it — the eval is an artifact, not a driver

- **The eval at actions 5-10 cannot discriminate candidates that diverge later.** On 9199/v1,
  D10-F7, D10-G7 and D10-G13 have IDENTICAL evals at every engine action 5..10
  (−26/−25/−88/−86/+28/+28 — the search's eng-5..10 replies converge across them) yet the
  outcomes are +41.2% W / −5.8% L / −65.5% L. The discriminator lives beyond action 12.
- **Where the eval does differ, it is INVERTED w.r.t. outcome.** 9199/v1: the base has the best
  early eval of any row (E5=+36, E9=+77) and the WORST final (−49.8% — its early banks get
  farmed, catalog loss 9199's exact story); the winner D10-F7 has E5=−26. 4864/v1: the winners
  have the worst evals (base −6@5, A10-C9 −28@5) and the losers the best (D10-F7 +2@5, D10-G7
  +16@5, D10-G10 +74@9 → all L). Empty start: D10-G10 has the biggest early eval of any
  candidate (E9=+121 vs base +62) and still loses (−28.9% v1, −22.4% v3).
- **So H6's "area-first blue book" is REJECTED**: every long diagonal toward center loses from
  the empty start (D10-G7 −22.0/−15.8, D10-G10 −28.9/−22.4, D10-G13 −41.6/−60.9) despite
  maximizing early area/room — the eval's area×12 + room credit rewards exactly the shapes that
  get farmed. This is catalog #2 (close-survival blindness) and H1 confirmed from the opening
  phase, one phase earlier than any rated sighting.
- The catalog §1.2 conclusion generalizes: first-close timing and the act-5..10 eval are NOT the
  discriminator. The None game is the only one whose eval is durably positive from act 5
  (base/D10-F7: +56@5 → +62@9, FLIP=117) — and it is the only empty-start game, won by the
  opener, not by the eval.

**Interaction finding (new, for lane A/B): D10-F7's value depends entirely on F10 being on the
board.** On 9199 (the site plays D10-F10 at action 1) the F7 diagonal has a second anchor
(F10-F7 vertical 3-gap → 2-touch triangles), and forcing D10-F7 at eng 4 wins BOTH matchups
(+41.2% / +32.4%). On 4864 (A10-D8, no anchor) the same move is the WORST candidate (−41.2% /
−49.1%, our final area 0.3-6.9 vs 15.7 base — the area-lifetime-asymmetry fingerprint). The
shipped opener is right when it fires and harmful when the site's action 1 leaves F10 off the
board — but in rated play it never fires at all, so this is unexercised either way.

Also: the search's unforced eng-4 pick IS D10-A7 on 4864 and 11723 (candidate rows byte-identical
to base, both legal) — the walk-backward is the natural first own move there, and it is the best
candidate on 11723 (−10.3%) and 2nd on 4864 (+0.5% vs v1).

## 4. Recommendation (ONE)

**No action-1 change fixes the site losses — the problem is later. Keep `blue_opener` = D10-F7;
do NOT adopt an area-first book; do NOT force D10-F7 at the first own move.**

- **Keep `blue_opener` (D10-F7).** From the empty start it is the best action-1 vs both v3
  (+37.6%) and v1 (+40.5%), worth +18 to +46 points of margin vs every alternative
  (A10-C8 +22.2/+11.0 second; D10-A7 +1.7/−8.4; long diagonals all negative). The catalog's
  premise that the unexercised opener is a lost advantage is FALSE in the other direction too:
  no tested alternative beats it, so there is nothing to replace it with.
- **No first-own-move candidate fixes the forced openings.** The per-opening best is a different
  move every time (4864 → A10-C9; 5589 → D10-G7; 11723 → base/D10-A7; 9199 → D10-F7), no
  candidate wins both matchups on more than one opening, and every candidate's mean margin vs v1
  over the 4 forced openings is negative (best −6.5%). The best aggregate (D10-F7@4: 3/8, mean
  −7.7%) is a high-variance gamble driven by one opening (9199) with a −49.1% worst case — not
  deployable. 11723 is unfixable at action 4: the base is already the best (all 7 candidates
  lose, base −10.3%).
- **The act-5..10 eval is an artifact, not a driver** — identical across candidates that diverge
  later (9199: F7/G7/G13 → +41.2/−5.8/−65.5) and inverted where it differs (9199 base +77@9 →
  −49.8; 4864 winners have the worst evals). H6's question is answered NEGATIVE: neither a
  D10-F7-class response nor an area-first book recovers the act-11 deficit durably.
- **Hand-off to lane B (primary): close-survival / farm pricing (catalog #2/H1), now confirmed
  from the opening phase.** Clean confirm target: the 9199/v1 base position at engine action 9
  (eval +77, the best early eval in the sweep, final −49.8 with our area farmed 35.4→recyclable)
  — a survival term must make that position read <= 0, and the 9199 D10-G7/G13 rows (identical
  5-10 eval, −5.8/−65.5 finals) show the discriminator is beyond action 12, i.e. horizon-depth
  (lane A) cannot see it either.
