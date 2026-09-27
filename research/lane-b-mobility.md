# Lane B: mobility / potential-area term for early expansion — measured, SKIPPED (2026-09-27)

Branch: `lane-b-eval`. Probe: `retaliator/examples/probe_b_shape.rs` (read-only
over the engine; no eval terms). Decision discipline: plan-v4 V4d ("one probe,
one decision — report and move on").

## Claim under test (task item a)

"Bot doesn't claim space early; GB walls 12 actions then banks ~10/loop" —
motivation for a mobility / potential-area term rewarding early expansion
("long sticks").

## Measurement (new numbers)

`probe_b_shape`, line 1: shipped-search self-play from the empty start
(watched Blue), per-action kind / length / area gain / hull growth / mobility:

```
self act=0  extend len=3 gain=+0.0  hull 0.0->4.5    (d+4.5)  mob=69
self act=3  extend len=3 gain=+0.0  hull 4.5->9.0    (d+4.5)  mob=151
self act=4  close  len=3 gain=+9.0  hull 9.0->9.0    (d+0.0)  area_now=9.0
self act=7  extend len=3 gain=+0.0  hull 9.0->16.5   (d+7.5)
self act=8  close  len=3 gain=+4.5  ...             area_now=13.5
self act=16 close  len=3 gain=+9.0  ...             area_now=27.0
self act=31 extend len=3 gain=+0.0  hull 67.0->83.5  (d+16.5)
self act=39 extend len=3 gain=+0.0  hull 111.0->132.0 (d+21.0)
self FINAL blue 2073-1294 area 35.5/13.2
self-play Blue: first_close=act 4 closes=22 cuts_taken=42
```

Full trace closes: +9.0, +4.5, +4.5, +9.0, +4.5, +4.5, +9.5, +6.3, +6.3,
+13.5, +2.0, +13.1, +5.0, +13.5, +9.2, +9.0, +5.0, +9.2, +13.5, +4.5 …
**avg ≈ +7.2 area/close.**

## Findings

1. **The small-loop disease is already fixed.** First close banks +9.0 at
   act 4 (site-game v1 autopsy: 28 closes -> 7.0 area total, ≈0.25/loop).
   Current avg ≈ +7.2/loop — GB-class (GB ≈10/loop from act 13-15).
2. **The bot already plays "long sticks" and claims space early.** Hull grows
   +4.5 to +26.5 per opening action (161.5 by act 95); extend length is
   mostly 3 (matches the field: every bot ≈2.8).
3. **GB's wall book is NOT an expansion model.** Its 12 wall actions grow
   hull only +0.5 to +4.0 each (thin parallel walls); first close +2.5 at
   act 15 in the probe line. The book is separately REJECTED (-48pp league).
4. Line 2 also shows the shipped search DIVERGES from the GB book at every
   action (as expected from the rejection).

## Decision

**SKIP (a).** A mobility / potential-area term would reward what the bot
already does — the ROOM_WEIGHT 1.5 = noise precedent. No early-expansion
deficit exists in the current search to fix. Do not relitigate without new
numbers (e.g. a future engine regression changing the opening shape).

Next: (b) cut+make gap math, measured on the games that matter (as-Blue
chair vs v1 — problem #1), see `research/lane-b-gap.md`.
