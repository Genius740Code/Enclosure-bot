# Lane B: legal-cuts-only vulnerability term — ablation report (2026-09-27)

Branch: `lane-b-eval`. Harness: `retaliator/src/eval_phases.rs` (shipped
`search.rs` skeleton copied verbatim + ONE gated term), probes
`probe_b_h2h` / `league_b` / `gauge_b` / `probe_b_v1`.
Iron rules held: all term units are full-horizon points (area × min(events,12));
legal cuts only via the engine's legal move set (V4d2); deterministic tiebreak
by move index; ablation control = `baseline_best_move` (term OFF).

## Step 1 — Faithfulness (harness must match shipped search first)

`probe_b_h2h` compares `eval_phases::baseline_best_move` (term OFF) with
`retaliator::search::best_move`, position for position, over two full
self-play games (empty start + forced opening 5589):

```
faithfulness: 0 mismatches in 239 positions (control vs shipped)
```

**PASS.** The skeleton copy has not drifted; games below measure the term,
not drift.

## Step 2 — Ablation: term ON (VULN_W=1.0) vs OFF (VULN_W=0.0)

n=8-10 both colors per gate; all bots deterministic (one run each is exact).
Three independent confirmations that OFF == shipped search: (a) faithfulness
probe 0/239 positions, (b) league OFF margins reproduce the scoreboard
baseline `-20.0%` exactly (every row: +29.4/+38.1/+29.4/-111.2/+29.4/-111.2/
+47.9/-111.4), (c) v1-h2h OFF games match the baseline `v3 vs v1` scores
position-for-position (1322-786, 1243-1237, 1256-1745, 1218-1219, 1637-1806,
1343-1259, 1415-2120, 1541-1578).

| Gate | Term ON | Term OFF (control) | Delta |
|---|---|---|---|
| v1 h2h (10 games) | **3/10** — as blue 1/5 avg -24.8%, as red 2/5 avg -7.3% | **4/10** — as blue 2/5 avg -11.6%, as red 2/5 avg +3.6% | **worse**: -1 win, as-blue -13.2pp |
| league vs scoutbase (8 games) | **-25.5%** (blue +26.5/+8.5/+26.5/+55.7; red +8.5/-120.7/-120.7/-105.9) | **-20.0%** (baseline exact) | **worse** by 5.5pp |
| gauge vs greedy (8 games) | **8/8** (blue +54.7%, red +69.0%) | **8/8** (blue +66.0%, red +69.3%) | same W/L; blue margin -11.3pp |

ON-arm game detail (v1 h2h): the as-Blue chair got WORSE — open=5589 blue
-53.3% (ON) vs -39.0% (OFF); open=4864 blue -38.0% vs +0.5%. Blue chair vs v1
is problem #1; the term moves it the wrong way.

Collapse line (the term's stated target — needs depth or vulnerability
pricing, `plan-v4.md` Outcomes): NOT fixed. league skip=10/20 red -111.2%
(OFF) -> -120.7% (ON) — WORSE; skip=30 red -111.4% -> -105.9% — mixed. The
term does not convert the synthetic 30+-pop line.

### Why it fails (mechanism read, from the numbers)

The erosion gap prices cuttable area at full horizon weight on EVERY
candidate. Baseline cut rate is ~25 cuts/side/game (`rival-analysis.md` §5)
— cuttable area is NORMAL play, not an emergency. So the term effectively
discounts all big-area claims by their cuttability and pushes the bot toward
small/dense shapes — exactly the small-loop habit that loses to v1
(problem #1). As Blue vs v1 the effect is worst (-13.2pp avg margin).

## Verdict

**REJECTED (worse on every gate).** `VULN_W` left at 0.0 in
`retaliator/src/eval_phases.rs` with provenance comment: the harness now
carries ONLY verified terms, so it stays a clean single-term rig for the next
candidate (mobility / potential-area). 0.5 dose untested — trend uniformly
negative across three gates, dose unlikely to flip it. Do not relitigate
without new numbers.

## Term-ON reference: head-to-head vs shipped search (context, not the gate)

`probe_b_h2h`, 10 games (5 openings × 2 colors), term ON vs shipped:

```
laneB vs shipped: 4/10 | as blue 1/5 avg margin -13.5% | as red 3/5 avg margin -19.7%
```

The red-side open=None line (-123.8%) is the known catastrophic synthetic
line (single 30+ area pop ~t=60 + erosion; `league-scoreboard.md` baseline).
