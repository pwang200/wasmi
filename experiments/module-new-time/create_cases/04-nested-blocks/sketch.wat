;; Readable shape only. gen-04-nested-blocks emits the ~100 KB case.wasm.
;; Do not assemble this WAT as-is: a real assembler will blow its stack on
;; DEPTH nested blocks. The generator writes the binary directly.
;;
;; EscrowCreate: OK. One fat unused function, tiny exported `finish`.
;; EscrowFinish: OK. Only `finish` runs. The nested function is not called.
;; (If you exported the fat function and called it, the engine call stack
;; could die; that is not this measurement.)
;;
;; Why this is expensive to load (relative to its file size):
;;
;; Each `block` opens a control-flow frame and each `end` closes one. Nesting
;; them (block inside block inside block …) makes the validator's control
;; stack grow to DEPTH frames at once. That is extra allocator traffic on
;; top of visiting every opcode.
;;
;; Sequential `block`/`end` at depth 1 reuses one frame and is *not* this
;; case. Same number of visits, much less peak memory.
;;
;; Encoding: 3 bytes per nest level (`block` + empty type + matching `end`).
(module
  (func $fat
    (block
      (block
        ;; … DEPTH times …
      )
    )
  )
  (func $finish (export "finish") (result i32)
    i32.const 0
  )
)
