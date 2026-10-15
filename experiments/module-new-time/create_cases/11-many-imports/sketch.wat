;; Readable shape only. gen-11-many-imports emits the ~100 KB case.wasm.
;;
;; EscrowCreate: OK in XRPL. Every import is a real host function with the
;; right signature, so XRPL's import screening accepts it. Duplicate imports
;; are legal wasm.
;; EscrowFinish: OK in XRPL (the linker resolves every copy). In this harness
;; instantiate fails because it defines no host functions; only Module::new
;; is meaningful here.
;;
;; Why this escapes the 40-byte rule: the rule averages function *bodies*. An
;; import has no body, so the whole 100 KB can be import entries.
;;
;; ~4.8k copies of host_lib::float_pow. It is the last entry in XRPL's
;; host-function list, so XRPL's screening walks the whole list for each copy.
;; Measured through XRPL's own check this stayed under 1 ms per 100 KB, so the
;; screening is not expensive.
(module
  (type (func (param i32 i32 i32 i32 i32 i32) (result i32)))  ;; float_pow
  (import "host_lib" "float_pow" (func (type 0)))
  (import "host_lib" "float_pow" (func (type 0)))
  ;; ... repeated until ~100 KB
  (func $finish (export "finish") (result i32)
    i32.const 0
  )
)
