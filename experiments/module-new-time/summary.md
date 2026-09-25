# module-new-time: Create / Finish / total hunt

Lab note for the XRPL-like Wasmi 2.0.0 timing hunt on branch `experiment/module-new-time`. Host crate: `experiments/module-new-time`. Numbers below are from this Mac (Apple P-core, L2 about 12–16 MiB) unless marked otherwise.

## Goal and constraints

Maximize wall time of XRPL-like **Create** (`Module::new`) plus **Finish** (instantiate + first `finish` call).

Hard caps:

- wasm ≤ 100_000 bytes
- 1_000_000 fuel
- export `finish` `() -> i32`
- same Engine as `escrow_engine_config()`: `consume_fuel`, `ignore_custom_sections`, `allow_start_fn(false)`, `LazyTranslation`, no floats, listed proposals off (including bulk memory), `EnforcedLimits::strict()` (max 10k funcs, average ≥ 40 body bytes after 1000 combined body bytes, max 1000 data segments, max 32 params)
- StoreLimits: 8 MiB, 1024 table elems, 1 instance / memory / table

Create and Finish each use a **fresh Engine**. Do not invent a different Engine. `Module::new` and instantiate (`Memory::new`, data write, `table.init`) are unmetered. Fuel starts with the Store. All function bodies are validated at `Module::new`. The first call of a function translates at 7 fuel per Wasm body byte.

Default op fuel: most ops 1; `nop` / `drop` / `block` / `loop` / `end` / `return` / `else` / `unreachable` are 0. `call` is 1 plus whatever the callee consumes. `br 0` is the 1-fuel loop-back (`loop` itself is 0).

Each call does `local_cells.fill_with(Cell::default)` for declared locals. `Cell` is `u64`, so 30k i32 is 240 KB. Unused locals are still slotted. Locals do not cost fuel today. Value stack max is 1_000_000 bytes. Recursion cap is 1000.

## Two kernels

There is an open PR to charge fuel on zeroing locals. That splits the hunt:

- **A** — 30k-class locals (current Wasmi). Fixable by that PR.
- **B** — no 30k-zero. Survives the PR. B is the more important kernel because A is the one the PR can close.

`strict()` translates at most 30k locals. Create dummies may declare 50k because they are never called and never translated (`LazyTranslation`).

## Isolation rule

One Finish case is one trick, or the same trick with a different parameter. No mixing (the original case 3 also used 30k locals). Locals-count clones of case 1 (1k / 10k) were deleted: **01** is the locals trick, **13** is the empty-call control. Unroll 256 was deleted (same trick as unroll 32). Isolation of Finish cases is commit `b930c696`.

## Create hunt

Cases live in `create_cases/` (`gen-01` … `gen-07`).

| case | trick |
| --- | --- |
| 01 | fat active element (Create-only; huge table failed instantiate) |
| 02 | short `br_table` |
| 03 | many functions at the 40-byte floor |
| 04 | nested blocks |
| 05 | fat data |
| 06 | many locals (cheap in the file, validator work) |
| **07-combo** | **winner** |

**07-combo** (~3 ms Create here) is the winner. Shape and why it beats the singles are in the next section.

## How 07-combo was chosen

We did not start with “max funcs × 50k locals.” We ranked singles, then combined only what **adds**.

**Singles first.** 01 elems, 02 `br_table`, 03 func-spam, and 04 nests were all ~1.2–1.7 ms / 100 KB. 05 fat data and 06 one function with 50k locals were measured after 04 because we were not sure they were worth a case. They were cheap alone (~0.5 ms and ~0.7 ms). “Many locals” looked like a dud until we asked what happens if you **repeat** it.

**The dead combo.** Half elems + half functions does not add. Case 1 and case 3 cost about the same per byte, so a split **trades**.

**The live combo.** Keep case 3’s shape: ~2400 functions at the 40-byte `strict()` floor (max function count in 100 KB). Replace each dummy’s nops with work that is cheap in the file and expensive per function at `Module::new`:

