;; Readable shape only. gen-08-many-types emits the ~100 KB case.wasm.
;;
;; EscrowCreate: OK. strict() caps params (32) and results, not the number of
;; types, and multi-value is off so each type has at most one result.
;; EscrowFinish: OK. Only `finish` runs.
;;
;; Why this escapes the 40-byte rule: the rule averages function *bodies*. A
;; type has no body, so the whole 100 KB can be type entries.
;;
;; Every type is distinct (i32/i64 params, shortest encodings first), about
;; 7.7k of them. Module::new validates each one and registers it with the
;; engine, which de-duplicates types: a distinct type is stored, not just
;; looked up.
(module
  (type (func))
  (type (func (result i32)))         ;; the `finish` type
  (type (func (result i64)))
  (type (func (param i32)))
  (type (func (param i64)))
  ;; ... every i32/i64 param list, with no result, i32 or i64, until ~100 KB
  (func $finish (export "finish") (type 1)
    i32.const 0
  )
)
