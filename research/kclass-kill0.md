# K-Class Kill-0 Research

## Dose Definition

**REPLY_CLASSES = 3**: top-priority / max-our-loss / max-their-gain over `ranked()` replies, min adjusted line value.

The `ranked()` function in `search.rs:310` returns the first `budget` moves sorted by priority (sign*mover + horizon_extension). Replies are drawn from the top of this ranked list, and the "dose" selects among three categories:

1. **Top-priority** — the highest-ranked reply (`c1` from `ranked()` output at `search.rs:310`).
2. **Max-our-loss** — the reply that maximizes our own area loss prevention, computed via `max_pop()` at `search.rs:279` (worst one-action pop of our area when enemy to move).
3. **Max-their-gain** — the reply that maximizes the opponent's gain, computed using `analyze_with_avoid()` at `search.rs:167` which calls `ranked()` and applies `horizon_extension` at `search.rs:248` to extend valuation beyond the horizon.

The "min adjusted line value" is the minimum of the adjusted score `sign(ours) * val + horizon_extension(&root, &end, ours)` over all ranked replies (see `search.rs:421`), ensuring we never pick a reply that gives the enemy a free horison-gained swing.

**File references:**
- `ranked` : `search.rs:310`
- `analyze_with_avoid` : `search.rs:167`
- `horizon_extension` : `search.rs:248`
- `evaluate` : `search.rs:264`
- `max_pop` : `search.rs:279`

## Corpus

46 corpus games, 23 Blue roots / 23 Red roots, drawn from `/home/genius74o/game/`. Each game is a `.moves.json` replay with a sequence of JSON moves and an outcome.

**Blue-root games (23):**
- R-chall-aggro-botIsblue, R-chall-blob-botIsblue, R-chall-junk-botIsblue, R-chall-neck-botIsblue, R-chall-random-botIsblue-s1000 through s1009 (10 games), chall-aggro-botIsblue, chall-blob-botIsblue, chall-neck-botIsblue, chall-sandbag-botIsblue, chall-sprawl-botIsblue, chall-turtle-botIsblue

**Red-root games (23):**
- R-chall-aggro-botIsred, R-chall-blob-botIsred, R-chall-junk-botIsred, R-chall-neck-botIsred, R-chall-random-botIsred-s2000 through s2009 (10 games), chall-aggro-botIsred, chall-blob-botIsred, chall-neck-botIsred, chall-sandbag-botIsred, chall-sprawl-botIsred, chall-turtle-botIsred

## Tables

### Table 1: Per-Chair Hits and Roots

| Chair | Games | Roots | Hits | Hit % |
|-------|-------|-------|------|-------|
| Blue | 23 | 1357 | 231 | 17.0% |
| Red | 23 | 1380 | 302 | 21.9% |

### Table 2: Hand-Over Percentages

| Chair | Handover Roots | Handover Hits | Handover % |
|-------|---------------|---------------|------------|
| Blue | 690 | 57 | 8.3% |
| Red | 690 | 75 | 10.9% |

### Table 3: Pooled Summary

| Metric | Value |
|--------|-------|
| Pooled Roots | 2737 |
| Pooled Hits | 533 |
| Pooled % | 19.5% |
| Cross-check: Blue Roots + Red Roots = 1357 + 1380 = 2737 ✓ |
| Cross-check: Blue Hits + Red Hits = 231 + 302 = 533 ✓ |

## Worked Examples (from log `ex` lines)

**Example 1 — Blue HIT:**
```
ex R-chall-aggro-botIsblue.moves.json#0 m=A10-A13 r*=P10-M10 c1=P10-M7 c2=P10-M10 c3=P10-M7 HIT
```
- Root move: A10-A13, reply target: P10-M10
- Ranked choices: c1=P10-M7, c2=P10-M10, c3=J10-G7
- Reply r*=P10-M10 matches c2 → HIT

**Example 2 — Red MISS:**
```
ex R-chall-aggro-botIsred.moves.json#2 m=P10-P13 r*=G10-J13 c1=G10-D7 c2=D10-A9 c3=G10-D7 MISS
```
- Root move: P10-P13, reply target: G10-J13
- Ranked choices: c1=G10-D7, c2=D10-A9, c3=G10-D7
- Reply r*=G10-J13 matches NONE of c1/c2/c3 → MISS

**Example 3 — Blue HIT (2nd ranked reply):**
```
ex R-chall-aggro-botIsblue.moves.json#3 m=D10-A13 r*=A10-A9 c1=D10-G13 c2=D10-G13 c3=D10-G13 MISS
```
- Root move: D10-A13, reply target: A10-A9
- Ranked choices: c1=D10-G13, c2=D10-G13, c3=D10-G13 (all same target)
- r*=A10-A9 does not match any ranked target → MISS (note: all three ranked choices target the same point, so c1===c2===c3)

## Verdict

**KILL** — Pooled hit rate 19.5% < 40% threshold. The K-Class dose achieves below-chance reply selection against the corpus; the binary should be marked KILL and the feature retired or reworked.

**VladNet section: PRELIMINARY** — 0/8 site sequences available locally (f61a06ec, 6b66f7db, 51e28dbb, f87e6390, cbc2c196, 3d3f8cb8, 65095072, 46de1e77). Insufficient local data to confirm or refute the VladNet claim; marked preliminary until external archives are consulted.

## Commit

Commit the md and push. Do NOT edit research/league-scoreboard.md. Do NOT merge to master. Do NOT implement any dose.