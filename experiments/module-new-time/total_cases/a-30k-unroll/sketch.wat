;; Total step 1 kernel A. No 100 KB Create filler.
;; 30k-zero payload + unroll applicator.
(module
  (memory 128)
  (func $fat (local i32)) ;; × 30000
  (func $finish (export "finish") (result i32)
    (loop
      (call $fat) ;; × 32
      (br 0)
    )
    i32.const 0
  )
)
