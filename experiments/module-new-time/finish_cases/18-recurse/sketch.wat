;; Isolated self-recursion. Wasmi default cap is 1000 frames.
;; `finish` + `$f(998)` … `$f(0)` is 1000 frames. No extra locals.
(module
  (func $f (param $n i32)
    (if (local.get $n)
      (then (call $f (i32.sub (local.get $n) (i32.const 1))))))
  (func $finish (export "finish") (result i32)
    (loop
      (call $f (i32.const 998))
      (br 0)
    )
    i32.const 0
  )
)
