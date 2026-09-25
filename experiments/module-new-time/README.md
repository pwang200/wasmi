Local Create-path harness. Times `Module::new`, then instantiates and calls `finish`.

```bash
# generate a case binary (one generator per case)
cargo run -p module-new-time --release --bin gen-01-active-element

# time it
cargo run -p module-new-time --release --bin module-new-time -- \
    experiments/module-new-time/cases/01-active-element/case.wasm
```

Each case in `cases/` has a `sketch.wat` (readable shape + why it is costly) and a Rust generator in `src/bin/` that writes `case.wasm`.