1. One group of **50_000 i32 locals** (5 bytes). wasmparser does `local_inits.resize(50_000, true)` on every body. One function is ~0.7 ms; ~2400 of them is the jump to ~3 ms.
2. Leftover ~34 bytes: a short **`br_table`**, not nops. That is case 2’s validator loop as filler. Small next to the memset.

`LazyTranslation` is why this is legal: Create **validates** every dummy and never **translates** them. 50k is over Wasmi’s 30k translate cap, so those dummies must stay uncalled. `finish` is a padded `i32.const` with no extra locals so Finish still works and the 40-byte average holds.

**What we did not put in.** Fewer, fatter bodies (more `br_table` / deeper nests) lose functions and lose. Unique types, imports, 1000 globals, fat data, and overlong LEBs all spend bytes that could be another 50k-local dummy. Staying under 1000 code bytes to dodge the 40-byte rule only allows a few hundred tiny funcs — not enough memsets.

After 07 we treated ~3 ms / 100 KB as the Create ceiling on this budget. That is why T-0 / T-A / T-B pad with the same unused 50k-local dummies: it is the Create floor, not a new trick.

## Finish isolation (this Mac)

Cases live in `finish_cases/`. After the isolation redo, Finish time without mixing is almost entirely the 30k-slot L2 memset times about ⅓e6 calls.

| case | trick | Finish (order of magnitude) |
| --- | --- | --- |
| 00 | `i32.const 0` baseline | — |
| 01 | hot `call` of 30k-local `$fat` | ~235–242 ms |
| 02 | `call_indirect` | ~1.6–2.5 ms |
| 03 / 04 | live `i32.div_u` / `i64.div_u` (loop-carried; `1/1` folds away) | ~0.5 ms |
| 05–09 | `i32.load` (same addr / strides 4, 64, 4096, 4160) | ~0.9–1.8 ms |
| 10 / 11 / 12 | one-shot translate (`br_table`, nested blocks, many callees) | ~0.2–1.5 ms |
| 13 | hot empty `call` (0 extra locals) | ~1.2–1.5 ms |
| 16 | unroll empty calls | ~2.7–3 ms |
| 17 | nest-3 empty funcs | ~2 ms |
| 18 | recurse depth 998 | ~1 ms |
| 20 | `memory.grow 1` at the 8 MiB cap | **host crash** on aarch64 tail-call dispatch |

Naked `loop { br 0 }` is the cheap fuel-burn floor: about **0.54 ms / 1e6 fuel** (~0.54 ns/fuel). It is a floor, not a candidate. Out-of-fuel on isolation Finish cases is a valid outcome. `run-all.sh` records a host crash and continues.

## What the isolation numbers mean

**Nest and L2.** Nest-3 of 30k locals is 720 KB of slots. That is far below this P-core’s 12–16 MiB L2, so nest-3 does not add an L2-miss cliff here (nest-3 + unroll-32 × 30k was ~443 ms, about the same as unroll-only ~437–460 ms). Nest is still part of A: a server L2 is often 1–2 MB, where 720 KB may matter. Nest does not *lower* time here. Nest-4 of full 30k overflows the 1 MB value stack (need ~28k/frame, or 30k × 3 plus a smaller 4th).

**Unroll.** Diagnostic, not a payload. Helps when body time/fuel is much larger than `br` (30k-zero). Helps a little when the body is only a bit above `br` (`call_indirect`). `br` is cheaper per fuel than `call_indirect`.

**`call` vs `call_indirect`.** `call` is a fixed callee. `call_indirect` is table[index] + type check + call.

**Pointer chase is not B on this machine.** Packed 100 KB chase (Sattolo cycle written at instantiate, `cur = mem[cur]`) was ~0.92 ms / ~0.9 ns/fuel — *faster* than `call_indirect` (~1.9 ns/fuel), so worse as a DoS here. 8 MiB linear memory still sits in this L2; a DRAM chase is unreachable on this Mac. The **8.9 ns/fuel** dependent chase from another session is a server / scattered-memory probe, not this machine’s B. Official B here: `call_indirect` ± unroll 32. Worth one server run of an **8 MiB scattered** chase only, not the packed 100 KB module.

