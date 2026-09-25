;; Isolated nest: a → b → c, each 0 extra locals. `$finish` loops `call $a`.
;; Compare to 13 (one empty call). Not the 30k-zero trick.
(module
  (memory 128)
  (func $a (call $b))
  (func $b (call $c))
  (func $c)
  (func $finish (export "finish") (result i32)
    (loop
      (call $a)
      (br 0)
    )
    i32.const 0
  )
)
