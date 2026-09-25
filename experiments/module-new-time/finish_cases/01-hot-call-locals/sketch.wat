;; Readable shape only. gen-finish-01-hot-call-locals emits case.wasm.
;;
;; Finish target: instantiate + first `finish` call, 1_000_000 fuel.
;; Out-of-fuel is the expected end of `finish`.
;;
;; $fat declares 30_000 i32 locals (Wasmi translate cap) and does nothing.
;; Locals do not cost fuel. Each call zero-fills those slots (~240 KB).
;; $finish loops `call $fat` + `br 0` (2 fuel / iter) until the budget dies.
;;
;; `(memory 128)` is 8 MiB, the StoreLimit. Instantiation zeros it unmetered.
;; The file stays tiny; leftover bytes are for later combo cases.
(module
  (memory 128)
  (func $fat
    (local i32) ;; × 30000
  )
  (func $finish (export "finish") (result i32)
    (loop
      (call $fat)
      (br 0)
    )
    i32.const 0
  )
)
