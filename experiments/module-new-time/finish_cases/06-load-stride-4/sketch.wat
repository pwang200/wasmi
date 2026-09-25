;; Isolated sequential load. Cursor at mem[0] persists across calls.
;; One i32 local is the cursor (not a 30k-zero). Stride 4.
(module
  (memory 128)
  (func $fat
    (local $c i32)
    (local.set $c (i32.load (i32.const 0)))
    (drop (i32.load (local.get $c)))
    (i32.store (i32.const 0)
      (i32.and (i32.add (local.get $c) (i32.const 4)) (i32.const 0x7ffffc)))
  )
  (func $finish (export "finish") (result i32)
    (loop
      (call $fat)
      (br 0)
    )
    i32.const 0
  )
)
