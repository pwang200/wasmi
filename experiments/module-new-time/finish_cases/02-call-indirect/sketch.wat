;; Readable shape only. gen-finish-02-call-indirect emits case.wasm.
;;
;; Same Finish loop as case 1 (30_000-local $fat, 8 MiB memory, burn 1e6 fuel),
;; but each iteration is `call_indirect` through a 1024-slot table instead of
;; a direct `call`. Extra work per fuel: bounds check + type check.
;;
;; Table min is the StoreLimit (1024). Active elem fills every slot with $fat
;; so instantiate copies 1024 funcrefs unmetered. The loop always indexes 0
;; so it never hits a null. Out-of-fuel is expected.
(module
  (type $t (func))
  (memory 128)
  (table 1024 funcref)
  (elem (i32.const 0) $fat $fat) ;; × 1024
  (func $fat (type $t)
    (local i32) ;; × 30000
  )
  (func $finish (export "finish") (result i32)
    (loop
      (call_indirect (type $t) (i32.const 0))
      (br 0)
    )
    i32.const 0
  )
)
