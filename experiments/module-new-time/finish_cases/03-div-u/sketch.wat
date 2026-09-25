;; Isolated live `i32.div_u`. `$finish` is the fuel loop.
;; One i32 local is the loop-carried numerator so `idiv` cannot be folded.
;; No `$fat`, no 30k locals. Out-of-fuel expected.
(module
  (func $finish (export "finish") (result i32)
    (local $i i32)
    (loop
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (drop (i32.div_u (local.get $i) (i32.const 3)))
      (br 0)
    )
    i32.const 0
  )
)
