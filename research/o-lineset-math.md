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

## 2. Results — PENDING RUN (filled on completion)

## 3. Cost — PENDING RUN

## 4. Recommendation — PENDING RUN
