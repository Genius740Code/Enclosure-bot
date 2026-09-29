# C2 loss classification — 53 site losses, autopsy.rs + analyzer (2026-09-29)

Method: `cargo build --release --example autopsy` in `retaliator/` (this
worktree, read-only — `git status` clean, no `src/*` touched). Ran
`target/release/examples/autopsy` on all 4 new riposte-v4-vs-capybara
losses (446956a1, b4dd04bf, 76074a41, a4b5e81b; transcripts
`/tmp/opencode/c2-*-autopsy.txt`, outside repo) — all exit 0, all show the
same shape: our 4.5-chains popped within 2-4 actions, opp breaks every
2-3 moves from act ~21 on. Per-action JSONs via the out-of-repo analyzer
crate (already path-deps into this worktree) + `/tmp/opencode/c2-classify.py`
(outside repo). Corpus: 49 prior `an/*.json` + the 4 new = **53 losses**.

Primary label (priority: mega-blob > farmed-reclose > wasted-cuts >
blue-chair-fade > missed-close > attrition):

| primary type | n | avg margin (our persp) | worst |
|---|---|---|---|
| mega-blob (opp peak area >= 90) | 26 | -1888 | -4949 (xmybot a3f5f01c, peak 256.9) |
| farmed-reclose (opp combi >= 8 or our farmed chains >= 8) | 27 | -496 | -1429 |
| wasted-cuts / blue-chair-fade / missed-close / attrition as PRIMARY | 0 | — | — |

Secondary tags (overlap — the real picture; every loss has 2-4 tags):

| tag | rate | per-type damage signal |
|---|---|---|
| farmed (our close popped within ~3 actions >= 8) or opp combi >= 8 | 51/53 (96%) | the base disease; avg 14.4 farmed chains + 8.2 opp combis in primary-farmed set |
| standing big opp close (gain >= 12 never cut below 50%) | 44/53 (83%) | missed-close opportunity present almost everywhere |
| wasted cuts >= 40% (of games with >= 10 cuts) | 31/51 (61%) | overall waste 41% (prior hand-count 47% — same ballpark, method differs) |
| blue led-then-lost (led after act 12, lost) | 24/28 blue losses (86%) | blue-chair fade is the DEFAULT blue loss shape, not a subtype |
| opp peak >= 90 | 26/53 (49%) | avg opp peak 137 in blob set; separates -1888 (blob) vs -496 (farmed) |

Read: **priority order hides that wasted-cuts + missed-close + blue-fade
co-occur with farming rather than competing with it.** Per-type damage ranking
by avg margin: mega-blob (-1888) >> farmed-reclose (-496) > close
losses (-1..-164 tail: f9c819ed -5, 4bf39c48 -1 — both farmed-primary with
>=16 farmed chains but low opp peak; the grind tail where B-3 form A matters).

The 4 new capybara-v4 losses: all farmed-primary (margins -242..-467,
farmed 11-17, waste 9-19/30-37, opp peak 32-47 — low-tempo farmer, no blob).
Capybara wins WITHOUT a mega-loop: 15-17 farmed chains at 4.5 each is enough.

Flags:
- 3 duplicate feature-signatures (6 games: 4975d4e6/02023cd5,
  276ab448/bb4dc0dd, 0ac7f8b4/f76a0178 — identical margin/color/peak/combi/
  farmed/waste/cuts). Distinct game ids; likely same-line rematches. Prior
  "0 duplicates" claim was byte-identity of move lists — check whether these
  pairs are byte-identical; if so the dedup check missed them.
- e6bf571b (margin -820, opp peak 219, only 1 our-cut): outlier shape —
  single huge loop, we barely interacted. Possible timeout/forfeit-adjacent
  game; exclude from dose tuning sets.
- a3f5f01c (-4949, xmybot peak 256.9): damage ceiling reference for blob
  insurance terms, but n=1 shape — do not overfit Q-thresholds to it.
