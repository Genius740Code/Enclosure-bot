# PREP-BOOK draft — offline counter-lines vs GB scripts (external session, 2026-09-30)

Inputs (read-only): `research/gb-habit-replication.md` (GB facts cited, not
re-derived) + 3 GB games on `origin/lane-v8-autopsy`
(30c7653b/ad65f054/d2d4b4fd). Method: exact replay per ply; per-color area =
index-space polygon area of own bounded faces (affine-invariant RANKING only —
does not reproduce engine absolutes; picks the winner 3/3).

## Bank-building moves per game

GB builds exactly ONE dominant loop per game: one `connect` worth ~95% of final
area, preceded by a 3-finger straight-run build. Per-move growth 0.0 for every
extend in the run — the bank is a SHAPE, not accretions.

| Game | Bank close (ply, move) | Growth | Loop | Corridor root (deg) |
|---|---|---|---|---|
| 30c7653b (GB blue W) | ply 53 `connect G19-J19` | 103.0 | 21 cells | D10 (-6,0) deg 5 |
| ad65f054 (GB red W) | ply 51 `connect M19-J19` | 103.0 | 20 cells | P10 (6,0) deg 4 |
| d2d4b4fd (GB red W) | ply 50 `connect P1-M1` | 108.0 | 17 cells | M10 (3,0) deg 5 |

New fact 3/3: the bank is a BORDER-HUGGING RECTANGLE (board-edge cells closed
by a 3-cell segment along one border row), not a corridor. Centroids all on the
row-10 axis. Close-rate tell reconciled: GB 13/10/14 vs v7 24/24/26 (0.50x) but
GB's single close worth 103/103/108 vs v7's best 17.0/22.5/16.0 (5–7x per
close), banked ply 53/51/50 vs v7's 39/41/41 (10–12 plies later).

## Counter-line per game

- 30c7653b: v7 held M19 (3,9) at plies 50-51, 4.0 from GB's closing interior
  (H19/I19 EMPTY). Counter: extend M19 along row 19 at ply 53 instead of
  `M16-P18` @54. Available from ply 50.
- ad65f054: v7 held M13 (3,3) plies 17-49 on GB's spine (2.0 from M16), never
  turned toward row 19. Counter: extend M13 up the spine @20 instead of
  `A10-D8`. Available from ply 17.
- d2d4b4fd: v7 held J10 (0,0) plies 8-49, 3.0 from GB's deg-5 root M10, never
  advanced. Counter: extend J10 onto M10 @12 instead of `G8-J11`. Available
  from ply 8.

## Common counter (3/3): deny the border run

Park one cell on the open 2–3-cell border segment GB is about to close, same
ply or 1–2 earlier. In 3/3 the closing interior was EMPTY and v7 was 4.0/6.1/9.8
away. One rule, testable: "opponent's unbanked wall run terminates on a border
row/col with an open segment → spend the next move inside it." Strictly stronger
than corridor-root contest (v7 played the centre row in 3/3 and still lost 2/3 —
root-contest is necessary, not sufficient). v7's failure is AXIS error (own bank
opposite half), not tempo.

## Falsifier (check before coding)

3 games, all vs v7, all GB wins — border-hug may be a counter to v7's half-board
opening, not a GB habit. Separate "hugs far border" from "hugs the border v7 did
not claim". Phase 4 b1e7d2d1/f33e29fc games are the discriminator.
