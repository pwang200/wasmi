;; Readable shape only. gen-finish-11-nested-blocks emits case.wasm.
;; Do not assemble this WAT as-is: a deep nest can blow an assembler's stack.
;;
;; Tier B: `$finish` is deeply nested `block`/`end`, then `i32.const 0`.
;; First-call translate builds the control stack. If the host thread stack
;; dies, lower DEPTH. Out-of-fuel is unlikely (one exec after translate).
(module
  (func $finish (export "finish") (result i32)
    (block
      (block
        ;; … DEPTH …
      )
    )
    i32.const 0
  )
)
