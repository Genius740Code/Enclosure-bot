# Lane D — AngelBot-style (middleweight, close floor 3.0) vs our bots — 2026-09-27/28

Probe: `retaliator/examples/opp_angelstyle.rs` (Scout's search refusing any
close worth < 3.0 area — AngelBot popped ~5.8 area/loop, between the bankers
(~10) and the poppers (~1.8); no fixed book). Tournament:
`retaliator/examples/probe_match_opp.rs` (8 games per matchup, 4 with each
color, solo jitter). With Scout-style = scoutbase (floor 0) and GB-style =
floor 8.0 + book, the three-way tournament isolates the close-size threshold.

## Results (n=8 both colors per baseline; margin from our perspective)

| Matchup | W-L | avg margin | avg score diff |
|---|---|---|---|
| v3 vs angel | **6-2** | +8.8% | +193 |
| v1 vs angel | **6-2** | +13.1% | +212 |
| v2 vs angel | **3-5** | -1.3% | +26 |
| **combined** | **15-9** | **+6.9%** | **+143** |

**v2 vs angel is our only sub-.500 matchup in the round robin** — the
moderate close floor (3.0) is exactly the threshold where our bots stop
winning. The gradient across the floor (0 → +0.8%, 3 → +6.9%, 8 → +59.1%)
puts the danger zone between floor 0 and 3.

## Action-numbered pattern of how it beats us

- The mimic closes its first loop at own action 2-3 (no book), keeps ~13-26
  closes at ~0.7-2.0 area/close, cuts 26-43x/game (background volume).
- In the 5 v2 losses our closes run 19-27/game at ~0.3-1.1 area/close —
  close COUNT matches the mimic's but the area per close is half or less.
  The -615 blowup (v2 g4): mimic 27.0 area on 17 closes vs our 7.8 on 19.
- v2's patience window (sub-2.0 closes barred before action 12) does not
  close the gap: the mimic's floor-3 closes are all ≥ 3.0 from action 2-3 and
  bank from the start; our first ≥ 3.0 close comes too late to matter.

## Hypotheses for Lane B (eval/phases)

- **H-B-ANGEL1 (floor ~1.5-3.0 all game is the danger zone — test the exact
  threshold).** v2 (flat patience 2.0 before action 12) loses 3-5 to a
  floor-3.0 opponent but beats floor-0 (4-4) and floor-8 (8-0); the shipped
  anti-rebuild routing alone only flips one game (v2+avoid 4-4, +3.8%). Test
  a continuous close-size weight (not an opening-window penalty): priority
  scaled by gained area for EVERY close all game, so sub-1.5 closes never
  outrank wall extensions. Sweep the threshold 1.0 / 1.5 / 2.0 / 3.0 vs this
  mimic (n=8 both colors each) and pick the one that flips v2 to ≥ 5-3
  without hurting the gb/scout matchups.
- **H-B-ANGEL2 (phase table: early banked-area weight).** The mimic banks
  from own action 2-3 and its score compounds; our losses show our banked
  total falling behind despite matching close counts. Test raising the
  early-game weight on banked area (or lowering HORIZON's cap from 12 in the
  first ~20 actions) so the eval values immediate banked area like the
  mimic's search implicitly does.
