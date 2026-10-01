# PROT-BLOCK kill-0 — KILL

Dose: MESH_B_SEIZE table (D10-G10, G10-J11, D10-G13, G13-J11) behind
Red-mesh-entry trigger (any Red node within Chebyshev 2 of G11);
rule 6 denies any Red pocket entry once 3 entries were played (cap at 3).
Blue plays the next unplayed seize edge while armed, else greedy by area.
Red adversary: mirror-mesh prefix (`search::mesh_prefix`) while available,
else pocket-seeking greedy (Chebyshev to G11, tie by area, index),
tiebreak rotated by game index.

Harness: `retaliator/examples/prot_kill0.rs` (NEW file, no src changes).
Run: `cd retaliator && cargo run --release --example prot_kill0`
Control (no seize, no rule 6): `PROT_CONTROL=1 cargo run --release --example prot_kill0`
Log: `retaliator/gate-logs-prot.txt` (dose), control rerun same binary.

## Results (9 games, deterministic)

| game | denials (dose) | red@16 (dose) | red@16 (control) | blue@9 (dose) |
|------|---------------|---------------|------------------|---------------|
| 0,3,6 | 448 | 17.5 | 17.5 | 0.0 |
| 1,4,7 | 829 | 17.5 | 17.5 | 0.0 |
| 2,5,8 | 762 | 17.5 | 17.5 | 0.0 |

Bars: 9/9 games deny the 4th entry (PASS — denials fire in all 9) AND
Red@16 <= 10.0 (FAIL — 17.5 in 9/9) AND Blue exposure@9 <= 4.5 (PASS, 0.0).

## Verdict: KILL

1. Red@16 misses the bar by 7.5 points in every game.
2. Decisive control comparison (added by coordinator): control red@16 is
   byte-identical 17.5 in all 9 games. Hundreds of rule-6 denials per game
   change Red's move choices but move no measured bar — the machinery is
   theater against this adversary. Zero delta vs control, so no
   conditional stack is justified.
3. Caveats (do not overturn): only 3 unique lines among the 9 games
   (tiebreak rotation yields 3 patterns); blue@9 bar passes trivially
   (nothing built by action 9 either side). Neither affects the verdict
   since the failing bar is identical with and without the dose.

Do not build MESH_B_SEIZE. Per roadmap, MIRROR-ORACLE is now the live
Blue fallback (ONLY if PROT-BLOCK dies — it has).
