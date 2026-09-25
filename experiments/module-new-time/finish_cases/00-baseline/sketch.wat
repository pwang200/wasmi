;; Readable shape only. gen-finish-00-baseline emits case.wasm.
;;
;; Floor for instantiate + finish: smallest legal module the host can run.
;; No memory, no table, no extra funcs. `finish` returns 0 immediately.
(module
  (func $finish (export "finish") (result i32)
    i32.const 0
  )
)
