# Lane C — Blunder catalog (autopsy of problems #1 and #2), 2026-09-27

Branch `lane-c-autopsy`; first run at `c63b15f` (probes imported from `lane-a-search`),
code base `eb100e9` (V5-1 baseline), verified byte-identical on re-run 2026-09-28.
Analysis only — no `retaliator/src/*` changes. Every claim below cites probe
action numbers `[n]` and measured evals from this run.

## 0. Method, numbering, and two instrument facts

- `probe_bluechair` (v3 as Blue vs v1) reproduced the scoreboard baseline **byte-for-byte**:
  finals 1322-786 (None), 1243-1237 (4864), 1256-1745 (5589), 1637-1806 (11723), 1415-2120 (9199).
- `probe_balloon` reproduced the collapse: skip=10 → 2177-1031 (−111.2%), skip=20 → 2177-1031,
  skip=30 → 1553-735 (−111.3%).
- **Numbering:** the engine starts Blue with edge A10-D10 and Red with P10-S10 (`position.rs`
  `Position::new`), Blue plays solo action 1, then 2-action turns Red-first (B[1]; R[2,3]; B[4,5]…;
  `mover_after`, `position.rs:597`). In forced-opening games the probe plays the opening outside
  its log, so **probe-n = engine action − 1** there. All action numbers below are probe-n.
- **The site forces Blue's action 1 (high confidence).** GB plays a byte-identical 26-move book
  in every tracked game yet its *first* action varies (D10-F11 / D10-C7 / A10-C11,
  rival-analysis §4) — a fixed book can only vary at action 1 if the site chooses it. Our
  deployed "Riposte v3" likewise played 5228 in `4f0c9650` and 4503 in `3910c73c` (same bot,
  same scheduled instant). Consistent with that: in every observed rated game our first move is
  **not** D10-F7 (7030, 4503, 5228), so `blue_opener`'s measured +20–41% advantage is never
  exercised on site; it exists only in local `open=None` games.
- **The opener-response experiment in `probe_bluechair` was a NO-OP in the
  first run** (it checked `D10-F7` legality on Red's turn, where the move is
  always illegal). **Fixed and re-run 2026-09-28** — Red's first turn free
  (v1), then D10-F7 forced as our first real action (engine act 3): results
  in §1.6. Side note from the fix: with no forced opening, v1's own search
  picks D10-A7 (backward diagonal) as its solo opener, exactly as
  `search.rs`'s comment predicts.

## 1. Problem #1 — Blue chair: v3 as Blue vs v1 on the site's forced openings

### 1.1 Summary (this run; final areas are enclosed area at game end)

| opening (probe-n / engine) | final B-R | Blue 1st close | Red 1st close | B closes/cuts | R closes/cuts | B final area | R final area |
|---|---|---|---|---|---|---|---|
| None | **1322-786 W** | act 5 (+9.0) | act 3 (+4.5) | 23 / 43 | 18 / 41 | 12.2 | 11.2 |
| 4864 = A10-D8 | **1243-1237 W** | act 4 (+2.7) [eng 5] | act 2 (+4.5) [eng 3] | 25 / 38 | 18 / 36 | 15.7 | 11.0 |
| 5589 = D10-B9 | **1256-1745 L** | act 4 (+4.5) | act 2 (+4.5) | 20 / 37 | 28 / 39 | 19.2 | 19.8 |
| 11723 = A10-C11 | **1637-1806 L** | act 4 (+4.5) | act 2 (+4.5) | 30 / 36 | 18 / 40 | 13.2 | 27.7 |
| 9199 = D10-F10 | **1415-2120 L** | act 4 (+7.5) | act 2 (+4.5) | 27 / 35 | 14 / 41 | 35.4 | 32.9 |

### 1.2 Ruled out, with numbers (applies to all three losses)

- **First-close timing is NOT the discriminator.** In engine actions, Red first-closes at
  action 3 (+4.5) and we first-close at action 5 (+2.7…+9.0) in **all five games** — identical
  timing in 2 wins and 3 losses.
