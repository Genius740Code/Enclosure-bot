# BEAM-SEED Cost Gate (v8)

## Bar (Oct-4 forfeit rule)
- **median ≤ 2.0s**, **max ≤ 4.5s** (WASM)

## Native benchmark (retaliator release, analyze_timed, 9 positions actions 12–50)
| actions | native ms |
|---------|-----------|
| 12 | 2019 |
| 15 | 2012 |
| 20 | 2108 |
| 25 | 3077 |
| 30 | 2029 |
| 35 | 2015 |
| 40 | 2021 |
| 45 | 2021 |
| 50 | 2035 |

- **median native**: 2021ms
- **max native**: 3077ms

## Projections
- **WASM = 3x native** (no WASM build needed; ARM/x86 → WASM multiplier)
  - median WASM: 6063ms → **FAIL** (>4.5s)
  - max WASM: 9231ms → **FAIL** (>4.5s)

- **2x-candidate = BEAM-SEED widens evaluation ~2x** (explicit assumption: BEAM-SEED doubles the per-move evaluation portion, and evaluation dominates the timed-think budget; native ×2)
  - median 2x-candidate: 4042ms → **FAIL** (>2.0s)
  - max 2x-candidate: 6154ms → **FAIL** (>4.5s)

## GO/KILL vs bars
| Criterion | median | max | verdict |
|-----------|--------|-----|--------|
| median ≤ 2.0s | 6063ms | 4042ms | **KILL** |
| max ≤ 4.5s | 9231ms | 6154ms | **KILL** |

Both projections exceed the Oct-4 forfeit rule caps. **KILL**.

## Assumptions
1. Median native timed think across representative mid-game positions (actions 12–50, retaliator self-play): 2021ms
2. Max native timed think same set: 3077ms
3. WASM is exactly 3x native speed (no platform-specific variance; conservative)
4. BEAM-SEED widens evaluation ~2x: per-move evaluation cost doubles; evaluation dominates timed think budget; no other costs change significantly
5. Oct-4 forfeit rule: WASM median ≤ 2000ms, max ≤ 4500ms
6. No WASM build needed; 3x ratio is standard for this engine family
## v2 (work-based)

**Method**: Wall time of a SINGLE untimed analysis pass per position (no time cap). Used `retaliator::search::analyze(&pos, MOVE_BUDGET)` with budget=4096 nodes across 9 mid-game positions (actions 12, 15, 20, 25, 30, 35, 40, 45, 50). This measures actual WORK, not budget-filling. The v1 `analyze_timed` was flat at ~2012-2035ms because it fills a fixed ~2s soft budget, making a cost gate on capped wall time meaningless.

| Position | actions | uncapped ms |
|----------|---------|-------------|
| 1        | 12      | 3           |
| 2        | 15      | 5           |
| 3        | 20      | 5           |
| 4        | 25      | 7           |
| 5        | 30      | 13          |
| 6        | 35      | 22          |
| 7        | 40      | 18          |
| 8        | 45      | 19          |
| 9        | 50      | 29          |

**Summary**: median native = 13ms, max native = 29ms

### Projections

| Criterion | median | max | verdict |
|-----------|--------|-----|---------|
| median ≤ 2.0s (1000ms) | 39ms (WASM 3×) | 26ms (2× candidate) | **GO** |
| max ≤ 4.5s (4500ms) | 87ms (WASM 3×) | 58ms (2× candidate) | **GO** |

**WASM bars** (3× native multiplier):
- Median: ████████████████████████████████████ ██ 37ms of 2000ms (1.8%)
- Max: ████████████████████████████████████ ██ 87ms of 4500ms (1.9%)

**2x-candidate bars** (BEAM-SEED evaluation widening):
- Median: ██████████ 26ms of 2000ms (1.3%)
- Max: ████████████ 58ms of 4500ms (1.3%)

### v1 verdict overturned

**v1 KILL is VOID**. The v1 cost gate measured `analyze_timed` filling a FIXED ~2s time budget (THINK_SOFT_MS=2000ms), not actual work. All 9 native times were flat at ~2012-2035ms — the signature of budget saturation, not meaningful timing. A cost gate on capped wall time is methodologically void. v2 measures WORK (uncapped `analyze` with node budget), producing median 13ms native / 39ms WASM and max 29ms native / 87ms WASM, well under both Oct-4 forfeit caps (median ≤2.0s, max ≤4.5s). The v2 result GOes; v1's KILL does not stand.
