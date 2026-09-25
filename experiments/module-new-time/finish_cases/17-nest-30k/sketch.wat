;; Three 30k-local functions: a -> b -> c. $finish loops `call $a`.
;; Four frames (case 1 + three 240 KB) hit Wasmi's 1 MB value stack.
(module
  (memory 128)
  (func $a (local i32) (call $b)) ;; × 30000 locals
  (func $b (local i32) (call $c))
  (func $c (local i32))
  (func $finish (export "finish") (result i32)
    (loop
      (call $a)
      (br 0)
    )
    i32.const 0
  )
)
