;; Same as case 1, $fat has 1_000 i32 locals.
(module
  (memory 128)
  (func $fat
    (local i32) ;; × 1000
  )
  (func $finish (export "finish") (result i32)
    (loop
      (call $fat)
      (br 0)
    )
    i32.const 0
  )
)
