;; Same as case 1 (call+$fat+br until OOF, 8 MiB), but $fat has no extra locals.
;; Pair with 14/15/01 to see if finish time scales with local-slot zeroing.
(module
  (memory 128)
  (func $fat)
  (func $finish (export "finish") (result i32)
    (loop
      (call $fat)
      (br 0)
    )
    i32.const 0
  )
)
