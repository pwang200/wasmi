;; Readable shape only. gen-07-combo emits the ~100 KB case.wasm.
;;
;; Combines case 3 (many functions at the 40-byte floor) with, in *each* dummy:
;;   - 50_000 i32 locals (case 6, cheap in the file, memset-style work per func)
;;   - a short br_table (case 2's validator loop, instead of nops)
;;
;; Do not also spend the 100 KB on a fat element list: case 1 and case 3 cost
;; about the same per byte, so splitting the file would not add, it would trade.
;;
;; EscrowCreate: OK (average ≥ 40).
;; EscrowFinish: OK. `finish` has no extra locals (Wasmi translate cap is 30k).
(module
  (func $dummy
    (local i32) ;; × 50000
    (block
      i32.const 0
      br_table 0 0 0 ;; ~25 targets, fills the 40-byte body
    )
  ) ;; repeated ~2400 times
  (func $finish (export "finish") (result i32)
    i32.const 0 ;; padded to the same body length
  )
)
