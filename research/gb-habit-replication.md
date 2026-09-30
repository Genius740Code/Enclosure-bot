# GB habit replication

Habit: **corridor-infra builds** (Great Barrier = methodical wall-builder). Per `research/archetype-gb.md` §1/§3/§4: GB builds walls with 2+ shared nodes ("near-unbreakable, any cut touches 2+ walls"), anchors them on its home corridor root early, and delays closes until the corridor is safe, banking area rather than snatching. Measured by exact replay of `state.moves` (player/kind/source/target/broken) across the 3 GB games, with **Riposte v7 on the same games as the control**.

## Habit definition (3 lines)

1. **C1 — 2+ shared nodes:** share the profile's own `corridor_infrastructure_score()` shared-node term, evaluated at the move's *attachment node* (`source`; in this engine `extend` chains onto structure at `source`, `target` is the fresh node). Hit = `shared2 >= 2`.
2. **C2 — home-root control:** fraction of `extend` moves in the first 20 actions landing within Chebyshev distance 3 of that color's home corridor root. Roots re-derived from the games (blue D10 = (-6,0), red P10 = (6,0), mirror-symmetric) because the profile's own coordinates (P11=(-3,1), J10=(0,4)) do not match the observed grid.
3. **C3 — banking over snatching:** `connect` (close) frequency, and the action index of the first close. Denominator for C1/C2 is `extend` moves only; `connect`/`capture` excluded.

## Counts per game

| Game | Side | extends | **C1** shared2>=2 | mean src deg | **C2** root<=3 (<20 acts) | **C3** connects | first close |
|---|---|---|---|---|---|---|---|
| `30c7653b` (GB blue, W) | **GB** | 45 | 10 (**22.2%**) | 1.47 | 7/9 (77.8%) | **13** | 15 |
| | v7 | 35 | 21 (60.0%) | 2.14 | 6/6 (100.0%) | 24 | 9 |
| `ad65f054` (GB red, W) | **GB** | 49 | 6 (**12.2%**) | 1.24 | 7/9 (77.8%) | **10** | 13 |
| | v7 | 34 | 16 (47.1%) | 1.76 | 7/8 (87.5%) | 24 | 12 |
| `d2d4b4fd` (GB red, W) | **GB** | 43 | 17 (**39.5%**) | 1.79 | 5/8 (62.5%) | **14** | 9 |
| | v7 | 32 | 11 (34.4%) | 1.78 | 8/8 (100.0%) | 26 | 12 |
| **POOLED** | **GB** | **137** | **33 (24.1%)** | 1.50 | 19/26 (73.1%) | **37** | 12.3 |
| | v7 | 101 | 48 (47.5%) | 1.90 | 21/22 (95.5%) | 74 | 11.0 |

**Deltas vs control (pooled):** C1 **−23.4 pp** · C2 **−22.4 pp** · C3 **37 vs 74 closes = 0.50x** (per-game 0.54x / 0.42x / 0.54x — consistent in all 3).

C1 phase split (GB): opening 1/17, mid 16/57, late 16/63 — the profile's "mid-game corridor infrastructure" window (12–59) is where GB is *weakest* relative to v7 (28.1% vs 61.5%).

## NOT REPLICATED

**Headline claim (C1, "2+ shared nodes = near-unbreakable corridor") FAILS the bar and is INVERTED.** GB hits 24.1% pooled against a ≥70% bar (per-game 22.2 / 12.2 / 39.5%), and sits **23.4 pp *below* the v7 control** — GB builds the *sparser*, more linear chains (mean attachment-node degree 1.50 vs 1.90). The profile's central "any cut touches 2+ walls" mechanism does not describe how GB wins; density is what v7 does.

C2 nominally clears 70% (73.1%) but is **22.4 pp below control** in all 3 games — root-anchoring is ordinary play, not a GB signature.

**The one real signal is C3, and the profile mis-describes it.** GB closes **half as often** as v7 (0.50x, consistent 0.42–0.54x across all 3 games) — that is a large, clean, one-sided effect. But the mechanism is **not** "delays the first close to action 13+": mean first close is 12.3 (GB) vs 11.0 (v7), and per-game first-close is 15/13/9 for GB against 9/12/12 for v7 — noise, not discipline. GB's actual signature is **never closing** (half the closes), not a delayed first close.

**Actionable for lanes:** do **not** cite "2+ shared nodes / near-unbreakable corridor" or the root-control figures as GB habits — both are below or indistinguishable from a v7 baseline. The one GB number that survives replication is the **close-rate halving**, which is directly measurable and consistent across all 3 games, and is the candidate worth carrying into a GB-tell feature. Secondary defect for the profile author: the declared corridor-root coordinates are wrong for this grid and should be corrected before anyone codes `corridor_infrastructure_score` against them.

---
*Sources (read-only via `git show`, nothing checked out, no repo files written): `origin/lane-v8-reply:research/archetype-gb.md` (2454256); `origin/lane-v8-autopsy:research/games/{30c7653b,ad65f054,d2d4b4fd}.json`. Script: `/tmp/gbrep/replicate.py` (stdlib python3, one replay pass per game).*

---
## Routing note (main session, 2026-09-30)

- Do NOT cite "2+ shared nodes / near-unbreakable corridor" or root-control
  figures as GB habits — NOT replicated (C1 inverted vs v7 control).
- Surviving GB signal: close-rate halving (0.50x, 3/3 games). Candidate for a
  GB-tell feature, not a corridor-infra bonus.
- Consequence for S2 (`origin/lane-v8-reply` 2454256 pseudo-spec): its
  `corridor_infrastructure_score` premise is invalidated by C1. S2 dose must be
  re-based on the close-rate tell or the lane kills. M2 port stays BLOCKED.
- Profile coordinates for corridor roots are wrong for this grid — correct
  before any code keys off them.
