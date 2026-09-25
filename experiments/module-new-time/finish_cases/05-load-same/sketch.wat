;; Readable shape only. gen-finish-05-load-same emits case.wasm.
;;
;; $fat always `i32.load`s address 0. L1-hit baseline for the load cases.
;; Same call+$fat+br loop as case 1. Out-of-fuel expected.
(module
  (memory 128)
  (func $fat
    (local i32) ;; × 30000
    i32.load (i32.const 0)
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
