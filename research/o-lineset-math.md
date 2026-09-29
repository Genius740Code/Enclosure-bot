# Q7a line-set math feasibility (Lane O probe `opp_lineset.js`) — 2026-09-29

**Status: measurement only.** No eval/search changes. Deterministic
(LCG seeds 7000–7007 + deterministic enumeration order), legality via
engine only (`E.applyMove` try/catch, `E.applyTimeout`, areas read from
engine-computed state; `E.computeTerritories` used as the ground-truth
face enumerator). Worktree `/tmp/opencode/game-v7-opp`, RESULT FIELDS
BELOW filled after run.

## 1. Closed-form area-yield formula (2-line sets)

Position: own segment set `S`, mover color `c`, base area `A0 = areas[c]`.
Candidate set: two ordered legal moves `s1=(a,b)`, `s2=(d_,e)` 
(`d_`/`e` to avoid confusion with Euler's number — below: `s2=(p,q)`).

**Scope condition** (checked per sample with `E.segmentIntersect`):
both new segs attach at endpoints only — no interior crossing of own
segs, no collinear overlap, no interior touch with each other. Samples
violating scope are counted as `xskip` (out of formula scope), samples
with enemy-segment removal as `cutskip` (confounded yield).

**Prediction.** Let `G` be the endpoint-adjacency graph of `S`.
Assume w.l.o.g. the ring threads `s1` then `s2`:

```
C1 = BFS-shortest-path_G(b -> p)      (existing chain closing s1's tip to s2's tail)
C2 = BFS-shortest-path_G(q -> a)      (existing chain closing s2's tip to s1's tail)
ring = [a, b, ...C1, ...C2]  (consecutive-dedupe, closed)
P  = |shoelace(ring)|  (= E.polygonArea)
```

If either BFS fails (no single ring through both new segs — open set or
multi-face close), the predictor abstains (`ok:false`, counted as
`unpredict`, error charged as `|Y|`).

**Actual.** `Y = areas_after[c] - A0`, both read from engine state after
the two legal `applyMove`s (same turn, `actionsRemaining==2` starts).

**k=1 base case** (XBot `one_move_potential` analogue, calibrates the
machinery): `ring = [a, b, ...BFS(b->a)]`, `P = |shoelace|`.

**k=3 extension**: `s1=(a,b), s2=(p,q), s3=(u,v)` played as
turn(m1,m2) + foe `applyTimeout` (engine-legal pass, changes no segments)
+ `m3`; `ring = [a,b,...BFS(b->p),...BFS(q->u),...BFS(v->a)]`.
Two sub-cases: EXT (pair already closed, third adds) and COLD
(pair yield 0, triple yield > 0 — true 3-line close).

**Cost model.** Predictor per evaluation: scope check `O(k*|S|)` segment
intersections + BFS `O(|S|)` + shoelace `O(ring)`. Baseline (what search
pays today per leaf): `k` × `applyMove` (each runs a full `Dr`:
`O(|S|^2)` all-pairs intersections + face walk) + one after-`Dr` for the
area read. Measured below as mean µs `tPred` vs `tDrAfter` (one full
`computeTerritories` on the after-state).

## 2. Results

Main run (`opp_lineset.js`, seeds 7000–7007, first 30 turn-starts/game,
quotas met after scanning **20 positions**; raw JSON
`/tmp/opencode/lineset-run3.log`, samples `/tmp/opencode/lineset-diag.log`).
`exact` = `|P-Y| ≤ 1e-9`. `unpredict` = single-ring BFS abstentions
(error charged as `|Y|`). Out-of-scope (interior touch) and
enemy-cut samples are excluded from `n` and counted separately.

| k | n (≥ quota) | exact | mean \|err\| | p50 | p90 | max | unpredict |
|---|---|---|---|---|---|---|---|
| 1 (Y>0 only, diag) | 60 (≥50 ✓) | **37/60 (62%)** | — | — | — | **5.7** | — |
| 2 | 65 (≥50 ✓) | **5/65 (7.7%)** | **1.49** | 1.5 | 2.5 | **4.5** | 56/65 |
| 3 (EXT+COLD) | 28 (≥20 ✓) | **3/28 (11%)** | **2.36** | 2.5 | 4.5 | **7.0** | 25/28 |

Error does NOT stay bounded with k: mean 1.49 (k=2) → 2.36 (k=3),
max 4.5 → 7.0. Areas involved are small (typical Y 1.5–4.0), so these
are 40–100% relative errors, not rounding noise.

Diag decomposition of 60 in-scope closing pairs (`opp_lineset_diag.js`):
joint single-ring matches **5**; `m1`-closes-then-`s2`-ring-matches 28;
`s2`-ring-in-post-`m1`-context-only 19; neither 8.
I.e. **47/60 (78%) of pair yields decompose sequentially** —
`Y = Y1 + ring(s2 | post-m1 context)` — while the joint formula fires
almost never. Search already reads exact mid-turn areas from each
`applyMove` for free, so joint math buys nothing where it matters.

Scope coverage (deterministic counts, same sweep): of all closing pairs
found, in-scope 60 vs `own-interior-cross` 232 vs `sib-t-junction` 22 —
**the endpoint-only formula covers ~19%** of real 2-line closes
(singles: 60 vs 210, ~22%). The engine allows crossing your own edges
mid-segment, and random/greedy play does it constantly; any Lane S
implementation would abstain 4 times out of 5 before accuracy even
enters. (BFS-shortest ≠ face boundary also bites inside scope:
k=1 positives only 37/60 exact, max err 5.7 — a shorter chord across a
big loop returns the wrong shoelace.)

Representative samples: `rnd0t0 Y=1.50 P=1.50 ring=true` (textbook U-close
works); `rnd0t2 Y=1.50 Y1=1.50 Y2|1=0.00 singleRing=ABSTAIN` ×8 (second
move dead weight — set eval must handle subset closes); `rnd0t3 Y=4.00
P=4.00` next to `Y=2.50 P=0.00` (same position, formula hit-or-miss).

## 3. Cost (same-machine, same-run means; shared-box wall clock — indicative)

| k | mean tPred (scope+BFS+shoelace) | mean tDrAfter (one full Dr) | ratio |
|---|---|---|---|
| 1 | 40 µs | 66 µs | 0.6× |
| 2 | 2037 µs | 453 µs | **4.5× (predictor slower than Dr)** |
| 3 | 66 µs | 145 µs | 0.46× |

(k=2's scope check pays `2·|S|` full `segmentIntersect`s — same costly
op as Dr's inner loop — plus graph build; it does not beat the thing it
replaces on the target case. Standalone anchor at 27 segs: `applyMove`
≈ 0.9–13 ms (noisy box), so a 2-line search leaf ≈ 2×applyMove + Dr is
tens of ms; the predictor is not orders-of-magnitude cheaper in the
regime where it would run, and it is less accurate than the area read
search already has.)

## 4. Recommendation: KILL for Lane S implementation

GO required 2-line error near-zero AND cost < 2× a leaf eval. Measured:
exact 5/65 (7.7%), MAE 1.49, max 4.5 → not near-zero; cost 4.5× a Dr on
the k=2 case → fails cost too. Contributing structural reasons, all
measured above: (1) ~81% of real closes leave endpoint-scope;
(2) 78% of in-scope pair yields decompose sequentially, which engine
mid-turn area reads already give exactly; (3) error grows k=2→k=3;
(4) BFS-shortest-chain ≠ face boundary even for singles. No GO path
without re-scoping to crossing-aware multi-face math — which is just
re-implementing Dr, defeating the purpose.
