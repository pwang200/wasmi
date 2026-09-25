;; Readable shape only. gen-03-func-spam emits the ~100 KB case.wasm.
;;
;; EscrowCreate: OK if average function-body size stays at least 40 bytes
;; (strict() only applies that once all bodies total ≥ 1 KiB). Tiny-function
;; spam below that floor is the cheap reject path — this case sits on the floor.
;; EscrowFinish: OK. No large table or memory. Only `finish` is called.
;;
;; Why this is expensive to load (relative to its file size):
;;
;; The cost is not one dense instruction. It is *many functions*. Each function
;; is a separate compile unit: set up a validator, copy the body, walk a few
;; opcodes, tear down. A 40-byte body is just above the average-size gate, so
;; you pack as many of those setups as a 100 KB module allows (~2.4k).
;;
;; Same total bytes as one fat function, but far more per-function overhead
;; (and one heap copy per body, since each is larger than 22 bytes).
;; Dummy functions are never called; Create still validates every one.
(module
  (func $dummy) ;; padded to ~40 bytes; repeated FUNC_COUNT-1 times
  (func $finish (export "finish") (result i32)
    i32.const 0 ;; plus padding so this body is the same length
  )
)
