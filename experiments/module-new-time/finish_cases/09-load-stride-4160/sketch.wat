;; Isolated load, stride 4160 (4096+64). Same as case 6 otherwise.
(module
  (memory 128)
  (func $fat
    (local $c i32)
    (local.set $c (i32.load (i32.const 0)))
    (drop (i32.load (local.get $c)))
    (i32.store (i32.const 0)
      (i32.and (i32.add (local.get $c) (i32.const 4160)) (i32.const 0x7ffffc)))
  )
  (func $finish (export "finish") (result i32)
    (loop
      (call $fat)
      (br 0)
    )
    i32.const 0
  )
)
