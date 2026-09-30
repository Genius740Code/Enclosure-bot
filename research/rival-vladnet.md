# Rival Analysis: VladNet (from game 3a414aad-5aeb-4c16-87c1-f89e87966b93 + 1c69056c-dd8a-4dc4-b62c-4846d08e92c8)

**Source**: 2 finished games vs Riposte v7 (both VladNet wins)
- **3a414aad**: VladNet as RED, v7 as BLUE — VladNet wins 2378 vs 1371
- **1c69056c**: VladNet as BLUE, v7 as RED — VladNet wins 2255 vs 1530

**VladNet Identity**: Medium-strength neural net bot (not GB/Angel tier). Beatable — but exposes v7's structural weaknesses.

---

## What VladNet Does

### 1. Early Bank Construction (Moves 1-40)
- Builds **shared-node 2-wall** structures by move 20-30
- Achieves **49.2 area bank by move 40** (3a414aad as Red) — survives, but is not unbroken (see §3)
- As Blue (1c69056c): builds 33.0 area by move 30, 60.0 by move 40
- **Method**: Thicket → mesh → 2-wall shared node → depth extension
- **Key difference from v7**: VladNet's banks have **depth** (3+ moves from edge), v7's are often shallow (1-2 moves)

### 2. Precision Break Timing (Moves 40-60)
- Waits for opponent to **overcommit to a close** (Connect into corridor)
- Breaks **immediately after our Connect** — same move or next opponent move
- In 3a414aad: 4 breaks in moves 42-47 against our moves 41, 45
- **Never breaks randomly** — only when we donate a breakable corridor

### 3. Bank Defense / Never Donates Back
- **3a414aad**: VladNet's 49.2 area bank is resilient but **not break-proof** — v7 landed **7 breaks** (one ≥10: **15.0 at move 113**, dropping it 53.25 → 38.25); VladNet rebuilt back to 49.25 by move 119 and won on **11 faces**
- **1c69056c**: VladNet as Blue **won 2255–1530**, but its bank did **not** hold — area 60.0 @40 → 33.0 @80, peak 64.8 @85, ended 35.3, and it took **6 catastrophic breaks** (≥10 area; worst −31.8 @86). The earlier "36.2→19.7, no catastrophic break" reading was v7's own area, not VladNet's
- Our breaks on VladNet: in 3a414aad, 7 of our 60 moves dealt damage at all, only **1** effective (≥10) — 31.26 area total off a bank that still ended at 49.25
- VladNet's structure: shared nodes make it **game-dependent, not reliably redundant** — in 3a414aad a 15.0 break did not hold and it rebuilt to 49.25, but in 1c69056c v7's 31.8 break at move 86 collapsed it 64.8 → 33.0 in one shot

### 4. Tempo Control
- As Red (3a414aad): Uses pair-turn advantage (moves 1-2 Red) to establish corridor control before Blue's first Connect
- As Blue (1c69056c): Matches Blue's opener, then transitions to thicket faster
- **Does not play "area moves" blindly** — every move serves bank construction or break setup

---

## When VladNet Does It

| Phase | Moves | VladNet Action | v7 Failure |
|-------|-------|----------------|------------|
| **Opening** | 1-20 | Thicket + shared-node census | Follows mesh8 script (D10-F7) into prepared corridors |
| **Bank Build** | 20-40 | 2-wall completion + depth | Tries to race area, builds shallow loops |
| **Break Window** | 40-60 | Waits for our Connect → surgical break | Re-closes broken corridors (donates 15-25 area/cycle) |
| **Midgame** | 60-100 | Holds bank, extends depth | Repeats re-close → break cycles (6-10 cycles) |
| **Endgame** | 100-120 | Bank intact, we exhausted | Area collapse to 15.3 vs 49.2 |

---

## What Beats VladNet

### Evidence from 30c7653b (CORRECTED 2026-09-29: we were RED, GB won 4335-2709 as Blue)
- GB (blue) built the **112.5 area bank by move 120** — unbroken all game (Cluster 2)
- Ours (red): 47.8 at 120. We led at 40 (+47.6) and 50 (+1.2), were **−64.3 at 60**, and our worst break was **−22.8 at move 120** (Cluster 5)
- The prior reading was inverted (4335 is GB's score, 112.5 is GB's bank). Real lesson:
  even with red's free opening, our banks pop late while theirs hold — bank fragility,
  not opener freedom, decided this game. The "25 breaks / 4 catastrophic" claim is withdrawn.

