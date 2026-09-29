# C2 color-split analysis — Q10 (2026-09-29)

Rule: per rival/skip table, only splits with n>=8 per color qualify;
flag any >=75% one-color win rate. Sources: site games (202, capybara-v5
perspective — a rival farmer, NOT us), lane scoreboard rows, our 53-loss
corpus (losses only — direction, not rates).

## 1. Site games (qualifying splits)

| matchup (capy perspective) | as Blue | as Red | flag |
|---|---|---|---|
| vs Great Barrier (n=37/color) | 15-22 (41%) | 19-18 (51%) | none (10pp blue gap, same direction as our blue-chair problem) |
| vs Great Barrier 0.2 (n=12/color) | **1-11 (8%)** | 8-4 (67%) | **FLAG: 92% one-color loss as Blue** |
| vs AngelBot WASM (n=9/10) | **9-0 (100%)** | **9-1 (90%)** | FLAG both (capy dominates; AngelBot-side helplessness) |
| vs VladNet (n=8/6) | 4-4 | 2-4 | red n=6 unqualified; blue exactly even |
| vs xmybot (n=6/6) | 2-4 | 3-3 | unqualified |

GB0.2-asymmetry notes: GB 0.2 runs a bigger loop (130 vs 109). As Blue,
capy loses 11/12; as Red it wins 8/12. First-move tempo + bigger enemy
loop = the blue-chair kill shape. Main GB shows the same sign (41% vs
51%) at lower amplitude. Consistent with our corpus: blue led-then-lost
24/28 (86%).

## 2. Lane scoreboard rows (n=5/color — UNDERPOWERED, direction only)

| gate | blue | red | read |
|---|---|---|---|
| S-control (capped d3) | 2/5 | **0/5** | search collapses as Red only |
| S0 (aspiration fix) | 2/5 | **0/5** | unchanged — not aspiration |
| E0 (doom-OFF control) | 1/5 | **4/5** | eval control is a RED player |
| B-2 doom 0.5 | 2/5 | 4/5 | same red lean |
| Baseline v3 vs v1 | 2/5 | 2/5 | symmetric mediocrity |
| O collapser repro | 1-7 (skip0 blue) | n/a | dies as Blue on genuine lines |
| O contest-gen (n=8 blue, QUALIFIES) | **8-0 +17.8%** | 5-3 +2.0% | **FLAG: 100% one-color wins as Blue** — but conversion 0/32, so the flag is selection-only |

## 3. Q10 verdict

- Opposite color leans in the two lanes are the headline: **search doses
  die as Red (0/5 twice), eval doses live as Red (4/5 twice) and die as
  Blue (skip0-blue flips)**. A dose that fixes one lane's weak color must
  be gated on BOTH colors — the modal h2h-pass/league-fail (§gate-review
  P4) is partly this: h2h red wins mask league blue collapse and vice versa.
- External confirmation (GB0.2 1-11 as Blue) that blue-chair-vs-loop is a
  field-wide shape, not our bug alone — but our blue-fade rate (86%
  led-then-lost) is the extreme end; chair play is our weakest color-side.
- Contest-gen Blue 8-0 is a real >=75% flag AND a warning: one-color
  win-rate flags do not imply conversion (0/32 walls closed anyway).
- Recommend: Q10 gate = every future dose reports 4 cells (h2h blue/red,
  league blue-avg/red-avg); any dose with a <=1/5 color cell is color-fail
  regardless of the total.
