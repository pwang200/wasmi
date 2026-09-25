;; Same 30k-local $fat as case 1. $finish calls it 32 times per loop so less
;; fuel is spent on `br`. One live 240 KB frame at a time (not a nest).
(module
  (memory 128)
  (func $fat
    (local i32) ;; × 30000
  )
  (func $finish (export "finish") (result i32)
    (loop
      (call $fat) ;; × 32
      (br 0)
    )
    i32.const 0
  )
)
