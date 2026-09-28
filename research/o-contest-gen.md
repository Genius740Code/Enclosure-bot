# Lane O — DEDICATED CONTEST move generation vs COLLAPSER

**Probe**: `retaliator/examples/opp_contest.rs` @ tip+41a6828, release.
Config: skip=0, 8 solos, n=8 per arm per color (32 games). BONUS=0
(contest wins only on full-horizon merit vs baseline pick).

## Conversion vs tempo table

| Arm | W-L | Avg margin | ourCuts/game | contest cut/occupy per game | conversion (no foe area>=25) |
|---|---|---|---|---|---|
| BASELINE ret=Blue | 1-7 | -11.1% | 34.6 | — | 0/8 |
| CONTEST ret=Blue | **8-0** | **+17.8%** | 38.0 | 25-35 / 8-15 | 0/8 |
| BASELINE ret=Red | 4-4 | +1.2% | 38.5 | — | 0/8 |
| CONTEST ret=Red | 5-3 | +2.0% | 38.5 | 21-37 / 12-16 | 0/8 |

## Reading

1. **Not KILL**: contests do not donate necks. Occupy moves (8-16/game at
   foe construction sites — exactly what FRESH_PENALTY forbids) coincide
   with a +28.9% Blue swing and no Red regression beyond noise (+0.8%).
2. **Not cut volume**: baseline already cuts 34-39x/game; contest cuts
   ~same count. The win is SELECTION: site-targeted cuts (corridor-root
   joints) + occupies picked on full-horizon merit that the capped
   horizon-12 eval rejects.
3. **Conversion 0/32**: the wall ALWAYS closes (foe max area 35-85 every
   game). Contest-gen does not prevent the close — as Blue it out-banks
   through it. "Prevent-the-wall" as a goal is dead; "contest-and-out-bank"
   is the viable frame.
4. **Blue-specific**: Red gains nothing (+0.8%). First-move advantage
   decides who profits from the construction-site fight. Do NOT ship as a
   color-blind change.
5. **Watch line**: contest ret=Red game5 -67.8% (baseline -35.8%) — contest
   deepened one loss. No <-300 collapse anywhere.

## Handoff (main session / Lane B-E-S)

- Candidate BLUE-ONLY counter-proposal: contest-set generation
  (CUT_SITE + OCCUPY within 2 of foe fresh/shielded wall, full-horizon
  merit pick) behind the CONTEST phase gate (`research/v7-phase-spec.md`).
- Still required before ship: probe_b_h2h >= 6/10, league_b > -9.9% with
  no collapse < -300, gauge_b 6/6 (main-session harnesses; Lane O cannot
  run them from this worktree without touching eval/search sources).
- Suggested Lane E first step (one variable): FRESH_PENALTY 1.5 → 0.0 for
  Blue only, then re-run this probe's Blue arm as gate.
