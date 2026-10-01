# Lane D2 — capybara net v5 fingerprint check (replay vs our searches) — 2026-09-28

Probe: `retaliator/examples/probe_capyfingerprint.rs` — replays capybara's
lines (Lane G's `research/g-capybara.md` on `origin/lane-g-recon`) against
our searches and records the divergence points: missed close? farmed
re-close? The raw site JSONs (`/tmp/opencode/sitegames-g/`) were wiped by a
restart, so the replay is from Lane G's recorded lines: the center-mesh
opening (Blue `D10-D13 D10-G8 G8-J10 J10-G13 G8-J11 G13-J11 G13-D10
J11-M13`, Red mirror), the no-floor banker continuation (close gains
`[1.25, 13.75, 1.25, 1.75, 1.25, 13.75…]`), and the DETECTOR-GATED farm
switch (re-cut re-closed ground only when the opponent re-closes the same
edge ≥ 2x — c-blunder-catalog §2.4, Lane G §5 + H2). Log:
`logs_lane_d2_capyfingerprint.txt`.

Arms (8 games each, 4 per color, the verified solo jitter + move-index
tiebreak): the deployed search (master's tip, the v8 stack, untimed probe
path + live avoid) and v2+avoid (CUT_MEMORY 6). The v8 site entry
(`best_move_routed`, mesh8 prefix + timed think) is NOT run — the timed
think costs 2-4.8s/move, ~1h per matchup.

## Results (margin from OUR perspective)

| Matchup | W-L | avg margin | avg score diff |
|---|---|---|---|
| v8 vs capy | **5-3** | +1.6% | +53 |
| v2a vs capy | **7-1** | | +365 |

Both arms beat the capybara style locally — a very different picture from
the site (capy 86-44, 4-0 vs our riposte v4) and from the long-farm mimic
(deployed search 1-7, see `d2-longfarm.md`).

## Divergence points (the requested check)

1. **Farmed re-close: RARE — the detector-gated farm switch mostly never
   fires.** Our re-closes of capybara's cut ground: 3-14 per game
   (v8: 5-14; v2a: 3-13), but re-closes popped by a capybara FARM-mode cut
   within 2 actions: **0-2 per game** (v8: 1, 2, 2, 0, 0, 0, 0, 0 — the
   farmed actions [52], [38+56], [49+55]; v2a: 0, 0, 0, 1, 0, 0, 1, 2 —
   [54], [39], [33+49]). The ≥2x re-close detector (c-blunder-catalog §2.4)
   requires OUR bots to rebuild the same ground twice — and the avoid wiring
   + the dead-wood/TIE-SYM terms route them away, so the switch stays off.
   **Compare the long-farm mimic: its farm is ALWAYS ON (no detector) and
   the deployed search loses 1-7.** The always-on farm is the threat; the
   detector-gated one is contained by the routing we already ship.
2. **Missed close: the close COUNT, not the timing.** Our first close comes
   EARLY (own actions 2-3, vs capybara's 2-4 — the act-9 site fingerprint
   does not reproduce locally because the mesh book forces it) — no
   first-close divergence. The divergence is mid-game: our close counts
   (v8: 15-30; v2a: 17-25) run UNDER capybara's (24-42) — capybara closes
   its best available loop every ~4-6 own-actions with NO floor, while our
   bots route to fresh ground instead of re-closing tiny (+1.25) loops. In
   the v8 losses capybara out-closes us (29-34 vs 24-27) — the no-floor
   cadence banks time into score (Lane G's temporal asymmetry) and the
   deficit compounds.
3. **The center-mesh suppression does NOT reproduce vs our current bots.**
   Capybara's largest loop stays 36-58 (no mega-loop), but that is its own
   book, not our suppression: capybara's WINS are the narrow-grind profile
   (Lane G §1) and its losses are the blowouts — locally our v8/v2a take
   12 of 16 games, so the mesh book + banker style is contained by the
   deployed evals. The site's 4-0 vs riposte v4 was the OLD bot (the
   catalog §3.2 farm fingerprint vs a rebuilder); the v8 stack's avoid
   wiring + TIE-SYM do not feed the farm switch.

## What this check answers

- **The capybara threat model is the farm switch, and it is
  detector-gated.** Our deployed bots don't feed it (0-2 farmed re-closes),
  so capybara-as-a-style is contained locally (v8 5-3, v2a 7-1). The
  dangerous form is the ALWAYS-ON farm (the long-farm mimic, deployed
  search 1-7) — a farmer that needs no detector.
- **The missed-close question resolves to the no-floor close cadence**:
  capybara's tiny re-closes (+1.25) bank time; our evals route to fresh
  ground instead. That is the same close-cadence gap as Lane D's scout
  finding (H-A-SCOUT2/H-B-SCOUT1), now with a mid-weight carrier.
- **The v8 stack vs v2+avoid inversion (5-3 vs 7-1)**: the v8 eval's
  TIE-SYM center-first tie break and doom tail weight score WORSE than
  plain v2+avoid against the mesh+banker style — the same inversion as vs
  the long farm (1-7 vs 5-3). The v8 eval's own-area × horizon dominates
  into exactly the positions where the banker/farmer styles profit.

## Caveats

- The replay is from Lane G's recorded lines (the raw JSONs were wiped):
  the center mesh + the no-floor banker + the detector-gated farm switch.
  Capybara's ACTUAL continuation is a pure-search player (Lane G §7: no
  book beyond action 1) — the mesh book here is the fingerprint line, not
  its real policy, so this is a STYLE probe, not a game replay.
- The mesh book is ALSO the v6 mesh8 prefix (Lane H shipped it) — the v8
  site entry would play the same prefix; that mirror match is untested
  (the timed think's cost).
- Action numbers are 0-based engine indices, own actions = (index + 2) / 2
  — the same convention as the other D2 probes.
