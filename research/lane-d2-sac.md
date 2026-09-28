# Lane D2 — sac-style (bank-sacrifice tempo trade) vs our bots — 2026-09-28

Probe: `retaliator/examples/opp_sacstyle.rs` — the C2 §3 signature
(`research/c-site-losses-2.md` on `origin/lane-c-autopsy`): site AngelBot
"lets us demolish its already-banked loops for zero score recovery, then
re-closes bigger" (`1c7b1aa1`: we land +15.0/+22.8/+37.8 demolitions and the
score gap still widens 220-386 → 924-471; `4929b951` is the pure form: both
sides at ~0 area, our 45 cuts pop real area, it wins on the bank 344-180).
Score is **cumulative**: cutting a banked loop recovers nothing, so area
already scored is sunk — the only currency is where the NEXT actions bank.

## The mimic

- **Banks early and often**: max-gain close with floor 3.0 (AngelBot's
  ~5.8/loop), first close at own action 2-3; close+cut double-duty moves
  ranked best (gain + enemy popped), matching AngelBot's 19-22 double-duty
  moves per game.
- **Never re-inflates farmed ground**: any close touching the endpoints of
  its own edges cut in the last 6 actions (the shipped wiring's avoid list,
  fed live by the harness) is refused unless it banks ≥ 10 — small re-closes
  are the tempo trap. The popped loop STANDS OPEN; our cuts of it pop
  nothing.
- **Pop beats bank when the balloon is bigger**: takes the max-pop cut
  whenever the enemy area it pops exceeds the close it would bank (never
  lets a balloon live), else banks. Real pops only (≥ 0.25 area), never a
  corpse-munching 0-pop cut.
- Fallback: Scout's wall search minus small closes and minus popped-ground
  re-closes.

## Results (n=8 both colors per baseline; margin from OUR perspective)

| Matchup | W-L | avg margin | avg score diff |
|---|---|---|---|
| v3 vs sac | **5-3** | +6.0% | +95 |
| v1 vs sac | 7-1 | +26.5% | +418 |
| v2 vs sac | 7-1 | +12.2% | +149 |
| v2+avoid vs sac | **6-2** | +10.2% | +106 |
| **combined** | **25-7** | **+13.7%** | **+192** |

**The sac-style takes 7 of 32 games — 3 of them off the SHIPPED search.**
It is the most dangerous D2 mimic so far (blob took 0 of 32) and the only
one that beats the deployed v3 more than the floor-3 angel-style did (5-3
vs angel's 6-2; avg margin +6.0% vs +8.8%).

Reference (Lane D, same protocol): the close-floor-3 angel-style WITHOUT the
sacrifice rule went 6-2 / 6-2 / **3-5** vs v3/v1/v2 and 4-4 vs v2+avoid —
v2's only sub-.500 Lane D matchup. The sac rows isolate what the
bank-sacrifice twist alone adds on top of the same floor.

## Action-numbered pattern (probe traces + tournament lines)

- Banking ladder from own action 2: close → +4 area → two extends → close
  → +9 ... the bank column compounds ~20-50 per turn-pair all game; even
  while its live area is being demolished (smoke trace: area 45 → 48 → 35
  → 30 while bank climbs 508 → 557 → 592 → 661). This is the mechanism the
  site games show at `1c7b1aa1` acts 43-50.
- **The pure bank-sacrifice game reproduced locally** (v3 g5, as red): both
  final areas ~0 (5.6 vs 0.0), a 40-41 cut war, sac out-closes 31-21 and
  wins **-179** — the `4929b951` form (site: 344-180, our 45 cuts popping
  real area, its bank already made). Our v3 spent the whole game demolishing
  loops whose bank was already scored.
- **Cut volume is vlad-class**: sac landed 34-45 cuts in all 32 games (site
  AngelBot: 37-46) — the pop-vs-bank rule makes it a banker-farmer hybrid,
  popping whenever our balloon out-sizes its next bank.
- **v2 handles sac far better than angel** (7-1 vs angel's 3-5): the
  sacrifice leaves popped ground dead, and v2 converts that open ground
  into its own farm; angel's endless re-closing is what v2 cannot stop.
  v3 inverts (5-3 vs 6-2): it corpse-munches more and banks less
  efficiently — the exact split H-B-D2-SAC1 predicts.
- **The avoid arm REGRESSES vs sac** (plain v2 7-1 → v2+avoid 6-2; the two
  losses -130/-378): the shipped rebuild penalty is keyed to OUR cut
  points only, so v2+avoid routes off ground sac has permanently abandoned
  — free to retake, sac never re-closes there by design — while sac farms
  v2's fresh closes elsewhere (loss g5: sac 43.0 area, out-closes 34-20,
  -378). The wiring cannot tell "enemy is farming me here" (vlad: avoiding
  is right, 8-0) from "enemy sacrificed this ground" (sac: rebuilding is
  free).

## Hypotheses for Lane B (the eval term this style demands)

- **H-B-D2-SAC1 (wasted-cut pricing — C2 H2, CONFIRMED locally).** The
  break/cut bonus must be weighted by the enemy area the cut actually
  UNBANKS (area that would still score) — not by the break itself. A cut of
  an already-open loop (sac's standing corpses) or of a thicket edge pops
  0 and must rank below ANY gaining move. **Evidence: v3 g5 — 40 cuts,
  enemy final area 0.0, we lose by 179**; the cuts recovered no bank
  because it was already scored. **Confirm target:** with the term added,
  v3 vs sac must drop the corpse-munch rate (cuts that pop < 0.25) below
  ~10/game and the matchup must go to ≥ 7-1 without regressing vs vlad
  (8-0 arm).
- **H-B-D2-SAC2 (avoid wiring must be enemy-conditioned — NEW, found by this
  probe).** The rebuild penalty is keyed to OUR cut endpoints and decays on
  a fixed 6-action clock; vs sac that is exactly wrong — sac's abandoned
  ground is safe to retake (it never re-closes there), so the penalty makes
  v2+avoid donate ground (6-2 vs plain v2's 7-1). The memory should decay
  only while the ENEMY keeps cutting near those points (per-edge enemy-cut
  counters, not a fixed clock) — the same mechanism as H-B-D2-FARM1 below.
  **Confirm target:** a enemy-conditioned avoid arm must keep vlad 8-0 AND
  lift sac back to ≥ 7-1.
- **H-B-D2-SAC3 (bank-race awareness — C2 H3's early form).** Because score
  is cumulative, area banked EARLY compounds: sac's bank is ~500 by own
  action 27 while farming/cutting all game. Our eval prices future area
  identically for both sides but is blind to the score column; a small
  early-phase banked-area weight (or lowering HORIZON's cap in the first
  ~20 actions) makes our search bank like sac does. Confirm: our first
  ≥ 3.0-area close must come no later than sac's (in v3's 3 losses it
  comes at action 2.3 vs sac's 2.0 — the deficit compounds from there).

## Caveats

- vs `scoutbase` smoke (floor-0 rush — not our subject): sac loses as blue
  (1437-2644) and wins as red by 40 (1234-1194) — the sac trade needs the
  enemy to spend tempo on demolitions, which a rusher never does; scout just
  out-farms the floor-3 banker.
- The mimic's avoid list comes from the harness (CUT_MEMORY 6, same as the
  deployed wiring) — good enough: the sacrifice only needs to skip the
  immediate re-close, and site AngelBot also re-cycles old pop spots later
  at bigger scale via its max-gain ranking.
