# Lane O — COLLAPSER Defense Ablation Table

**Baseline (skip=0, ret=Blue, 8 solos): 1-7, avg margin -11.1%**

The COLLAPSER (scoutbase, floor-0 rush-closer) reproduces the league collapse pattern:
- Closes at own actions 2-3, many small loops compounding
- 30+ pop at ~t=60 followed by steady erosion
- 7/8 losses confirm repro gate (3+/8 required ✓)

## Single-Term Ablation Results (vs baseline -11.1%)

| Defense | Term | W-L | Avg Margin | Delta | Verdict |
|---------|------|-----|------------|-------|---------|
| **CAPTURE_W** | capture exposure | 1-7 | **-31.6%** | **-20.5%** | **CRITICAL** — removing destroys defense |
| **DOOM_W** | vulnerability pricing | 2-6 | -14.0% | -2.9% | HELPFUL — prices doomed megaloops |
| **HORIZON_WEIGHT** | horizon extension | 1-7 | -7.4% | **+3.7%** | **HURTS** — overvalues future area vs rush |
| **DENSE_BONUS** | thicket density | 2-6 | -4.2% | **+6.9%** | **HURTS** — rewards dense ground collapser ignores |
| **FRESH_PENALTY** | fresh wall avoidance | 3-5 | -5.3% | **+5.8%** | **HURTS** — avoids contesting fresh walls collapser exploits |
| REBUILD_PENALTY | anti-rebuild routing | 1-7 | -11.1% | 0.0% | NO EFFECT — avoid list empty in probe |
| PATIENCE_PENALTY | patience | 1-7 | -11.1% | 0.0% | NO EFFECT — never fires (V5-1 confirmed) |
| IDLE_PENALTY | do-nothing filter | 1-7 | -11.1% | 0.0% | NO EFFECT |
| DEADWOOD_PENALTY | dead-wood discount | 1-7 | -11.1% | 0.0% | NO EFFECT |
| CONTACT_PENALTY | contact avoidance | 1-7 | -11.3% | -0.2% | NEUTRAL |
| REMOTE_BONUS | two-front play | 2-6 | -11.2% | -0.1% | NEUTRAL |

## Key Findings

1. **CAPTURE_W (3.0) is the single most critical defense** — it prices single-edge nodes (capturable) which the collapser exploits by farming our loose nodes.

2. **FRESH_PENALTY and DENSE_BONUS actively hurt vs rush-closer** — the collapser closes early (floor 0) so fresh walls are irrelevant; thicket density rewards ground the collapser never contests (it closes small loops fast).

3. **HORIZON_WEIGHT (0.5) overvalues future banking** — the collapser banks NOW (actions 2-3); our horizon extension prices area × events past 12, which the collapser's early pops make irrelevant.

4. **DOOM_W (2.5) helps but isn't enough** — it discounts doomed area but the collapser's 30+ pop at t=60 arrives after the horizon anyway.

## Recommended Adjustments for Lane B (Eval)

| Term | Current | Proposed | Rationale |
|------|---------|----------|-----------|
| FRESH_PENALTY | 1.5 | **0.0** (or negative = bonus) | Collapser exploits fresh walls; we should CONTEST them |
| DENSE_BONUS | 1.0 | **0.0** | Wasted on ground rush-closer ignores |
| HORIZON_WEIGHT | 0.5 | **0.2-0.3** | Reduce future-discounting; match collapser's now-banking |
| CAPTURE_W | 3.0 | **KEEP / INCREASE** | Critical defense vs node farming |
| DOOM_W | 2.5 | **INCREASE (3.0-4.0)** | Stronger vulnerability pricing for early pops |

## Next Steps

1. **Q3b SHAPE-BREAKING COUNTERS**: Test early contest of corridor root, pre-building to deny second wall, price-denial of bank race.
2. **Q11 PREVENT-THE-WALL**: Closure-progress trigger — when enemy live-area growth / frontier-pair count signals big close in 2-4 turns, STOP banking small stuff and contest corridor root.