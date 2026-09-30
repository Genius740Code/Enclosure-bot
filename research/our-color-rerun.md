# Our-colour autopsy re-run — b0ac4141, 1c69056c (2026-09-30)

Re-run of the two games quarantined by roadmap Phase 0 after `30c7653b` proved inverted
(`30c7653b` fixed by `e1b82d3`). Source JSONs read from `origin/lane-v8-autopsy`; no build, no
engine change. Replayed with the vendored engine's own `examples/autopsy` (identical `vendor/`
tree; output reconciles exactly with each JSON's final `state.areas`/`state.score`).

**`e1b82d3` correction applied throughout: 4335 and 112.5 are the *opponent's* score and
bank. No number below is attributed to us on blue's column.** In **all three** games
Riposte v7 is **RED**, both opponents BLUE — verified from the `blue`/`red` fields, not assumed.
b0ac4141 = Stompy, 3208/2471. 1c69056c = VladNet, 2255/1530.

Area timeline, `blue` = opponent, `red` = us, lead = ours − theirs:

| move | b0ac4141 b/r/lead | 1c69056c b/r/lead |
|------|-------------------|-------------------|
| 20 | 27.0 / 26.5 / −0.5 | 12.0 / **20.2 / +8.2** |
| 40 | 47.0 / 46.5 / −0.5 | 60.0 / 36.2 / −23.8 |
| 60 | 59.3 / 47.5 / −11.8 | 48.0 / 30.1 / −17.9 |
| 80 | 55.3 / 44.8 / −10.5 | 33.0 / 20.9 / −12.1 |
| 100 | 75.9 / **82.2 / +6.3** | 48.0 / 22.8 / −25.2 |
| 110 | 75.9 / 31.4 / −44.5 | 49.5 / 31.9 / −17.6 |
| 120 | 75.9 / 33.3 / −42.6 | 35.3 / 19.7 / −15.6 |

**b0ac4141** — best our-lead `+6.3` @100, then 31 opponent breaks took **286.8** area, 11
catastrophic (≥10): the fatal one is **−39.7 at move 101**, which with `−9.0`, `−5.6`, `−30.0`
turns our `+6.3` lead into `−44.5` by move 110 (a **−50.8** swing). We made **zero** effective
breaks all game. Stompy's bank was genuinely unbroken: peaked 75.9, ended 75.9, gave back
**0.0**. → *Verdict: **CITABLE** — late bank-pop loss from a narrow lead; this game's only
pathological number is `−39.7 @101`.*

**1c69056c** — best our-lead `+8.2` @20 and never again. The collapse window is **moves
20→40**, not 40→60: VladNet ran 12.0 → 60.0 (+48.0) while we went 20.2 → 36.2 (+16.0), so a
`+8.2` lead became `−23.8`; in 40→60 we lost only 6.1. VladNet's bank was **not** unbroken — it
peaked 60.0 @40, ended 35.3, giving back **24.7** to our 6 effective breaks (103.8, best of the
three games). 40 opponent breaks took **179.3**; 4 catastrophic; `−28.4` @120 confirmed.
→ *Verdict: **CITABLE** — early bank-race loss (20→40) and the counter-example that opponent
banks are breakable.*

## What changed vs the bad numbers

The old per-game `B/R` area **tables were arithmetically correct**; the **Cluster 5 prose and
`rival-vladnet.md` misattributed blue's area to us**. Corrections:

1. `b0ac4141` was cited as "lead 75.9→31.4 at 110 via −39.7". **75.9 is Stompy's area, not
   ours** — our area at 100 was **82.2**, so the real arc is `+6.3` @100 → `−44.5` @110. The
   `−39.7 @101` is genuine and correctly located.
2. `1c69056c` was cited as "Move 40-60: our area evaporates" / "lead 48→30 at 60". **48 and 30
   are VladNet's.** Our area moved 36.2 → 30.1 there (−6.1, no evaporation); the real collapse
   is 20→40. The `−28.4 @120` is confirmed correct.
3. `rival-vladnet.md` still says "VladNet as Blue (1c69056c) lost area gradually but never
   collapsed". **VladNet won 2255–1530** and its area collapsed 60.0 @40 → 33.0 @80 — wrong on
   both counts; needs a follow-up edit on `lane-v8-autopsy`.
4. Residual in `e1b82d3` itself, flagged so it is not re-cited: for `30c7653b` it says "We led
   at 60" (we led at 40/50, `+47.6`/`+1.2`; at 60 we were `−64.3`) and "`−39.7` at move 101"
   (that is b0ac4141's; 30c7653b's move 101 was `−3.4`, worst `−22.8 @120`). "Unbroken all game"
   also overstates it — GB gave back 4.5 (117.0 @90 → 112.5).

**Citable now:** b0ac4141 and 1c69056c, with the per-game verdicts above. The `30c7653b`
paragraph in `rival-vladnet.md` is still not citable until item 4 is fixed.
