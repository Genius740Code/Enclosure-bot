# findings-ideas.md — IDEAS / ADAPTIVE-BOT track (chess-programming techniques)

## TOP-LINE SUMMARY (FINAL — all matches complete)
- Variants built (all NEW files, locked files untouched): `botX-quiesce.js` (ID + ordered 2-ply + opponent-reply quiescence on volatile finals), `botX-ttable.js` (transposition/state-hash cache), `botX-crit.js` (variance-based criticality time control), `botX-ponder.js` (ponder prediction/validation prototype + design).
- HEADLINE: all three playable variants went 3W-3L vs `bot-tourney.js` over 6 alternating-color games, every win as Red / every loss as Blue — i.e. color decided everything, no variant differentiated from baseline. NOTHING meets the bar for a merge recommendation.
- Quiesce 3-3 (+280.4, variance-dominated); ttable 3-3 (−327.1, 0/115 cache hits — dead weight); crit 3-3 (−489.9).
- Ponder probe: 100/174 exact-reply hits (57.5%) vs same-family opponent = upper bound; stays design-only (no harness hook exists).
- Recommendation to tournament: keep `bot-tourney.js` as-is. Only follow-up worth time (not now, mid-tournament): quiescence penalty weight sweep (0.5 untested vs alternatives) or break-threat-specific reply search, since ordering probes show midgame evals are mostly flat zeros — the eval function itself, not the search, is the bottleneck.

## 1. Iterative deepening + move ordering + quiescence (`botX-quiesce.js`)
Design: depth-1 full 1-ply sweep locks fallback; depth-2 deepens 12x30 (vs tourney 10x25) with improved `quickScore2` (adds banked-score delta + opp denial); quiescence stage re-scores top-4 finals, extending ONLY volatile ones (break occurred or area changed) with one bounded opponent-reply search (nodeCap 7, ≤40 replies), penalty 0.5×reply. Quiet finals stand pat.
Result: 3W-3L-0D, avg margin +280.4 — but all wins as Red, all losses as Blue (color-decided); mirror games byte-identical to tourney lines. Ordering probe: quickScore2 ranks identically to quickScore midgame. Verdict: NEUTRAL, do not merge.

## 2. Transposition / state cache (`botX-ttable.js`)
Design: canonical hash over sorted segment sets + turn + actionsRemaining; eval cache + second-ply dedupe set.
Probe: 0/115 hits midgame — different first moves essentially never transpose at these widths, and `singleActionCandidates` already dedupes identical (from,to).
Result: 3W-3L-0D, avg margin −327.1, same color-decided pattern; game 3 byte-identical to tourney mirror line. Verdict: NEGATIVE-TO-NEUTRAL (dead-weight hashing), do not merge.

## 3. Pondering (`botX-ponder.js`) — design + prototype, no harness code
Design: on our turn, cache predicted opp reply (their quickScore-best 1-ply) + our pre-searched response; validate by state hash next turn. Worker-thread protocol sketch in file header (post predicted state to worker, get response back, validate-or-discard). Prototype exports `predictOpponentReply / preSearchResponse / validate / ponderCycle`.
Result: 100/174 exact-reply hits (57.5%) in tourney-vs-tourney games — but predictor and opponent are same-family (both quickScore-based), so this is an UPPER bound; vs a foreign opponent it will be lower. No harness hook exists for between-turn compute, so this stays design-only. Verdict: DESIGN-ONLY, do not merge.

## 4. Variance-based time control (`botX-crit.js`)
Design: after cheap 1-ply sweep, `crit = 1 - gap/5` clamped [0,1]. crit<0.2 → narrow (5x12) + 35% time cap (move fast on clear-best); crit>0.6 → widen (14x32) full budget; else tourney widths. Returns `_crit/_gap` for logging.
Result: 3W-3L-0D, avg margin −489.9, all wins as Red / all losses as Blue; games 2/3/5 hit the same 1001.9-vs-3705.5 mirror line as baseline. Criticality fired (midgame gap=0 → crit=1.0 → widened search) but wider search of flat-zero evals changed nothing. Verdict: NEUTRAL, do not merge.

## Match log (A-variant vs B=bot-tourney.js, alternating colors, budget 400ms)
Method note: time-boxed search is timing-sensitive, so parallel background matches add noise (same-color games not byte-identical). Color edge dominates (Red wins big either way); the signal is whether the variant wins its Blue games or shifts mirror-line scores.
| variant | games | W-L-D (variant) | avg margin (variant) | notes |
|---|---|---|---|---|
| quiesce | 6 | 3-3-0 (all W as Red, all L as Blue — color-decided) | +280.4 (variance-dominated, ±900–2700) | NEUTRAL — mirror games byte-identical to tourney lines |
| ttable | 6 | 3-3-0 (color-decided) | −327.1 | NEGATIVE-TO-NEUTRAL — 0 cache hits, dead weight |
| crit | 6 | 3-3-0 (color-decided) | −489.9 | NEUTRAL — widened search over flat evals changes nothing |
| ponder(base) | — | design-only | — | 57.5% reply-prediction hit (upper bound, same-family opp) |
