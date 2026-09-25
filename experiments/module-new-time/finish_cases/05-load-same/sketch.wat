;; Isolated `i32.load` of address 0 (L1-hit). Same call+$fat+br loop as 13.
;; `$fat` has no extra locals. Out-of-fuel expected.
(module
  (memory 128)
  (func $fat
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
