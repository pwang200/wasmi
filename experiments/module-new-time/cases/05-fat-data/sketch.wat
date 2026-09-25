;; Readable shape only. gen-05-fat-data emits the ~100 KB case.wasm.
;;
;; EscrowCreate: OK. Memory is unexported, so the export walk does not see it.
;; EscrowFinish: OK under 8 MiB store limit (2 pages). Instantiate copies the blob.
;;
;; Why we expected this to be cheap: the payload is raw bytes. Create walks the
;; section and memcpy's it. No per-byte validator loop, no object per byte.
(module
  (memory 2)
  (data (i32.const 0) "… ~100 KB of zeros …")
  (func $finish (export "finish") (result i32)
    i32.const 0
  )
)
