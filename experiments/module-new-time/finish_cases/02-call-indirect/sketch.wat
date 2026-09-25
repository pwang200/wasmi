;; Readable shape only. gen-finish-02-call-indirect emits case.wasm.
;;
;; Isolated `call_indirect` trick: 1024-slot table, empty `$fat` (no 30k locals).
;; Loop burns 1e6 fuel. Compare to 13-hot-call-0-locals (direct call, no locals).
;; Out-of-fuel expected.
(module
  (type $t (func))
  (memory 128)
  (table 1024 funcref)
  (elem (i32.const 0) $fat $fat) ;; × 1024
  (func $fat (type $t))
  (func $finish (export "finish") (result i32)
    (loop
      (call_indirect (type $t) (i32.const 0))
      (br 0)
    )
    i32.const 0
  )
)
