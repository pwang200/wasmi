;; Isolated live `i64.div_u`. Same shape as case 3, wider idiv.
;; No `$fat`, no 30k locals. Out-of-fuel expected.
(module
  (func $finish (export "finish") (result i32)
    (local $i i64)
    (loop
      (local.set $i (i64.add (local.get $i) (i64.const 1)))
      (drop (i64.div_u (local.get $i) (i64.const 3)))
      (br 0)
    )
    i32.const 0
  )
)