> **Correction note (2026-09-30, `our-color-rerun.md` re-run, d6a4951).** Two numbers in the two
> bullets above were wrong and are fixed inline: we did **not** lead at 60 (we were −64.3 there;
> the leads were +47.6 @40 and +1.2 @50), and the **−39.7 break at move 101 is b0ac4141's, not
> this game's** — 30c7653b's move 101 was −3.4, and its worst break was −22.8 @120 (8 breaks ≥10
> on us, totalling 188.8). Also: "unbroken all game" overstates it — GB's bank peaked **133.5 @116**
> and ended 112.5, giving back **21.0**. Verified by replaying `30c7653b` and `b0ac4141` from
> `research/games/` through the vendored engine; output reconciles exactly with each JSON's final
> `state.areas`/`state.score`. We were RED, GB BLUE, in both.

### Hypothesis: VladNet is Beatable By
1. **Corridor-root contest at move 20-30** (Lane W: WALL-RACE 1.0)
   - Detect enemy 2-wall shared nodes at move 20
   - Place **single stone at corridor root** (not in corridor) — prevents bank completion
   - Cost: 1 move. Gain: denies 50+ area bank

2. **No re-close discipline** (Lane P: rebuild-denial gate)
   - When our corridor breaks: **do not re-close same corridor**
   - Expand elsewhere — force VladNet to spend moves breaking new corridors
   - VladNet's breaks are reactive; make them chase

3. **Build our own unbreakable bank first** (Lane W: steal method)
   - Mirror VladNet: thicket → shared-node 2-wall → depth
   - But **gate on tempo** (Lane W): only if we're not behind on move 20
   - Red has advantage: pair-turns 1-2 → can start bank before Blue's first Connect

4. **Red opener design** (Lane R)
   - VladNet as Blue (1c69056c) **won 2255–1530** — and its area did collapse: 60.0 @40 → 33.0 @80
   - VladNet as Red (3a414aad) crushed us
   - **Red's pair-turns (moves 1-2) are the key** — VladNet uses them for bank roots
   - v8 Red needs: opener that claims corridor roots before Blue's move 3

---

## VladNet vs Other Top Bots (Inference)

| Bot | VladNet Likely Result | Reason |
|-----|----------------------|--------|
| **GB / GB 2.0** | VladNet loses | GB builds deeper banks + better break timing (see d2d4b4fd: GB2 117 area unbroken) |
| **AngelBot WASM** | VladNet loses | Angel builds impenetrable 2-wall shared-node (E3: 52.5% unbreakable-share winrate) |
| **capybara** | VladNet loses | Capybara net v5 — neural net with better eval |
| **Stompy** | VladNet loses | Stompy crushed v7 4393-1517 (9c3b27a5) — aggressive break-first style |
| **YshanV1** | VladNet loses | Top bot, builds walls v7 can't break |
| **Riposte v8 (target)** | **v8 wins** | If v8 implements: corridor-root contest, no re-close, Red opener, unbreakable bank |

---

## VladNet's Weaknesses (Exploitable)

| Weakness | Evidence | Exploit |
|----------|----------|---------|
| **No proactive break hunting** | Only breaks when we donate corridor | Never donate breakable corridors; force VladNet to attack intact banks |
| **Bank is broad, not a single fragile cell** | 3a414aad: 49.25 area spread over **11 faces** at the end; a 15.0 break at move 113 dented it to 38.25 but it recovered to 49.25 — "one bank, no backup" overstates it | Split our area 2-3 ways too — broad banks survive single breaks on either side |
| **Predictable break timing** | Always immediately after our Connect | Bait: play Extend in corridor → VladNet waits → we close elsewhere |
| **"Red-only dominance"** — *not supported* | As Blue, VladNet **won 2255–1530** and its area **collapsed** 60.0@40 → 33.0@80 (peak 64.8@85, end 35.3). It does not lose Blue gradually (1c69056c) | No Blue-specific counter to copy — VladNet won on both colors here; the transferable gap is our own bank construction (Cluster 2) |
| **No endgame technique** | 1c69056c: VladNet 60.0@40 → 33.0@80 (peak 64.8@85, ended 35.3) yet still banked the win 2255–1530. The "36.2→19.7" pair is **v7's own** area (36.25@40 → 19.74@120), not VladNet's | In endgame, out-maneuver: VladNet has no plan when board fills |

