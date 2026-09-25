;; Total step 1 kernel B. No 100 KB Create filler.
;; `call_indirect` payload only (survives a local-zero fuel PR).
(module
  (memory 128)
  (table 1024 funcref)
  (elem (i32.const 0) $fat) ;; × 1024
  (func $fat)
  (func $finish (export "finish") (result i32)
    (loop
      (call_indirect (type $t) (i32.const 0))
      (br 0)
    )
    i32.const 0
  )
)
