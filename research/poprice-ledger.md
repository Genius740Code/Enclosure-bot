# POP-PRICE Decision-Node Ledger (Game 3a414aad)

**Objective**: Audit the 7 re-close decision nodes from game `3a414aad` (Actions 56, 57, 64, 80, 84, 88, 89) to determine if move pricing causes suicidal re-closing, and evaluate whether Dose 1 (D1) flips the move choice to non-close alternatives.

---

## 1. Per-Node Decision Ledger

Evaluated at each node with `MOVE_BUDGET = 4096`, `avoid` history tracking cut memory, and exact `retaliator::search` terms:

| Node (Action) | Chosen Re-Close | Rank (In-Beam) | Re-Close Gain | Honest Eval | Horizon Tail | Doom Charge | Rebuild / Fresh | Total Adj | Best Non-Close Alt | Non-Close Rank (In-Beam) | Non-Close Adj | Gap (Re-close - Non-close) | D1 Re-Close Adj | D1 Flips? |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **56** | `(-3,-2)-(-5,0)` | 5 (Yes) | +17.8 | -122.95 | +307.00 | 213.60 | 0.00 / 0.00 | -29.55 | `(-3,-2)-(-5,-5)` | 78 (No) | -42.45 | **+12.90** | -443.35 | **YES** |
| **57** | `(-8,-3)-(-5,0)` | 1 (Yes) | +12.9 | -357.55 | +0.00 | 154.80 | 0.00 / 0.00 | -512.35 | `(-4,-2)-(-5,-5)` | 81 (No) | -550.25 | **+37.90** | -589.75 | **YES** |
| **64** | `(0,1)-(-2,3)` | 1 (Yes) | +17.1 | -359.85 | +158.50 | 204.75 | 0.00 / 0.00 | -406.10 | `(-3,-2)-(-5,-5)` | 104 (No) | -409.16 | **+3.06** | -666.97 | **YES** |
| **80** | `(-7,3)-(-5,5)` | 1 (Yes) | +12.6 | -696.46 | +73.00 | 150.74 | 0.00 / 0.00 | -774.20 | `(0,3)-(3,6)` | 106 (No) | -760.79 | **-13.41** | -922.56 | **YES** |
| **84** | `(-6,0)-(-6,3)` | 1 (Yes) | +8.1 | -841.22 | +35.70 | 97.20 | 0.00 / 0.00 | -902.72 | `(0,3)-(3,6)` | 68 (No) | -832.02 | **-70.70** | -987.02 | **YES** |
| **88** | `(-7,3)-(-4,0)` | 8 (Yes) | +18.7 | -601.94 | +75.68 | 295.50 | 0.00 / 0.00 | -821.76 | `(-5,5)-(-4,2)` | 15 (No) | -1028.22 | **+206.45** | -1009.64 | **NO** |
| **89** | `(-8,5)-(-5,5)` | 1 (Yes) | +18.8 | -897.44 | +0.78 | 225.00 | 0.00 / 0.00 | -1121.66 | `(-5,5)-(-4,2)` | 10 (No) | -1152.65 | **+30.98** | -1234.94 | **YES** |

---

## 2. Key Diagnostic Findings

### 2.1 The Twofold Failure: Beam Filtering + Phantom Tail
1. **Beam Seeding Lockout**: Across **all 7 nodes (100%)**, the chosen re-close ranked in the top 8 of `ranked()` (ranks 5, 1, 1, 1, 1, 8, 1) and entered the search beam. Conversely, **0 of the best non-close alternatives entered the beam** (ranks 78, 81, 104, 106, 68, 15, 10). The initial priority function awards massive points to loop area gains, completely shutting out expanding non-close moves before 2-ply search can evaluate them.
2. **Phantom Horizon Tail**: In Nodes 56, 64, 80, 84, and 88, `horizon_extension` awards **+35.7 to +307.0 points** for the newly claimed territory, assuming the bank survives for the remaining `E` scoring events. In reality, VladNet pops 100% of these banks on the immediate next turn, turning the tail into pure phantom equity.
3. **Doom Under-charging**: Even at Actions 80 and 84 where the selection adjusted score of the non-close alternative was already superior (-760.79 vs -774.20, and -832.02 vs -902.72), the engine played the re-close anyway because the non-close was excluded from the beam (`in_beam = false`).

### 2.2 Dose 1 (D1) Evaluation
Under D1 (pricing the immediate pop: deleting the phantom horizon tail and factoring pop doom `DOOM_TAIL_W * pop * hz`):
- Action 56: Flips to non-close (-443.35 vs -42.45, +400.9 pt swing)
- Action 57: Flips to non-close (-589.75 vs -550.25, +39.5 pt swing)
- Action 64: Flips to non-close (-666.97 vs -409.16, +257.8 pt swing)
- Action 80: Flips to non-close (-922.56 vs -760.79, +161.8 pt swing)
- Action 84: Flips to non-close (-987.02 vs -832.02, +155.0 pt swing)
- Action 88: No flip (-1009.64 vs -1028.22, re-close still slightly leads due to tactical mess)
- Action 89: Flips to non-close (-1234.94 vs -1152.65, +82.3 pt swing)

---

## 3. Kill-0 Verdict

- **Total Nodes**: 7
- **Nodes where D1 flips choice**: **6 / 7 (85.7%)**
- **Threshold**: `flips < 3/7 -> KILL-0 (pricing is not the cause)`
- **Verdict**: **PASSED (SURVIVES)**.

**Conclusion**: Pricing IS an overwhelming causal driver of the rebuilder-farming losses. Correcting the phantom tail and pricing the pop penalty flips 6 out of 7 nodes away from the suicidal re-close. However, because all 7 non-close moves were filtered out by `ranked()` prior to beam insertion, POP-PRICE (Lane P) must be paired with BEAM-SEED (Lane S2) so that non-close candidate moves actually reach the search beam.
