Local Create-path harness. Times `Module::new`, then instantiates and calls `finish`.

```bash
# generate a case binary (one generator per case)
cargo run -p module-new-time --release --bin gen-01-active-element
cargo run -p module-new-time --release --bin gen-02-br-table
cargo run -p module-new-time --release --bin gen-03-func-spam
cargo run -p module-new-time --release --bin gen-04-nested-blocks
cargo run -p module-new-time --release --bin gen-05-fat-data
cargo run -p module-new-time --release --bin gen-06-many-locals
cargo run -p module-new-time --release --bin gen-07-combo

# time it
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/cases/01-active-element/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/cases/02-br-table/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/cases/03-func-spam/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/cases/04-nested-blocks/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/cases/05-fat-data/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/cases/06-many-locals/case.wasm
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/cases/07-combo/case.wasm
```

Each case in `cases/` has a `sketch.wat` (readable shape + why it is costly) and a Rust generator in `src/bin/` that writes `case.wasm`.
