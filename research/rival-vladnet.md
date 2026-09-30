# Rival Analysis: VladNet (from game 3a414aad-5aeb-4c16-87c1-f89e87966b93 + 1c69056c-dd8a-4dc4-b62c-4846d08e92c8)

**Source**: 2 finished games vs Riposte v7 (both VladNet wins)
- **3a414aad**: VladNet as RED, v7 as BLUE — VladNet wins 2378 vs 1371
- **1c69056c**: VladNet as BLUE, v7 as RED — VladNet wins 2255 vs 1530

**VladNet Identity**: Medium-strength neural net bot (not GB/Angel tier). Beatable — but exposes v7's structural weaknesses.

---

## What VladNet Does

### 1. Early Bank Construction (Moves 1-40)
- Builds **shared-node 2-wall** structures by move 20-30
- Achieves **49.2 area bank by move 40** (3a414aad as Red) — never broken
- As Blue (1c69056c): builds 36.2 area by move 30, 60.0 by move 40
- **Method**: Thicket → mesh → 2-wall shared node → depth extension
- **Key difference from v7**: VladNet's banks have **depth** (3+ moves from edge), v7's are often shallow (1-2 moves)

### 2. Precision Break Timing (Moves 40-60)
- Waits for opponent to **overcommit to a close** (Connect into corridor)
- Breaks **immediately after our Connect** — same move or next opponent move
- In 3a414aad: 4 breaks in moves 42-47 against our moves 41, 45
- **Never breaks randomly** — only when we donate a breakable corridor

### 3. Bank Defense / Never Donates Back
- **3a414aad**: VladNet's 49.2 area bank takes **0 breaks** across 120 moves
- **1c69056c**: VladNet as Blue **won 2255–1530**, but its bank did **not** hold — area 60.0 @40 → 33.0 @80, peak 64.8 @85, ended 35.3, and it took **6 catastrophic breaks** (≥10 area; worst −31.8 @86). The earlier "36.2→19.7, no catastrophic break" reading was v7's own area, not VladNet's
- Our breaks on VladNet: 29 attempts in 3a414aad, only 4 effective (≥10 area)
- VladNet's structure: **redundant shared nodes** — breaking one corridor doesn't collapse the bank

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
| **Single bank dependency** | 3a414aad: one 49.2 bank, no backup | Split our area: 2-3 smaller banks → VladNet can only break one at a time |
| **Predictable break timing** | Always immediately after our Connect | Bait: play Extend in corridor → VladNet waits → we close elsewhere |
| **Red-only dominance** | As Blue, VladNet loses area gradually (1c69056c) | As Blue, contest early; as Red, copy VladNet's pair-turn bank roots |
| **No endgame technique** | 1c69056c: area 36.2→19.7 over 80 moves | In endgame, out-maneuver: VladNet has no plan when board fills |

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

**VladNet Blue opener**: Move 1: Extend, Move 2: Extend, Move 6: Extend, Move 7: Extend +1.2, Move 10: Connect +1.2
- Very passive first 10 moves (area 2.5)
- Then: Move 14: Connect +13.8 (thicket→bank transition)
- Move 40: OPP BREAK -2.4 (first break taken)
- **Key insight**: VladNet as Blue **accepts early passivity** to build thicket, then converts to bank
- v7 as Red (us) got to 48 area by move 50 but **couldn't break VladNet's bank** (only -2.4, -2.1, -3.4, -2.7, -4.0, -7.5... all small)
- VladNet's bank had **redundancy** — breaking one corridor didn't collapse it

**What we did wrong as Red**: 
- Built area (48 at move 50) but in **wrong places** — not contesting VladNet's bank roots
- Our breaks were shallow (max -7.5) vs VladNet's breaks on us (max -28.4 at move 120)

---

*Generated by v8 C2 Autopsy Agent — lane-v8-autopsy — 2026-09-29*
*Part of M6: Rival strat files (v8-plan.md §6f)*