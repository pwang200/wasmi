;; Readable shape only. gen-finish-04-div-u64 emits case.wasm.
;;
;; Same loop as case 1, but $fat does `i64.div_u` (1 / 1). Wider idiv than
;; case 3's i32.div_u. Extra fuel per call; out-of-fuel expected.
(module
  (memory 128)
  (func $fat
    (local i32) ;; × 30000
    i64.const 1
    i64.const 1
    i64.div_u
    drop
  )
  (func $finish (export "finish") (result i32)
    (loop
      (call $fat)
      (br 0)
    )
    i32.const 0
  )
)
