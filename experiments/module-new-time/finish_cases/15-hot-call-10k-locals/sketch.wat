;; Same as case 1, $fat has 10_000 i32 locals.
(module
  (memory 128)
  (func $fat
    (local i32) ;; × 10000
  )
  (func $finish (export "finish") (result i32)
    (loop
      (call $fat)
      (br 0)
    )
    i32.const 0
  )
)
