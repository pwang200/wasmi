;; Readable shape only. gen-finish-12-many-callees emits case.wasm.
;;
;; Tier B: many ~40-byte dummies (strict() average floor). `$finish` calls
;; each once, then returns 0. First call translates finish + every callee
;; (7 fuel / body byte). No infinite loop; expected runnable if 7*bytes < 1e6.
(module
  (func $dummy) ;; padded to ~40 bytes; × N
  (func $finish (export "finish") (result i32)
    (call $dummy) ;; × N
    i32.const 0
  )
)
