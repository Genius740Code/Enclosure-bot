# Lane D2 — blob-style (mega-blob swallow) vs our bots — 2026-09-28

Probe: `retaliator/examples/opp_blobstyle.rs` — the C2 S1 signature
(`research/c-site-losses-2.md` §4.1 on `origin/lane-c-autopsy`: xmybot beat
v4 **7878–2929** and **4422–3603**; GB 0–19; GB2.0 130-area loop; Atlas v2's
loop lands acts 41–51; all with our eval at **+1000–2400** right up until the
close). Tournament: `retaliator/examples/probe_match_d2.rs` (round-robin vs
shipped `search::best_move` = v3, v1base, v2base, plus a v2+avoid arm;
8 games per matchup, 4 with each color, the same verified solo jitter and
move-index tiebreak as Lane D's `probe_match_opp`). Log:
`logs_lane_d2_blob.txt`.

## The mimic

- **0 area for 30+ own actions**: draws one giant ring as an open polyline
  (an open path encloses nothing) in the FAR band — Blue walls off the west
  x∈[−9,−3] (108 area, GB's site loop size, 9+ from Red's start), Red the
  east mirror. 16 book edges, closing Connect at own action ~16-17
  (engine ~act 33) — the same "build silent, then swallow" arc as
  xmybot (0.0/1.4/2.6/3.6 area at acts 10-50 → 180.3 at act 60).
- **Never closes small**: the only closes are the book rings; the fallback
  (Scout's search) refuses any close under 25 area and takes the smallest
  one only when every candidate closes small.
- **Hold = repair + thicket**: every action re-places the first missing ring
  edge (rule 6 makes enemy cuts arrive a turn late — their cutting edge
  shields the slot, so repairs land one action later). A blocked slot (enemy
  node anchored on the line with 2 edges) gets an anchor-clearing cut first.
  A **partner wall** (parallel polyline one step outside the enemy-facing
  wall) makes wall cuts 2-touch illegal, and a lone partner cut drops nothing
  — the invincible-thicket hold of C2 §4.2a.
- **Growth rings** (xmybot's 180→257 acts 60-80): ring2 pushes the far wall
  to the midline (162), ring3 across it (216), each gated on the previous
  layer actually banking (≥80/100/170 area), so a contested mimic fights
  instead of growing dead layers. Layered cuts only lose the outer strip
  (ring1 stays behind the partner), the layered-thicket grind of the GB
  family.

## Results (n=8 both colors per baseline; margin from OUR perspective)

| Matchup | W-L | avg margin | avg score diff |
|---|---|---|---|
| v3 vs blob | 8-0 | +31.3% | +904 |
| v1 vs blob | 8-0 | +36.0% | +893 |
| v2 vs blob | 8-0 | +38.8% | +1027 |
| v2+avoid vs blob | 8-0 | +31.7% | +868 |
| **combined** | **32-0** | **+34.5%** | **+923** |

## Action-numbered pattern (probe traces, blob's own actions)

- Build phase is clean: own actions 1-16/17 are all extends at 0.0 area —
  even the most contest-happy baseline leaves the far band alone while the
  ring is an open polyline. The giant Connect lands at own 16-17
  (first_close column: 16-17 in every game; engine act ~33).
- The war starts the moment area appears: from the close onward our
  baselines land ~24-33 cuts/game on the ring (their ambient contest rate —
  the same 25-35 background volume they cut ANY style with). Each cut opens
  the ring; the shield rule delays the repair one action, so ~1 scoring
  event per cut is lost.
- **Duty-cycle economics (the whole matchup)**: ring held = 108 bank/event;
  our baselines hold 45-83 area continuously for ~55 events. The blob only
  wins that race at ~65%+ duty cycle. Measured: cuts arrive ~0.5-0.7/turn
  ⇒ the ring is open at roughly every other enemy-turn-end scoring ⇒ blob
  banks ~50-60% ⇒ 108 × 43 × 0.55 ≈ 2500 vs our ~2800-3300. Final blob
  areas: median 5.8, max 114; our margins +181 to +1562. In the games where
  two repairs landed between cuts (traces: area back to 107-126 at own
  38-40), the margin shrank to +181/+298 — the sensitivity Lane B's term
  needs to price.
- The partner wall never materializes in contested games: ring1 repairs
  have scan priority, so once the war starts every action goes to
  repair/anchor-clearing (partner 13-15 edges missing at game end). The
  thicket hold only completes vs a passive opponent — which is exactly the
  site condition (site-us cut xmybot **2 times** in 120 actions, local-us
  cuts 24-33 times).

## What this probe proves

1. **The S1 signature is stoppable by contesting — and our baselines DO
   contest.** Every cut costs the blob ~one scoring event of its full
   area; 25-30 cuts = −2700-3200 bank, which is the whole margin. The site
   losses happened because deployed Riposte NEVER ROUTED to the loop
   (C2: "the loop boundary is simply out of reach... and we never routed
   toward it") — site-us tried 2 cuts where local-us lands 30. The swallow
   is a routing failure before it is an eval failure.
2. **The eval blindness is still real and now measured locally**: while the
   ring is open (area 0), our baselines' eval reads +300-1000 (their own
   area × HORIZON) and nothing in the search prices the enemy web that is
   ONE edge from enclosing 108-216. No baseline made any move toward the
   far band until the close actually landed — the build phase was untouched
   in every game. H4's term would have to be paid BEFORE own action ~16.
3. **Depth cannot see the close** (C2 §4.1 confirmed): v1/v2/v3 are 2-ply;
   the closing move is 8-16 actions out of the tree at the moment the eval
   peaks. This is B's term, not A's depth.

## Hypothesis for Lane B (the eval term this style demands)

- **H-B-D2-BLOB1 (enemy closable-potential, the local form of C2 H4).**
  Add a term: for the ENEMY's open web, estimate the area its best single
  close would enclose (one movegen scan for THEIR max-gain close, like
  `max_pop` but for their close) times a closure-progress factor (share of
  the loop's boundary already drawn), priced × min(events_left, HORIZON).
  **Confirm target:** in this probe's traces at blob own action 15 (one
  edge from closing 108), our eval must read ≤ 0; today it reads +300-1000
  and we make no routing move. Pricing check: the term must exceed our own
  area × remaining events at that node, i.e. ~108 × 40 × (1/16 progress)
  scaling, or a flat "their room × their events" floor.
- **H-B-D2-BLOB2 (routing is the cheaper half).** The local tournament
  shows contesting beats the swallow outright (8-0 sweeps at every ring
  geometry tried: midline 18-edge, far-band 16-edge, far-band + partner
  wall — margins +30-44% throughout). If B's term is expensive, a cheap
  intermediate is a *routing* rule: when enemy room (convex hull of their
  nodes) is ≥ ~2× ours and their area is ~0 for 10+ actions, rank moves
  that march into their hull. Confirm: at blob own action 10-15 our bot
  places ≥1 node inside the far band and the ring never closes.
- **For Lane A:** confirm depth-3 still finds nothing at blob own action
  15 (the close is 1-7 actions out but the eval peak is earlier) — if so,
  the fix is B's term alone (same split as C2 H4/H5).

## Caveats

- The mimic never contests OUR ground, so our baselines bank unopposed —
  the real xmybot games had our area capped at ~60 by its incidental
  presence. A blob+farmer hybrid would be strictly scarier; that is the
  sac/longfarm probes' job.
- Smoke reference: vs `scoutbase` (floor-0 rush, our hardest style per
  Lane D) the blob loses 1724-2589 as blue and 1554-1726 as red — the
  ring closes at own 16 in both, then loses the bank race at ~55% duty.
