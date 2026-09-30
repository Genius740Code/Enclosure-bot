# Ship-prep dry run — v8 stack (POP+TIE) — 2026-09-30

Mechanical dry run of the two pre-upload gates on **current master**. No engine changes,
no eval changes, no tuning, no gating. This is a readiness probe only.

- Base commit: `244e4ef` ("roadmap: stack-2 fully gated (h2h 10/10, league +19.8%/-75.5%;
  R attempt left inert scaffold") = `origin/master` at time of run.
- Worktree: `/tmp/opencode/game-v8ship` (detached at origin/master, branched `lane-v8-shipprep`).
- `git checkout` was never run inside `/home/genius74o/game`.

## 1. Toolchain

| Item | Value |
|---|---|
| `rustup target list --installed \| grep wasm` | `wasm32-wasip1` present |
| `rustup target add wasm32-wasip1` | **not needed** — no TOOLCHAIN-GAP |
| cargo / rustc | 1.98.1 (797e8a9bc) / 1.98.1 (48a229cea) |

## 2. WASM build

`cd retaliator && cargo build --release --target wasm32-wasip1` → **SUCCESS** (22.85s, no warnings).

| Item | Value |
|---|---|
| Artifact | `retaliator/target/wasm32-wasip1/release/retaliator.wasm` |
| Size | **310,981 bytes** (≈303.7 KiB) |
| Site limit | 8 MB → **3.9% of budget**, ~26× headroom |
| sha256 | `088632c54a9a44d852d12167e06ad16365a2faa805c2ec4beab95a41bb405f19` |

Lineage context: v6 was 290,011 B, v7 was 306,523 B. 310,981 B is the expected
v7-lineage + v8-stack delta. No size regression signal.

## 3. Exports

There is no `research/site-ops.md` in the repo; the verify method comes from
`prompt.md` ("verify exports (meridian_abi=1, alloc, run)") and `research/v7-todo.md` step 7.

No wasm tooling is installed locally (no `wasm-objdump` / `wasm-tools` / `wasm2wat`), so
this run adds `tools/wasm-exports.py` — a stdlib-only WASM export-section reader
(LEB128 + section id 7). Reusable, no deps:

```
python3 tools/wasm-exports.py retaliator/target/wasm32-wasip1/release/retaliator.wasm
```

Result — **4 exports, all 3 required present**:

| Export | Kind |
|---|---|
| `meridian_abi` | func |
| `meridian_alloc` | func |
| `meridian_run` | func |
| `memory` | memory |

Note: `prompt.md`'s shorthand "alloc, run" = the `meridian_alloc` / `meridian_run`
`#[unsafe(no_mangle)]` fns in `retaliator/src/lib.rs` (lines 114 / 120 / 129).
`meridian_abi()` returns literal `1`. This is a doc-abbreviation, **not** a missing export —
do not read the raw `alloc`/`run` names as a failure.

## 4. site_check

`cargo run --release --example site_check` → **OK** (exit 0)

```
opening reply: 16058
12 site-path replies, all legal. moves=[16058, 2713, 12419, 4867, 861, 17147,
                                      14946, 1254, 2247, 2767, 4933, 7608]
analysis OK, depth=2
site_check OK
```

**12/12 legal.** Opening reply `16058` is byte-identical to the v6/v7 lineage value in
`research/league-scoreboard.md` — the deterministic-prefix invariant still holds after the
v8 stack. Analysis depth 2 as before.

## 5. Blockers

**None.** Build, exports, and site_check are all green on `244e4ef` with no code fixes
required. Nothing was changed in the engine.

## 6. Verdict for v8 stack (POP+TIE) upload

**GO for the mechanical build/upload path, NOT-YET for a v8 *strength* claim.**

The toolchain is proven: this exact tree compiles to WASM inside the site limit, exports
the full required ABI, and answers 12/12 legal site-path replies. Whatever v8 becomes,
the upload mechanics do not need re-litigating.

What is *not* established here, and must not be inferred from this file:

- This is a dry run on **master**, not on the v8 stack. The v8 POP+TIE layer is not
  measured by anything in this report.
- No gates were run. No eval numbers, no h2h, no league, no gauge. Strength is unproven.
- The 12/12 result is a *legality* check, not a *quality* check. It says the bot answers
  valid moves; it says nothing about whether those moves are good.

Recommended next step before upload: re-run these two commands on the branch carrying the
actual v8 stack, and confirm the three stack gates separately before any upload.
