;; Readable shape only. gen-finish-06-load-stride-4 emits case.wasm.
;;
;; Cursor at mem[0] persists across calls (locals reset each call).
;; Each $fat: load *cursor, cursor = (cursor + 4) & (8MiB-4).
;; Sequential stream; prefetcher-friendly. Out-of-fuel expected.
(module
  (memory 128)
  (func $fat
    (local $c i32) ;; plus more i32 to 30000
    (local.set $c (i32.load (i32.const 0)))
    (drop (i32.load (local.get $c)))
    (i32.store (i32.const 0)
      (i32.and (i32.add (local.get $c) (i32.const 4)) (i32.const 0x7ffffc)))
  )
  (func $finish (export "finish") (result i32)
    (loop
      (call $fat)
      (br 0)
    )
    i32.const 0
  )
)
