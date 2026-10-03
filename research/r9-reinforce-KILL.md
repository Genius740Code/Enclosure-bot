# REINFORCE-LINES dose (v9 item 4) — KILL after 3-weight dose-response — 2026-10-03

Branch: `lane-v9-reinforce` (worktree `/tmp/opencode/game-v9reinf-g2`, fresh
`git worktree add` per law; this lane replaced the prior GLM lane that never
created a worktree). Base: scaffold `530d4b4`. Kill-0 GO adopted from
`lane-v9-reinf` (`5e43810`: builds behind 2+ own shared nodes survive 84.5%
pooled, 94.6%/95.7% in wins vs 79.1%/28.6% in losses, both chairs).
Ownership respected: only `retaliator/src/eval_reinforce.rs` (rewritten from
the inert H1 scaffold into the v9 dose rig), `retaliator/examples/
probe_r_reinforce.rs` (rewritten), these logs, this note, and the scoreboard
row. `search.rs` / `lib.rs` untouched — the shipped bot is provably
unaffected (identity below).

## Dose (one variable: REINFORCE_W)

`adjusted += REINFORCE_W x Δdurable x hz` at candidate selection, where
Δdurable = the change across our turn in the area of ours that survives the
enemy's best SINGLE legal cut. Measured by engine legality (structural, no
shape names): the enemy's full legal-move list is enumerated on a
shield-cleared enemy-to-move view (`Position::setup`, no recent edges — the
rule-6 transient shield never credits a farmed re-close), worst pop
subtracted from held area. A 2+-touch wall reads in full, a single-touch
loop reads as poppable whatever it touches. Gate: the term fires only when
the move gains area or lands within DEADWOOD_ENEMY_DIST of the enemy
(DEADWOOD_PENALTY keeps its exclusive claim on far quiet no-gain Connects).
Full-horizon points (x min(events, 12)); tiebreaks unchanged (center
distance, then move index — the shipped D1).

How this differs from the killed v7 forms (scoreboard E1-E6 / Q12-D1-D2):
those paid proximity+degree bonuses that fired on thickening regardless of
measured protection. This term pays ONLY a measured Δdurable increase — it
cannot fire on slow thickening that protects nothing, and pays nothing on
the farmed re-closes (still legally poppable). It is c-autopsy §2 #2's
"price our own post-move cuttability" in bonus form. The prior H1 lane's
term (enemy-node adjacency, swept {0.5,1,2,4} with a mis-set W=1.5
"control", all weights identical = vacuous, NO-GO on
lane-v9-reinforce-sweep) is a different term and was not re-run.

## Rig

- `eval_reinforce.rs`: verbatim skeleton copy of shipped `search.rs` +
  the dose block in `analyze_dosed` (w=0.0 skips the block entirely).
- `probe_r_reinforce.rs`: selftest / identity / h2h / league / gauge / v1 /
  corpus / gates subcommands. Deterministic (fixed openings/skips, no RNG).
- BARS stated before every run (in the logs): identity 0 mismatches over
  >=239; h2h >=6/10 no chair <2/5; league AVG(dose) > AVG(control) and no
  row < -300%; gauge >=7/8; v1 dose >= control; survival dose > control.
  Discipline stated before D1: league fail at 0.5 -> D2 0.25 -> still fail
  -> KILL (monotone-negative = the v7 E-family verdict).

## Numbers (all logs in research/logs/r9-reinforce-*.txt)

Rig validity:
- selftest: 6/6 PASS (single-touch triangle durable 0.0; chorded 2-touch
  triangle durable 2.25 exactly; unreachable triangle durable == area;
  invariants 0 <= durable <= area over a live game; dose-off == control ==
  shipped).
- OFF-identity: **239/239 byte-identical** control == shipped picks
  (scoutbase + control self-play lines).

Dose-response (league AVG vs scoutbase, skips 0/10/20/30 x 2 colors,
control measured in the same runs):
| REINFORCE_W | league AVG | h2h dose-vs-control | gauge | v1 | survival pooled dose/ctrl |
|---|---|---|---|---|---|
| 0 (control) | **-3.6%** | — | — | 5/10 | 7.3% / — |
| 0.1  | -8.2% | 6/10 (B 4/5, R 2/5) | — | — | — |
| 0.25 | -8.2% (worst -58.7%) | 6/10 (B 4/5, R 2/5) | 8/8 | 6/10 vs 5/10 | **20.7% vs 6.7%** |
| 0.5  | -16.8% (worst -111.2%) | 5/10 (B 3/5, R 2/5) | 8/8 | 6/10 vs 5/10 | **21.6% vs 7.3%** |

Mechanism confirmed: the term does exactly what it was designed to — pooled
survived/(survived+popped) triples (20.7-21.6% vs control 6.7-7.3%), v1 h2h
improves (6/10 vs 5/10), gauge holds 8/8. But league is monotone-negative at
every positive weight: 0 -> -3.6%, 0.1 -> -8.2%, 0.25 -> -8.2%,
0.5 -> -16.8%. No weight beats control on h2h + league jointly. KILL per the
pre-stated discipline — the v7 E-family lesson reproduced with a measured
form: scoutbase converts the expansion tempo the protection costs into more
banking than the protection saves ("the h2h pass does not travel").

## Post-mortem pointers (for a future lane; NOT this lane's scope)

- The damage is concentrated in the OPENING rows: skip0 dose-blue -56.4% /
  -54.0% / -63.4% vs control -22.3%; skip0 dose-red -43.7%/-41.2%/-111.2%
  vs control -4.5%. Mid/late rows IMPROVE at 0.1-0.25: skip10-red +34.7pp,
  skip20-blue +22.6pp, skip20-red +26.5pp, skip10-red +14.2pp (W=0.1). A
  phase-gated variant (term armed only after ~action 20-30, or only when
  events_left < some bar) is a NEW one-variable dose for a new lane with
  its own kill-0 + gates — do NOT stack it on this kill.
- Corpus readout (24 recorded games: 12 v7-era + 12 v8-rated, fetched
  read-only from lane-v8-autopsy `research/games/` via `git show`; regen:
  `git show origin/lane-v8-autopsy:research/games/<id>.json > <dir>/<id>.json`,
  probe `corpus <dir>`): winner survived-share 43.9% vs loser 22.2%,
  winner durable-share 0.929 vs loser 0.822. Our v8 losses sit at ~10-30%
  survival vs VladNet 50-62% and GB 57-73% in the same games (autopsy
  baselines 5.3%/50-55%/88-90% hold directionally). The winning shape is
  still the right target — this dose's payment mechanism was wrong.

— Lane REINFORCE (v9 item 4), 2026-10-03
