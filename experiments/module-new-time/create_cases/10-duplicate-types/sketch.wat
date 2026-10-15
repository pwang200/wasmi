;; Readable shape only. gen-10-duplicate-types emits the ~100 KB case.wasm.
;;
;; EscrowCreate: OK. strict() does not cap the number of types, and duplicate
;; types are legal.
;; EscrowFinish: OK. Only `finish` runs.
;;
;; Case 08 with the opposite choice: instead of ~7.7k distinct types averaging
;; ~13 bytes, ~33k copies of the shortest type, `() -> ()`, at 3 bytes each.
;; The engine de-duplicates types, so each copy is validated and looked up but
;; not stored. This tests whether more entries per byte beats storing each one.
(module
  (type (func (result i32)))         ;; the `finish` type
  (type (func))
  (type (func))
  ;; ... `(type (func))` repeated until ~100 KB
  (func $finish (export "finish") (type 0)
    i32.const 0
  )
)
