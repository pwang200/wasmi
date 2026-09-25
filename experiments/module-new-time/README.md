Timing harness for XRPL-like `Module::new` (Create) and instantiate + `finish` (Finish).

Same host for both. Create cases live in `create_cases/`; Finish cases live in `finish_cases/`.

```bash
# generate a Create case
cargo run -p module-new-time --release --bin gen-01-active-element
# … gen-02-br-table … gen-07-combo

# generate a Finish case
cargo run -p module-new-time --release --bin gen-finish-01-hot-call-locals
cargo run -p module-new-time --release --bin gen-finish-02-call-indirect

# time any .wasm (Create or Finish)
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/create_cases/01-active-element/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/finish_cases/01-hot-call-locals/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/finish_cases/02-call-indirect/case.wasm
```

The host builds a fresh Engine per run (`escrow_engine_config`), times `Module::new`, then instantiates with `Store::limiter` and 1_000_000 fuel and calls export `finish` (`() -> i32`). It prints file size, engine, `Module::new`, instantiate, finish, `instantiate + finish`, and total. Out-of-fuel on `finish` is a valid outcome.
