;; Real total T-A. Kernel A + create-7 filler to 100 KB. Returns 1 (no OOF).
;;
;; Live chain: $a → $b → $c, each 30_000 i32 locals. `$finish` unrolls
;; `call $a` × 32 per counted iter, then returns 1.
;; Remaining file bytes are unused combo dummies (never called / translated).
(module
  (memory 128)
  (func $a (local i32) (call $b)) ;; locals × 30000
  (func $b (local i32) (call $c)) ;; locals × 30000
  (func $c (local i32))           ;; locals × 30000
  (func $dummy
    (local i32) ;; × 50000
    (block
      i32.const 0
      br_table 0 0 0
    )
  ) ;; repeated to fill 100 KB
  (func $finish (export "finish") (result i32)
    (local i32)
    (local.set 0 (i32.const 4700))
    (loop
      (call $a) ;; × 32
      (local.tee 0 (i32.sub (local.get 0) (i32.const 1)))
      (br_if 0)
    )
    i32.const 1
  )
)
