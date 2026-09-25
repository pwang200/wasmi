;; Readable shape only. gen-finish-03-div-u emits case.wasm.
;;
;; Same loop as case 1 (`call $fat` + `br 0` until 1e6 fuel), but $fat does
;; `i32.div_u` (1 / 1). Division is 1 fuel and slower than add/call setup.
;; Extra ops in $fat mean fewer iterations than case 1. Out-of-fuel expected.
(module
  (memory 128)
  (func $fat
    (local i32) ;; × 30000
    i32.const 1
    i32.const 1
    i32.div_u
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
