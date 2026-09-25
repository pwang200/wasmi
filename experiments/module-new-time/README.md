Timing harness for XRPL-like `Module::new` (Create) and instantiate + `finish` (Finish).

Same host for both. Create cases live in `create_cases/`; Finish cases live in `finish_cases/`.

Each Finish case is one trick, or the same trick with a different parameter. No mixing (no 30k locals plus ALU/load/unroll/nest). Locals-count variants of case 1 (1k / 10k) were removed; 01 is the locals trick, 13 is the empty-call control.

```bash
# generate every case, then time every case.wasm
./experiments/module-new-time/run-all.sh

# generate or time one case
cargo run -p module-new-time --release --bin gen-finish-01-hot-call-locals
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/finish_cases/01-hot-call-locals/case.wasm
```

The host builds a fresh Engine per run (`escrow_engine_config`), times `Module::new`, then instantiates with `Store::limiter` and 1_000_000 fuel and calls export `finish` (`() -> i32`). It prints file size, engine, `Module::new`, instantiate, finish, `instantiate + finish`, and total. Out-of-fuel on `finish` is a valid outcome. A host crash (currently `20-memory-grow` on aarch64 tail dispatch) is recorded by `run-all.sh`; later cases still run.