---

## v8 Implementation Checklist (from this analysis)

- [ ] **Lane W**: WALL-RACE 1.0 trigger at move 20 — shared-node census → corridor root contest
- [ ] **Lane P**: Rebuild-denial gate — `rebuild_allowed = (re_place_blocked_count < 2)`
- [ ] **Lane R**: Red opener design — claim 2 corridor roots in moves 1-2 (pair-turns)
- [ ] **Lane W**: Own bank construction — thicket → shared-node 2-wall → depth (gated on tempo ≥0)
- [ ] **Lane P**: Move pricing — every Connect must carry lifetime area ≥2.0 while open_space > 20
- [ ] **C2**: Track "VladNet-style bank unbroken" metric per game

---

## Game 1c69056c (VladNet as Blue) — Additional Notes

**VladNet Blue opener** (VladNet's own moves only — in 1c69056c it plays Blue, so its moves are the odd-numbered ones): Move 1: Extend A10-A12, Move 4: Extend D10-A13, Move 5: Connect A13-A12 (+4.5), Move 8: Extend A13-D11, Move 9: Extend D10-A7
- Very passive first 10 moves (VladNet area **4.5** at move 10; the 2.5 there is v7's Red area)
- Then: Move 17: Connect A10-A7 (+4.5) → 12.0 — VladNet's own thicket→bank transition, its first move past 10 area
- The "+13.8 at move 14" line was **v7's** (Red) move, M13-P10 Connect +13.75, not VladNet's
- Move 40: OPP BREAK -2.4 (first break taken) — VladNet's first break, correct
- **Key insight**: VladNet as Blue **accepts early passivity** to build thicket, then converts to bank
- v7 as Red (us) had only **29.0** area at move 50 — the 48.0 there is **VladNet's**, and v7 did not regain 48 until move 118 (peak 48.2)
- v7 **did** break VladNet's bank: **6 breaks ≥10 area** (12.0@43, 15.0@67, 15.0@78, 31.8@86, 15.0@103, 15.0@119). The "-2.4, -2.1, -3.4, -2.7, -4.0, -7.5" series is **VladNet's** breaks on us (moves 40-53), not ours
- VladNet's bank was **not** redundant: v7's **31.8 break at move 86** collapsed it 64.8 → 33.0, all the way back to the move-80 floor

**What we did wrong as Red**: 
- Built area (**29.0 at move 50**, well behind VladNet's 48.0) but in **wrong places** — not contesting VladNet's bank roots
- Our breaks were **deeper** (max **-31.8 at move 86**) vs VladNet's breaks on us (max -28.5 at move 120); the -7.5 was VladNet's break on us at move 53, not ours

> **Correction note (2026-09-30, round 2 re-run).** Three further claims in this file were
> misattributed between the two bots; all three verify as wrong and are fixed inline above.
> Method: replayed `1c69056c` (and `3a414aad` for the cross-checks) from `research/games/`
> through the vendored engine, action by action, with the +9 coordinate shift; the replay
> reconciles exactly with each JSON's final `state.areas` / `state.score`.
>
> - **(a) "v7 as Red got to 48 area by move 50"** → v7 (Red) had **29.0** at move 50; the 48.0
>   is **VladNet's** (Blue). v7's first return to 48 is move 118 (peak 48.19). The companion
>   claim that v7 "couldn't break VladNet's bank … all small" is **reversed**: the
>   −2.4/−2.1/−3.4/−2.7/−4.0/−7.5 series is VladNet's breaks **on us** (moves 40-53), while
>   **v7 landed 6 breaks ≥10 on VladNet** (12.0@43, 15.0@67, 15.0@78, 31.8@86, 15.0@103,
>   15.0@119) — which is exactly what §3 above already records. VladNet ended on 8 faces /
>   35.3 area, v7 on 8 faces / 19.7.
> - **(b) "1c69056c: area 36.2→19.7 over 80 moves"** → that pair is **v7's own** area
>   (36.25 @40 → 19.74 @120), the same misattribution as the §3 line above it. VladNet went
>   60.0@40 → 33.0@80, peak 64.8@85, end 35.3.
> - **(c) "As Blue, VladNet loses area gradually"** → VladNet **won** as Blue, 2254.75–1530.32,
>   and its area **collapsed** 60.0@40 → 33.0@80. So neither "loses area gradually" nor the
>   "Red-only dominance" label survives; §3 and "What Beats VladNet" #4 already recorded the
>   same result, and the two lines previously contradicted each other.
>
> **Round 3 (2026-09-30) — all five deferred items above are now verified and fixed inline.**
> Same method: replayed `1c69056c` and `3a414aad` from `research/games/` through the vendored
> engine action-by-action with the +9 shift; replay reconciles exactly with each JSON's final
> `state.areas`/`state.score`, and all 55 new numbers are assertion-gated.
> - §1 "builds **36.2** area by move 30" → **33.0**. VladNet Blue is 33.0 @30; 36.25 is v7's
>   Red area at move 40 (v7 Red is only 28.37 @30). The "60.0 by move 40" half was correct and
>   is untouched.
> - The "VladNet Blue opener" list mixed both colors. VladNet's real first 10: **M1 Extend
>   A10-A12, M4 Extend D10-A13, M5 Connect A13-A12 (+4.5), M8 Extend A13-D11, M9 Extend
>   D10-A7**, holding **4.5** area at move 10 (2.5 there is v7's Red). Moves 2, 6, 7, 10 and
>   14 are all **v7 Red**; move 14 = M13-P10 Connect **+13.75**. VladNet's own thicket→bank
>   transition is **move 17** (Connect A10-A7, +4.5 → 12.0, its first move past 10 area).
>   Move 1 and move 40's −2.4 break were already VladNet's and are untouched.
> - "bank had **redundancy**" → **false**: v7's 31.8 break at move 86 took it 64.8 → 33.0,
>   exactly back to the move-80 floor.
> - "Our breaks were shallow (max -7.5)" → **inverted**: v7's max on VladNet was **-31.8 @86**
>   (6 breaks ≥10). VladNet's max on v7 was 28.45 @120; the −7.5 was VladNet's break on us
>   at move 53.
> - 3a414aad "**0 breaks**" / "**never broken**" / "**no backup**" → all three overstated it.
>   v7 landed **7 breaks**, one ≥10: **15.0 at move 113** (53.25 → 38.25). VladNet rebuilt to
>   49.25 by move 119 and won on **11 faces** (12 @60, 13 @80), so the "single bank, no backup"
>   weakness cell is now stated as a broad multi-face bank.
> - Two adjacent §3 lines in the same 3a414aad block were also wrong and left the section
>   self-contradicting, so they are folded in here: "**29 attempts** in 3a414aad, only **4
>   effective** (≥10)" → of our **60** moves, **7** dealt any damage and only **1** cleared 10
>   (31.26 area total). And "VladNet's structure: **redundant shared nodes** — breaking one
>   corridor doesn't collapse the bank" → true in 3a414aad (the 15.0 break did not hold) but
>   false in 1c69056c (31.8@86 collapsed it), so it is now stated as game-dependent.
>
> **Still open, NOT verified this round** (left untouched, flagged for round 4): §2 "In 3a414aad:
> 4 breaks in moves 42-47 against our moves 41, 45" — VladNet's breaks in that window are
> **3** (@42=4.5, @43=3.375, @47=19.375), not 4; and our moves 41/45 were v7 Blue *Connects*,
> not the trigger being described. Same family of error, but out of this round's scope.

---

*Generated by v8 C2 Autopsy Agent — lane-v8-autopsy — 2026-09-29*
*Part of M6: Rival strat files (v8-plan.md §6f)*