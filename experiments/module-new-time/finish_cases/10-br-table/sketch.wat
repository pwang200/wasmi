;; Readable shape only. gen-finish-10-br-table emits case.wasm.
;;
;; Tier B: first-call translation. `$finish` *is* the fat `br_table` (Create
;; case 2's body, but invoked). One translate of ~100 KB, then return 0.
;; Fuel: 7 per body byte (~630k) plus a cheap exec. Expected runnable.
(module
  (func $finish (export "finish") (result i32)
    (block
      i32.const 0
      br_table 0 0 0 ;; … TARGET_COUNT …
    )
    i32.const 0
  )
)
