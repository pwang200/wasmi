;; Readable shape only. gen-02-br-table emits the ~100 KB case.wasm.
;;
;; EscrowCreate: OK (Module::new + export walk). No table/memory to fail limits.
;; EscrowFinish: OK. instantiate has nothing large to allocate; `finish` is tiny.
;; The fat function is never called, but Create still validates its body.
;;
;; Why this is expensive to load (relative to its file size):
;;
;; `br_table` is a jump table: one opcode, then a long list of target labels.
;; In the binary each target is a single byte (depth 0). The file is compact.
;;
;; Loading has to walk that list twice: once to decode it, once to check that
;; every target is a real label and that they all agree on what they do to the
;; stack. One cheap byte of table becomes a loop iteration in the validator.
;; No extra heap object per target (unlike the element-list case). The cost is
;; CPU per byte, not decompression into many allocations.
;;
;; The unused fat function is still fully checked at Module::new under
;; LazyTranslation. Calling `finish` does not translate or run the jump table.
(module
  (func $fat
    (block
      i32.const 0
      br_table 0 0 0 ;; … TARGET_COUNT times …  0
    )
  )
  (func $finish (export "finish") (result i32)
    i32.const 0
  )
)
