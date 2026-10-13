# real_total

Whole-system wasm for XRPL-like timing. Each `case.wasm` has no imports, exports `finish` (`() -> i32`), and returns 1 without running out of fuel. `sketch.wat` next to it shows the shape (not the exact bytes). Generators: `../src/bin/gen-real-total-*.rs`.

Caps: wasm ≤ 100_000 bytes and 1_000_000 fuel, except `t-b-50k-400k` (≤ 50_000 bytes, 400_000 fuel). Engine is `escrow_engine_config()` in `../src/config.rs`.

## Timing

Medians of 25 runs, one session, this Mac (Apple P-core), 2026-10-13. Fresh Engine per run.

| case | what it is | `Module::new` | finish | total |
| --- | --- | --- | --- | --- |
| T-00 `t-00-br-loop` | 59 B; counted loop spends ~950k fuel | 0.04 ms | 0.29 ms | 0.34 ms |
| T-0 `t-0-control` | 100 KB of Create padding; `finish` just returns 1 | 1.65 ms | 0.02 ms | 1.70 ms |
| T-A `t-a-30k-nest-unroll` | 100 KB padding + 30k locals × nest-3 + unroll 32 | 1.64 ms | 423 ms | 425 ms |
| T-B `t-b-call-indirect-unroll` | 100 KB padding + `call_indirect` × 32 per iter | 1.63 ms | 1.68 ms | 3.72 ms |
| T-B 50k-400k `t-b-50k-400k` | T-B at 50 KB / 400k fuel | 0.82 ms | 0.69 ms | 1.90 ms |

Total is engine + `Module::new` + instantiate + finish. Instantiate is ~0.4 ms for T-A / T-B (128-page memory; T-B also fills a 1024-entry table) and ≤ 0.04 ms for T-00 / T-0.

Single runs swing a lot on this Mac (the same file has shown 1.6–3.4 ms `Module::new`). Compare medians or minimums, not one run.

| case | bytes | fuel limit | fuel used | padding dummies | loop iters |
| --- | --- | --- | --- | --- | --- |
| T-00 | 59 | 1_000_000 | 948_172 | 0 | 158_000 |
| T-0 | 100_000 | 1_000_000 | 275 | 2437 | — |
| T-A | 99_972 | 1_000_000 | 932_032 | 2432 | 4700 |
| T-B | 99_963 | 1_000_000 | 939_965 | 2406 | 9200 |
| T-B 50k-400k | 49_983 | 400_000 | 378_958 | 1187 | 3700 |

## Cases

- **Create padding** (T-0, T-A, T-B, T-B 50k-400k). Unused functions at the 40-byte `strict()` floor, each declaring 50k i32 locals plus a short `br_table`. `Module::new` validates every body (wasmparser resizes its locals list per body). `LazyTranslation` never translates them and they are never called, so they cost no fuel. This is Create case 07-combo, the most `Module::new` time per byte found.
- **T-00** — Finish floor. Cheapest counted loop that spends most of the fuel and returns 1. No padding.
- **T-0** — Create alone. Padding plus a trivial `finish`.
- **T-A** — Finish kernel A. `finish` calls `$a → $b → $c`, each with 30k i32 locals, 32 times per iteration. Each call zeroes 240 KB of local slots and zeroing costs no fuel today. The open PR that charges fuel for zeroing locals should close this.
- **T-B** — Finish kernel B. 32 `call_indirect` per iteration into an empty function through a 1024-slot table. Survives the locals PR. About 1.8 ns/fuel versus 0.3 ns/fuel for T-00. The table and 1024-entry elem segment take ~1 KB, so T-B has fewer dummies than T-0.
- **T-B 50k-400k** — T-B under the smaller caps. Create scales with dummy count (about half), Finish with fuel (about 0.4×). Instantiate does not shrink.

Loop counters are `i32.const`, which is signed LEB128. Unsigned encoding turns some counts negative and the loop runs out of fuel.

## Run

```bash
# generate and time one case (fuel argument defaults to 1_000_000)
cargo run -p module-new-time --release --bin gen-real-total-b-50k-400k
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/real_total/t-b-50k-400k/case.wasm 400000
```

`../run-all.sh` regenerates and times every case and fails any `real_total/` case that does not print `finish -> 1`.
