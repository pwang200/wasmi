;; Readable shape only. Same as case 6, stride 64 (one load per cache line).
(module
  (memory 128)
  (func $fat
    (local $c i32) ;; plus more i32 to 30000
    (local.set $c (i32.load (i32.const 0)))
    (drop (i32.load (local.get $c)))
    (i32.store (i32.const 0)
      (i32.and (i32.add (local.get $c) (i32.const 64)) (i32.const 0x7ffffc)))
  )
  (func $finish (export "finish") (result i32)
    (loop
      (call $fat)
      (br 0)
    )
    i32.const 0
  )
)