- **Patience never fires.** Every observed first close is +2.7…+9.0, all above
  `PATIENCE_MAX_GAIN = 2.0`, so `PATIENCE_PENALTY` is applied to nothing in these lines. This is
  the mechanism behind the V5-1 ablation's byte-identical result (scoreboard 2026-09-27). The
  term is dead code at threshold 2.0 — not "harmless", unpriced.
- **Cut volume is NOT the discriminator.** We suffered 41 cuts in the *None win* and 36 in the
  4864 win, vs 39/40/41 in the losses. What differs is what the cuts pop and who holds durable
  ground while farming (see per-loss findings).
- Our first searched action sits at eval −64…−66 in every game (Red's +4.5 close at engine act 3
  priced ×12). The None game recovers to positive by act 11 and stays there; the losses only
  spike positive briefly before their divergence windows (11723 +10 at [27], 9199 +51 at [28]).

### 1.3 Loss 5589 (D10-B9) — 1256-1745

**Shape of the game.** Forced D10-B9; we answer A10-A13 [3] and close D10-A13 +4.5 [4], then
build one long west/center wall (A10-A13-D13-G10-G7-D8-J7-G4-L8…, acts 7–36). Red runs its
standard corner web (P7-P10 [2], M7-P7 [6], P4-M7 [10], P4-S4 [14], P1-P4 [18], M1-P1 [22],
P10-P13 [26], M7-M10 [30], M7-J10 [34]) — 45.0 area by act 34, ours 40.3. Score gap is small and
stable: −13 (act 10), −18 (20), −17 (30), −28 (40).

**Divergence.** Two-stage:
1. Acts [37]–[46]: Red dismantles our wall with six cuts (J10-H8 [37] cuts J7-G10; H8-E8 [38]
   cuts G7-G10; J10-H7 [41] cuts G7-J7; H7-E9 [42] cuts D8-G10; H8-F10 [45] cuts D10-G10;
   E8-G11 [46] cuts G10-D13 — our area 44.8 → 14.8). Our O-file counter-raid [39]–[44] cuts
   Red's M-web but reclaims **zero** area (ours falls 35.8 → 26.6 on Red's [41]/[42] and never
   recovers). Eval before our moves: −94 [36] → −162 [39] → −192 [47].
2. Act [54]: Red closes M1-K4 **+13.8** — the first of the center-swallow cut-close combos. Eval
   flips terminal: −205 at [51] → **−428 at [55]**; score gap −49 (act 50) → −109 (60) → −192
   (70) → −290 (80) → −489 (final).

**The decisive mechanism (measured):** Red's acts 54–118 closes bank ~470 area, and at least ten
of them are **double-duty** (big close *and* a cut of one of our edges in the same move):
[58] K4-H7 +31.5 cutting our G4-J7; [65] M10-P13 +46.5 cutting O11-P14; [73] S4-P7 +43.5 cutting
O6-R6; [78] K4-H7 +57.0 cutting G4-J5; [85] P4-M7 +30.0 cutting M4-P5; [91] M1-K4 +48.0 cutting
J2-M4; [94] S4-P7 +26.6; [101] M1-K4 +48.0; [102] S1-S4 +18.0; [109] S4-P7 +18.0. Our closes in
the same window bank ~130, almost all single-duty. Red ends 19.8 area *alive since act 2*; our
19.2 area oscillated through zero — Red's score integral wins 1745-1256 on near-equal final area.
Sub-plot confirming rebuild-farming: we close **D8-E11 three times** ([47] +12.2, [80] +7.7,
[96] +7.7, all move id 16381) and Red re-cuts it twice with the **same move id 8098**
(E9-C9, [61] and [90]).

