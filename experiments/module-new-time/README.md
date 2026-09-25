Timing harness for XRPL-like `Module::new` (Create) and instantiate + `finish` (Finish).

Same host for both. Create cases live in `create_cases/`; Finish cases live in `finish_cases/`.

```bash
# generate a Create case
cargo run -p module-new-time --release --bin gen-01-active-element
# … gen-02-br-table … gen-07-combo

# generate a Finish case
cargo run -p module-new-time --release --bin gen-finish-00-baseline
cargo run -p module-new-time --release --bin gen-finish-01-hot-call-locals
cargo run -p module-new-time --release --bin gen-finish-02-call-indirect
cargo run -p module-new-time --release --bin gen-finish-03-div-u
cargo run -p module-new-time --release --bin gen-finish-04-div-u64
cargo run -p module-new-time --release --bin gen-finish-05-load-same
cargo run -p module-new-time --release --bin gen-finish-06-load-stride-4
cargo run -p module-new-time --release --bin gen-finish-07-load-stride-64
cargo run -p module-new-time --release --bin gen-finish-08-load-stride-4096
cargo run -p module-new-time --release --bin gen-finish-09-load-stride-4160
cargo run -p module-new-time --release --bin gen-finish-10-br-table
cargo run -p module-new-time --release --bin gen-finish-11-nested-blocks
cargo run -p module-new-time --release --bin gen-finish-12-many-callees

# time any .wasm (Create or Finish)
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/create_cases/01-active-element/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/finish_cases/00-baseline/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/finish_cases/01-hot-call-locals/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/finish_cases/02-call-indirect/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/finish_cases/03-div-u/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/finish_cases/04-div-u64/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/finish_cases/05-load-same/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/finish_cases/06-load-stride-4/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/finish_cases/07-load-stride-64/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/finish_cases/08-load-stride-4096/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/finish_cases/09-load-stride-4160/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/finish_cases/10-br-table/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/finish_cases/11-nested-blocks/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/finish_cases/12-many-callees/case.wasm
```

The host builds a fresh Engine per run (`escrow_engine_config`), times `Module::new`, then instantiates with `Store::limiter` and 1_000_000 fuel and calls export `finish` (`() -> i32`). It prints file size, engine, `Module::new`, instantiate, finish, `instantiate + finish`, and total. Out-of-fuel on `finish` is a valid outcome.
