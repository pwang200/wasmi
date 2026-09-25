;; Isolated unroll: 32 `call` of empty `$fat` (0 locals) per loop.
;; Compare to 13 (one empty call per loop). Not the 30k-zero trick.
(module
  (memory 128)
  (func $fat)
  (func $finish (export "finish") (result i32)
    (loop
      (call $fat) ;; × 32
      (br 0)
    )
    i32.const 0
  )
)
