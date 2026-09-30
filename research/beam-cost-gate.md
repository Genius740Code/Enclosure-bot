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
