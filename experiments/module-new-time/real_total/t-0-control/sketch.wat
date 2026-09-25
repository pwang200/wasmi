;; Real total T-0 (control). Create-7 filler, `finish` returns 1.
;;
;; Unused dummies: 50_000 i32 locals + short br_table, 40-byte floor.
;; `finish` is `i32.const 1` padded to the same body length. Never calls a dummy.
;; No memory / table — same shape as create_cases/07-combo.
(module
  (func $dummy
    (local i32) ;; × 50000
    (block
      i32.const 0
      br_table 0 0 0 ;; fills the 40-byte body
    )
  ) ;; repeated to ~100 KB
  (func $finish (export "finish") (result i32)
    i32.const 1 ;; padded to the same body length
  )
)
