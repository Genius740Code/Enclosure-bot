# GB-tell threshold (external session, 2026-09-30) — code 25%, window ≥20

Validated 12/12 (scores/areas/counts exact; codec 1440/1440; `kind:"connect"`
re-derived geometrically 486/486). Per-side rates (connects/60): GB
21.7/16.7/23.3 (mean 20.6, sd 3.5pp); Stompy 28.3/25.0; VladNet 31.7/30.0;
v7 23.3–53.3 (sd 10.5pp); v6 33.3–56.7.

## Confound found: low rate = v6-style evidence, not GB identity

v7's own rate depends on opponent: 41.1% vs GB, 42.5% VladNet, 44.2% Stompy,
24.3% vs v6. The GB band (16.7–23.3%) sits INSIDE the v7-vs-v6 band
(21.7–31.7%). 4/5 false positives are v7-vs-v6 sides — systematic matchup
effect. Reframe: LOW-CLOSER detector ("GB or v6-style"), not "GB
identification", or gate on opponent identity.

## Recommendation: `opp_closes/opp_moves < 0.25`, fire at opp_moves ≥ 20

- 25% vs 27% identical on sample; 25% tighter, sits on GB-max/Stompy-min gap;
  strict `<` excludes Stompy at exactly 15/60. Recall 100%, precision 37.5%
  (75% ex-v6). 23% misses GB 2.0; 20% catches 1/3; 30% pulls VladNet.
- Window: separation GB−Stompy decays 18.3pp@20 → 6.1pp@60; v6 confound
  unexpressed early. Best cell 23%@20 (F1 80%) but drops a GB side.
- Binomial: single-side 95% CI ±10.2pp vs 6.7pp band — soft prior, not a
  decision. n=3 GB games. Ratio/margin discriminators checked and worse.
- Live-computable at zero cost (`state.moves[].kind`); log opp bot name with
  the flag. S2-ready.