**Other dead ends.** Constant-folded `1/1` made fake div times; the live payload is `(i+1)/3`. `memory.grow(0)` folds to `memory.size`. A 100 KB pad on B failed `strict()` average 40 until fat/finish were padded to the 39-byte body floor.

## Total hunt, two steps

1. **Finish combos, no 100 KB pad** — `total_cases/`. Rank by total (Create of a tiny module is noise).
2. **Real total** — `real_total/`. Same kernels plus Create-7 combo dummies to 100 KB.

Step 1 on this Mac:

| case | kernel | size | total (OOF Finish) |
| --- | --- | --- | --- |
| `a-30k-unroll` | 30k `$fat` + unroll 32 | 125 B | ~437–460 ms |
| `b-call-indirect` | `call_indirect` only | small | ~3 ms |
| `b-pointer-chase` | packed Sattolo, 24927 nodes | 99998 B | ~0.92 ms Finish (not B here) |

`gen-total-probe` writes `total_cases/_probe/` (nest/unroll mixes). Those are diagnostics; `run-all.sh` skips `_probe/`.

Agreed A for step 2: 30k-class locals × nest-3 + unroll 32 (nest stays even though it does not add time here). Agreed B: `call_indirect` + unroll 32.

## Real total (T-*) — whole-system wasm

`real_total/`. Whole-system testing cannot accept out-of-fuel, so these **return 1**. The three loops are **counted** (not `loop { br 0 }`). Iteration counts leave about 50k fuel.

`i32.const` is **signed** LEB128. Encoding the counter with unsigned LEB makes 64..=127 (and some larger values) a negative count; the loop never hits zero and OOFs.

| case | path | kernel | pad | iters | size | `Module::new` | finish | fuel | total |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **T-00** | `t-00-br-loop` | counted decrement (`get`/`sub`/`tee`/`br_if`, ~6 fuel/iter) | no | 158000 | 59 B | 0.36 ms | 0.70 ms | 948172 | 1.2 ms |
| **T-0** | `t-0-control` | Create-7 dummies + `i32.const 1` | 100 KB (2437 dummies) | — | 100000 | 3.0 ms | 0.11 ms | 275 | 3.3 ms |
| **T-A** | `t-a-30k-nest-unroll` | nest-3 × 30k + unroll 32 | 100 KB (2432 dummies) | 4700 | 99972 | 3.0 ms | 439 ms | 932032 | 443 ms |
| **T-B** | `t-b-call-indirect-unroll` | unroll-32 `call_indirect` | 100 KB (2406 dummies) | 9200 | 99963 | 2.5 ms | 2.4 ms | 939965 | 5.5 ms |

T-0 is the Create floor (same dummy as 07-combo; `finish` returns 1). T-00 is the cheapest legal way to burn most of 1e6 fuel. T-A is still the 30k-zero Finish. T-B is Create + `call_indirect`, the one that survives a local-zero fuel PR. T-B has fewer dummies because the table + 1024-entry elem list take about 1 KB.

## How to run

```bash
./experiments/module-new-time/run-all.sh
```

Generates every Create / Finish / step-1 total / T-* case, then times each `case.wasm`. Isolation Finish may OOF. A `real_total/` case that does not print `finish -> 1` is a failure. A host crash is recorded; later cases still run.

## Open

- Port only an **8 MiB scattered** pointer chase to a server (small L2 / DRAM). Do not replace official B on this machine with the packed chase.
- A is likely closable by the local-zero fuel PR; B is the one that remains.
- Do not commit `crates/wasmi/src/instance/handle.rs`, `crates/wast/tests/spec`, `.claude/`, `.idea/`, `PR.md`, or `issue.md` with this experiment.
