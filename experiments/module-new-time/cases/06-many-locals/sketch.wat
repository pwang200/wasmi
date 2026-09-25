;; Readable shape only. gen-06-many-locals emits the ~100 KB case.wasm.
;;
;; EscrowCreate: OK. 50_000 locals is wasmparser's cap; the group is a few bytes.
;; The rest of the 100 KB is nops in the same unused function so the file is fair.
;; EscrowFinish: OK. Only `finish` runs (4 bytes, no extra locals).
;;
;; Wasmi translation allows only 30_000 locals. That hits on first *translate*
;; of this function, not at Module::new. We keep the 50k locals off `finish`.
;;
;; Why we expected Create to be cheap: one compact local group + memset-style
;; bookkeeping, then a nop walk. Not 50k heap objects.
(module
  (func $fat
    (local i32) ;; × 50000 in the binary
    nop nop ;; … padding …
  )
  (func $finish (export "finish") (result i32)
    i32.const 0
  )
)