**RANKED HYPOTHESIS (the one): unpriced enemy close-ground — our cut wall releases Red's
double-duty closes, and no term sees it.** The doom discount only prices enemy pops of *banked*
area; it prices nothing for ground our exposed edges are *shielding from the enemy's next big
close*. → **B: test an enemy-release term** — for each of our exposed edges, charge
(enemy's best single-action area gain if it is cut) or equivalently extend the doom scan to
enemy max area-gain, not just our max area-loss. Confirm target: at the [55] position the term
should make holding/reinforcing G4-J7 (or contesting K4/M1) outrank the D8-E11 re-close.
→ **A (secondary): depth check** — at the position before [55], depth-3 (our pair + Red's pair)
sees [58] K4-H7 (+31.5 & cut) at ply 3; 2-ply cannot because Red never appears in a turn-start
PV (the reply ply is our own second action). If depth-3 holds > −49 through [58], hand the fix
to A; if not, it's B's term.

### 1.4 Loss 11723 (A10-C11) — 1637-1806

**Shape of the game.** We close small early (+4.5 ×4 by act 16, +9.0 [24], +4.5s through 40),
reach score parity at act 40 (**+4**: 517-513) with area 52.3 vs 45.0. Then the balloon phase:
each of our big closes hangs on a **stale boundary edge placed 8–35 actions earlier**, and Red
pops it with a single Extend within two actions:

| our close (act, gain) | boundary edge (created) | Red's pop (act, our loss) |
|---|---|---|
| [47] G6-I6 +32.8 | D7-G10 (act 12) | [49] F7-E8: **−37.3** |
| [51] E9-H11 +36.2 (self-cross) | D1-A4 (act 40) | [53] B2-B5: −36.3 |
| [55] D1-D4 +27.2 | D1-G3 (act 36) | [57] E2-E1: −27.3 |
| [59] I6-K8 +17.5 | K8-H11 (act 56) | [61] M10-J8: −17.5 |
| [67] D1-G3 +23.8 | D1-D4 (act 55) | [69] B1-E2: −23.8 |
| [76] F1-I4 +23.8 | F1-D4 (act 76) | [77]+[78]: −40.2 |
| [88] F1-D4 +16.3 | F1-D4 | [93]+[94]: −29.8 |

**Divergence action: [49]** — one Red Extend (F7-E8) erases the loop we closed at [47]; eval
**+207 at [48] → −288 at [51]** (a −495 swing on one action). Eval never durably recovers
(min −1045 at [79]); the *score* grinds −11 (50) → −41 (70) → −81 (100) → −169 (final). We die
with **more closes than Red (30 vs 18)**: our closes net ≈ 0 area-lifetime, Red's 18 closes
hold 25–45 area continuously from act 30 to the end despite our 36 cuts.

**RANKED HYPOTHESIS (the one): close-survival blindness — the eval credits a doomed close at
full area×12 and the doom scalar can't discriminate (every candidate leaves some grazeable
loop), so the bot re-inflates balloons on stale single cuttable edges.**
→ **B: test per-close survival pricing** — before crediting gain G for a Connect, scan the
closed region's boundary for *legal* one-move enemy cuts (per V4d2's legality rule, via
`check_move`-style test per boundary edge); if any exists, cap the credit at the 1–2 turns the
region actually survives (or subtract the enemy's best legal boundary cut value). Confirm
target: at [47], close G6-I6 should be priced ≈ +32.8×1–2 turns, not +32.8×12, and a
routing/densifying alternative should outrank it.
→ **A (secondary):** at the [47] position, run depth-3: if it still picks G6-I6 (because the
close banks once before the pop and that is genuinely +EV in score terms), the fix is B-side
*boundary density* (make closes land on 2-touch edges), not avoidance — depth won't save a
structure that is profitable to inflate and pop.

### 1.5 Loss 9199 (D10-F10) — 1415-2120

**Shape of the game.** Our best early game of the five: first close +7.5 [4], eval positive
[11]–[36] (peak **+51 at [28]**), score **+38 at act 30** (330-292) and +51 at 40 (546-495).
Then Red runs a 9-cut farming blitz in 17 actions: [34] J7-G9 cuts F7-I10; [37] G9-E8 cuts
F7-F10; [38] E8-B9 cuts C7-F10 (our area 44.8 → 32.8); [41] B9-A6 cuts C7-A10; [42] E8-B11 cuts
A10-D10; [45] A6-D3 cuts C3-E6; [46] J7-G8 cuts F7-I10 again; [49] B11-A14 cuts A10-C13;
[50] B9-D12 cuts F10-C13 (a loop held since act 8).

**Divergence actions: [34]–[38]**; eval **+25 at [35] → −75 at [39]**, never positive again
(−278 by [51], −1184 min at [83]); score flips +51 (40) → +5 (50) → −60 (60) → −705 (final).

**The farm chain, exact move ids:** Red cut our F7-I10 at [34] (id 12397); we **re-closed**
F7-I10 at [40] (+16.5, id 17086 — simultaneously "cutting" Red's J7-G9, an edge that shields
**zero** Red area, so our cut bonus was paid for nothing); Red re-cut at [46] (id 9870).
Red cut C7-A10 at [41] (id 875); we re-closed it at [47] (+16.5); Red re-cut at [57] (id 5191).
Red suffered 35 of our cuts but its area never fell below 32.9 all game; our area oscillated
2.7–53.8. Red ends 32.9 area on **14 closes** vs our 35.4 on 27 closes — pure area-lifetime
asymmetry.

**RANKED HYPOTHESIS (the one): rebuild-farming susceptibility — the anti-rebuild routing is
absent in every probe (see §3.6) and numerically too weak in the deployed path.**
`REBUILD_PENALTY 3.0 × hz 12 = 36` cannot flip any re-close it applies to: a +4.5 re-close is
worth 54, the F7-I10 re-close 198. → **B: test scaling the rebuild penalty to the candidate's
own gain on cut ground (penalty ≥ own_gain × hz within CUT_RADIUS), plus weight our break bonus
by the enemy area it actually un banks (rival-analysis steal #3) — the [40] move shows both
sides of the bug in one action (worthless cut paid a bonus + farmed re-close chosen).**
→ **A (secondary):** at the [40] position, depth-3 sees Red's J7-G8 re-cut at ply 3; if the
re-close still wins on banked-once EV, the answer is B's routing penalty, not depth.

### 1.6 The opener-response question, ANSWERED (2026-09-28, fixed probe)

D10-F7 forced as our first real action (engine act 3) after each site opening,
Red's first turn free. Baseline = the main games above (v3's search responds freely).

| opening | baseline | +D10-F7@act3 | change |
|---|---|---|---|
| None | **1322-786 W** | **1322-786 W** | same |
| 4864 = A10-D8 | **1243-1237 W** | **1019-1438 L** | **flipped to loss** |
| 5589 = D10-B9 | 1256-1745 L | 1038-1356 L | margin −489 → −318 |
| 11723 = A10-C11 | 1637-1806 L | 916-1150 L | margin −169 → −234 |
| 9199 = D10-F10 | 1415-2120 L | **1053-619 W** | **flipped to win** |

**Finding: first-response placement is a first-order factor, but no fixed response
dominates.** Forcing the measured opener flips 2 of 5 games in OPPOSITE directions
(9199's farm-fest becomes a win: the baseline's A10-C7 response at [3] invites the
34–50 cut blitz; 4864's won game becomes a loss). Margins swing ±300–700 points on
one move choice. The win/loss is decided by the interaction between our first
response and Red's first-turn reply — i.e. the search's action-1/act-3 choice is
unstable at the scale that decides rated games. This is consistent with
hypothesis #1/§1.3 (enemy close-ground release): different first responses expose
different edges to Red's first cut-close combo.
→ **For Lane A/B:** the first response is worth MORE search (plan-v3 §2.4 time
management) or a stability check (easy-move fast path must not fire at act 1–3);
a single fixed response is NOT the answer (measured mixed, n=1 per line, deterministic).

## 2. Problem #2 — Red collapse (`probe_balloon`, ret as Red vs scoutbase after skip pre-moves)

### 2.1 The single 30+ pop is GONE — the collapse is now a farm cycle

- `maxEnemyPop = 0.0` at **every** sampled our-turn t=41…89 in **all three** lines. No break
  exceeded −14.2 anywhere in the big-break log (drops: −4.5 ×many, −5.2, −5.8, −5.9, −6.0,
  −6.8, −7.5, −8.4, −11.2, −13.5, −14.2). The plan-v4 "single 30+ pop ~t=60" no longer exists on
  this build — the V4 doom discount (DOOM_W 1.0) did its job. **But the margin is unchanged
  (−111%)** because the loss mechanism moved to continuous erosion + re-close farming.
- skip=30 is the purest case: our area is **0.0 before our turn at t=41, 57, 61, 65, 69**; we
  re-close the
  same M1-P1 / P1-S4 region for +13.5 at t=41, 45, 49, 57, 65, 73, 85 and scout re-pops it with
  **identical move ids**: 2581 (N1-Q3, cutting M1-P1) at t=35, 50, 66; 1518 (S1-R4, cutting
  P1-S4) at t=42, 58, 74; 2942 (O1-Q3) at t=86. Eight-plus re-cuts of the same triangle by the
  same three moves. Score-wise the two lines differ: in skip=10/20 scout (blue) holds
  **36–46.6 area continuously** from t=30 to t=110 (score 673@act40 → 2177@110, ~21/action)
  while ours crawls 470 → 1031; in skip=30 both sides get farmed (blue area oscillates
  4.5–31.5 from act 40) but the deficit is already **−346 at act 40** (930 vs 584) and the farm
  cycle widens it to −818 (1553-735).

### 2.2 The doomed shape, actions 45–85 (skip=10 line; skip=30 in brackets where sharper)

| t (probe, our turn) | our area before→after | hull | comps | loops | maxEnemyPop | minDist |
|---|---|---|---|---|---|---|
| 41 | 17.2→19.6 | 76 | 1 | 3 | 0.0 | 1 |
| 45 | 24.9→29.4 | 76 | 1 | 3 | 0.0 | 1 |
| 49 | 18.1→22.6 | 76 | 1 | 3 | 0.0 | 1 |
| 53 | 21.4→23.6 | 76 | 2 | 4 | 0.0 | 1 |
| 57 | 19.1→23.8 | 70 | 2 | 4 | 0.0 | 1 |
| 61 | 20.6→23.4 | 70 | 2 | 3 | 0.0 | 1 |
| 65 | 15.9→20.4 | 70 | 3 | 3 | 0.0 | 1 |
| 69 | 17.1→20.3 | 63 | 2 | 3 | 0.0 | 1 |
| 73 | 14.5→16.2 | 63 | 3 | 3 | 0.0 | 1 |
| 77 | 12.0→15.3 | 69 | 3 | 3 | 0.0 | 1 |
| 81 | 15.3→15.3 | 92 | 3 | 4 | 0.0 | 1 |
| 85 | 12.0→12.0 [skip=30: 2.9→16.4] | 93 [180] | 3 [4] | 3 [0] | 0.0 | 1 |

Reading: **hull inflates 63→180 while loops stay 0–4 and comps 1→4** — the bot spreads into
empty space (a hull with nothing in it) instead of holding enclosed ground; it sits at
Chebyshev-1 contact the whole game. Erosion cadence in 45–85: one enemy pop every 3–8 actions,
each a single Extend (skip=10: −11.2 [46], −4.5 [50], −4.5 [55], −7.5 [63], −5.9 [70], −4.5
[74]; skip=30: −14.2 [46], −5.8 [47], −13.5 [50], −6.2 [54], −13.5 [58], −13.5 [66], −13.5
[74], −13.5 [86]). We spend 2 actions per +13.5 re-close; scout spends 1 action per pop and
(skip=10/20) banks 36–46 area every turn-end regardless. Cuts: suffered 44 (skip=10/20), 40
(skip=30); inflicted 35/39.

### 2.3 League-gate artifact: skip=10 and skip=20 are the SAME game

Identical finals (2177-1031), identical cut counts (35/44), and the logs match with an exact
+10 action offset (skip=20's t=36/40/45/53/60/64/76/80/92 events are skip=10's
t=46/50/55/63/70/74/86/90/102 events, same move ids). Scoutbase's first 10 moves as Blue
reproduce the self-play continuation, so the two "collapse lines" are one game counted twice —
`probe_league`'s 8-game average contains 7 distinct games.

### 2.4 Ranked hypothesis for the collapse

**RANKED HYPOTHESIS (the one): the doom discount removed the single-pop, and what remains is
the farm cycle — the counter is routing to new ground, which the eval never chooses because
(a) probes never activate the avoid/rebuild machinery at all, and (b) on the deployed path the
rebuild penalty is too small to beat the re-close's own gain.** The bot's shape is
deliberately ungrazeable (maxEnemyPop 0.0) but holds nothing: score comes only from
transient +13.5 banked-once closes.
→ **B: test the farm-cycle detector + scaled rebuild penalty of §1.5** — when the enemy has cut
the same ground twice (identical or within CUT_RADIUS), a re-close there must be priced below
*any* fresh-ground move of the same gain; CUT_MEMORY 6 is also borderline for cycles that run
4–8 actions (§3.2 shows the deployed path failing against exactly this on site).
→ **A (secondary):** at skip=30 t=41, depth-3 sees scout's 2581/1518 re-pop at ply 3. If the
+13.5 close still wins on banked-once EV under depth, the fix is B-side routing, not depth.

## 3. Site games (API + `autopsy.rs` replay)

### 3.1 ID resolution

- Short hashes in SUBAGENT-BRIEF are UUID prefixes. `f61a06ec-e366-…` (Riposte vs VladNet,
  1243-2512) and `f4b7f187-37e3-…` (Riposte vs Great Barrier, 2047-4426) resolve via
  `GET /api/games/{uuid}`; `6b66f7db-…` (VladNet vs Riposte, 1701-1215) also replayed.
- **`2ae426c6` does not resolve**: 404, and it is not the prefix of any of the 1,504 games
  reachable by paging `GET /api/games?limit=50` (cursor). It is likely older than the API's
  window. Moving on per instructions.

### 3.2 f61a06ec — VladNet farms us *with anti-rebuild routing active* (deployed path)

Every close of ours ≥ +4.5 is answered by an equal-or-larger pop within 1–2 actions:
[40] +9.5 → [42] −4.5, [43] −9.0; [45] +13.7 → [46] −13.7 (id 10660), [47] −4.5; [49] +4.5 →
[50] −9.5; [56] +6.3 → [58] −6.3; [61] +4.5 → [62] −5.6; [64] +7.9 → [66] −7.9; [72] +9.4 →
[74] −6.3, [75] −7.9; [80] +26.8 → [82] −26.8 (id 1274); [84] +23.0 → [87] −23.0 (id 3388);
[89] +14.7 → [90] −14.7; [92] +18.6 → [94] −18.6; [97] +6.4 → [98] −6.4; [101] +6.3 → [102]
−7.9; [117] +7.4 → [119] −7.4. VladNet reuses pop ids (137, 556, 5191, 7299, 12469, 15718 each
twice); we re-close the same moves (id 14975 recurs at acts 4, 57, 69, 93, 117 — act 4 is an
Extend, the other four are Connect re-closes; 16058 ×2; 16800 ×2).
VladNet's area sits at 48–51.8 from act 40; we end at **7.0 area, 41 cuts suffered** — the
exact rival-analysis §5 fingerprint, and it happened **with** `best_move_with_avoid` + CUT_MEMORY 6
+ CUT_RADIUS 3 + REBUILD_PENALTY 3.0 live. **Measured conclusion: the deployed anti-rebuild
penalty is numerically incapable of flipping these picks** (36 < 54 for a +4.5 re-close;
36 ≪ 198 for +16.5).

### 3.3 f4b7f187 — we out-play GB for 50 actions, then its second mega-loop decides it

Acts 1–50 are our best site half on record: score **+271 at act 50** (655 vs 384), areas 55.1
vs 56.5. GB's first mega-loop lands act 38–40 (area 5.0 → 56.5, score 25 → 102), its **second**
act 50–60 (area 56.5 → **109.2**; score 384 → 925). From act 60 GB banks ~110 area every turn
(~58/action) vs our 30–45; the gap runs −2 (60) → −297 (70) → −658 (80) → −1029 (90) → −1440
(100) → −1845 (110) → −2379 (final). Our 15+ cuts of GB change its area by **+0.0 to +2.1** —
they recover nothing (several even *add* to GB's area: wasted actions against an unbreakable
shape, V4d2 confirmed in rated play) — while GB's 34 cuts of us take −4.5…−18 each.
Divergence window: acts 50–60.

### 3.4 6b66f7db — the mirrored VladNet farm (we are Red)

VladNet pairs its cuts as **both actions of a turn** 12+ times ([44]/[45], [52]/[53], [56]/[57],
[64]/[65], [68]/[69], [72]/[73], [76]/[77], [84]/[85], [92]/[93], [96]/[97], [104]/[105],
[116]/[117]) — popping costs it no banking. Each of our big closes is dead within 1–2 actions:
[47] +23.6 → [48] −23.6 (id 12098); [54] +36.0 → [56] −29.2 (id 4821); [63] +13.7 → [64]+[65]
−7.1, −16.6; [79] +30.4 → [80] −30.4 (id 4173); [82] +27.0 → [84] −27.0 (id 4821 again); [91]
+16.9 → [92] −16.9; [95] +26.3 → [97] −32.3 (id 1629); [119] +26.3 → [120] −28.2 (id 3016).
id 4821 pops us **three times** ([56], [68], [84]). We hold to −164 at act 90, then the same
endgame collapse. Final areas 33.7 (them, alive all game) vs 1.1 (us).

### 3.5 NEW: the currently deployed "Riposte v3" is being farmed by AngelBot WASM too

Paging `/api/games` surfaced two finished losses from **today 13:01Z** not in the brief:
- `4f0c9650` Riposte v3 (blue) vs AngelBot WASM: **413-1017**. Our area is **0.0 at act 30**,
  ≤20.5 the whole game (AngelBot pops us from act 14; 49 cuts suffered). AngelBot area 24.8–34.8.
- `3910c73c` Riposte v3 (blue) vs AngelBot WASM: **1314-2603** (42 cuts suffered).
An alpha-beta engine farms us with the same signature as the transformer. The disease is
ours, not theirs. (Also live: `b64a562b`/`6d96df35` — VladNet beat Riposte v3 both colors at
13:00Z.) Site bot IDs: Riposte v3 `de424a0a-4e74-43cf-b295-8000e33ba30b`, AngelBot WASM
`622b7d84-4eea-433b-9393-05c1afd83db5`, new rivals "Great Barrier 2.0" `3effeb5d…` (1808) and
"Flux" `e25ba7b8…` are active and beating VladNet/GB-0.2 today.

### 3.6 Instrument gap: the league gate never exercises the deployed anti-rebuild path

`lib.rs` replays cut history and calls `best_move_with_avoid(position, &avoid)` — but every
probe (`probe_league`, `probe_v3all`, `probe_bluechair`, `probe_balloon`, `gauge`) calls
`search::best_move`, i.e. `avoid = []`. **All local gates measure the bot without the only
anti-farming machinery it has.** Any rebuild-penalty experiment must be gated with a harness
that reconstructs `avoid` from the game's cut history the way `lib.rs::replay` does — otherwise
we are validating a different bot than the one on site.

## 4. Handed-off hypotheses (one per loss, ranked; addressee first, test second)

| # | Loss / line | Divergence (action, eval) | HYPOTHESIS | Lane: test |
|---|---|---|---|---|
| 1 | 5589 blue chair | [54] Red M1-K4 +13.8; eval −205→−428; score −49→−109 over acts 51–60 | Enemy close-ground release: our exposed edges shield Red's double-duty cut-closes (10+ of Red's closes 54–118 also cut us); no term prices it | **B**: enemy-release term (charge our exposed edges by enemy's best single-action area-gain if cut; one movegen scan like `max_pop`). **A**: depth-3 at pre-[55] must hold > −49 through [58] K4-H7 (+31.5 & cut) |
| 2 | 11723 blue chair | [49] Red F7-E8 pops −37.3; eval +207→−288 (−495 on one action) | Close-survival blindness: big closes on stale, legally-cuttable boundary edges are credited full area×12; doom scalar can't discriminate (all candidates doomed) | **B**: per-close survival pricing (legal-cut scan of the closed region's boundary; cap credit at turns-survived). **A**: depth-3 at pre-[47] — if G6-I6 still wins (banked-once EV), fix is B-side boundary density, not avoidance |
| 3 | 9199 blue chair | [34]–[38] 3 cuts of F7-I10/F7-F10/C7-F10; eval +25→−75; score +51 (40)→−60 (60) | Rebuild-farming susceptibility: anti-rebuild inert in probes, 18–67% of the needed magnitude on site (36 vs 54–198); break bonus pays for worthless cuts ([40]: cut J7-G9 worth +0 to Red) | **B**: scale REBUILD_PENALTY ≥ own_gain×hz on cut ground + weight break bonus by enemy area unbanked (steal #3). Gate with the avoid-reconstructing harness of §3.6. **A**: depth-3 at pre-[40] sees J7-G8 re-cut |
| 4 | Red collapse (balloon) | Farm cycle t=41–89; maxEnemyPop 0.0 everywhere (single-pop is DEAD, doom discount works) | Same root as #3, red side: routing never chosen because re-close always outranks it locally; skip=10 ≡ skip=20 (league double-counts one game) | **B**: farm-cycle detector (enemy cut same ground ≥2×) → forbid/overprice re-closes there ≥8 actions; re-gate `probe_league` deduped. **A**: depth-3 at skip=30 t=41 sees identical re-pop (2581/1518) at ply 3 |

Shared cross-cutting finding for both lanes: **the losses are not "too many cuts" (the None win
suffered 41) — they are area-lifetime asymmetry**: opponents hold 20–118 area alive for dozens
of turns while our area is recyclable. Every rival tested (v1, scoutbase, VladNet, GB,
AngelBot) exploits it; durability comes from boundary legality (V4d2 thickets) + routing, both
eval-side problems today.

## 5. Caveats

- Probe numbering: forced-opening games' probe-n = engine action − 1 (§0); all citations above
  are probe-n as printed in the logs.
- `probe_bluechair`'s opener-response experiment returned no data in the FIRST run
  (no-op bug, §0); **fixed and re-run 2026-09-28, results in §1.6** — mixed (flips 2 of
  5 in opposite directions), so the "one fixed response" idea is measured OUT; the
  instability of the first response is the takeaway.
- Balloon findings rest on the three deterministic skip lines (one distinct collapse game,
  §2.3); conclusions consistent across all three.
- Site autopsies (f61a06ec, f4b7f187, 6b66f7db, 4f0c9650, 3910c73c) are of the *deployed*
  bots (with avoid routing); local probes are without it (§3.6).
- No `retaliator/src/*` files were touched; artifacts: this file + the two probes
  (`probe_bluechair` — fixed opener-response test, `probe_balloon`). Site game JSONs and
  autopsy transcripts kept outside the repo at `/tmp/opencode/sitegames/`.
- Verification re-run 2026-09-28 (fresh binaries, same code base): all five bluechair
  finals and all three balloon finals reproduced byte-identically to the first run;
  the §1.6 opener-response numbers are from the re-run.
