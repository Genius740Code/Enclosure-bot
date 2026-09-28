# Lane D — Scout-style (scoutbase, floor 0) vs our bots — 2026-09-27/28

Probe: `retaliator/examples/support/scoutbase.rs` used directly (the
established Scout-class baseline opponent — the control arm). Tournament:
`retaliator/examples/probe_match_opp.rs` (8 games per matchup, 4 with each
color, solo jitter, tiebreak by move index).

## Results (n=8 both colors per baseline; margin from our perspective)

| Matchup | W-L | avg margin | avg score diff |
|---|---|---|---|
| v3 vs scout | **5-3** | +0.2% | +22 |
| v1 vs scout | **4-4** | +6.4% | +104 |
| v2 vs scout | **4-4** | **-4.1%** | **-31** |
| **combined** | **13-11** | **+0.8%** | **+32** |

**The rush-closer style (close floor 0) is the MOST dangerous style to us** —
near even against every baseline, the only style that takes games off all
three. The style gradient across close-size floors (Scout 0 → +0.8%, Angel 3
→ +6.9%, Vlad greedy → +19.2%, GB 8 → +59.1% avg margin for us) says: the
more the opponent pops tiny loops, the worse we do.

## Action-numbered pattern of how it beats us

- Scout closes first at own action 2-3 and keeps popping (16-30 closes at
  ~0.7-2.9 area/close, cuts 28-41x/game — background volume).
- In our 11 losses: close COUNTS match (ours 19-29 vs scout's 16-30) but the
  area per close decides it — ours ~0.3-1.5 vs scout's ~1.3-2.9. We burn the
  same actions on closes that bank half as much.
- The blowups are the known collapse lines: v2 g4 **-855** (scout 28.7 area
  on 10 closes vs our 7.9 on 27), v2 g5 -661, v3 g5 -426. A single 30+ pop
  mid-game plus erosion — the scoreboard's "needs depth or vulnerability
  pricing" note.
- Color split: 5 losses as Blue, 6 as Red — the style hurts both colors.

## Hypotheses for Lane A (search)

- **H-A-SCOUT1 (see the farmer's pop inside the tree).** The -855/-661
  collapse lines have scout holding 28-44 area on 10-26 closes vs our 8-17 —
  its pops compound while our 2-ply cannot see past the horizon. Lane A's
  `search_deep` (3-ply) should catch the mid-game 30+ pop: re-run this
  tournament with depth and check the blowup margins disappear (target: no
  loss worse than -300).
- **H-A-SCOUT2 (the close-count race is winnable on area).** Close counts
  match in every loss; the area per close is the gap. The shipped search's
  `repeats_a_connection` dedup + longest-first ordering is already
  Scout-shaped — the difference is our EVAL picks tiny closes. Re-run with
  Lane B's close-size weight (see lane-d-angel.md) and check our
  area-per-close rises above scout's.

## Hypotheses for Lane B (eval/phases)

- **H-B-SCOUT1 (close-size weight — same as H-B-ANGEL1).** The v2 patience
  penalty (sub-2.0 before action 12, flat, opening-windowed) never fires
  (V5-1); the shipped anti-rebuild routing alone only flips one game
  (v2+avoid 5-3 but avg margin still -2.7%). Replace with a continuous
  per-close weight: close priority scaled by the loop's gained area ×
  scoring events left, all game. Validate vs scout-style n=8 both colors:
  target ≥ 6-8 W with no collapse line.
- **H-B-SCOUT2 (banked-area × time accounting).** In the losses scout's
  cumulative banked score outruns ours despite equal close counts — its
  closes bank from action 2-3. Test an early-phase banked-area weight (see
  lane-d-angel.md H-B-ANGEL2) so early banked area compounds for us too.
