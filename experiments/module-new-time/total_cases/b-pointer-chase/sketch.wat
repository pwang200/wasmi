;; B payload probe. Permutation is an active data segment (instantiate, no fuel).
;; `finish` only does `cur = i32.load(cur)` (dependent, no prefetch).
;; Packed i32s: working set ≈ file data, L2 on this machine (8 MiB < P-core L2).
(module
  (memory 128)
  (data (i32.const 0) "...sattolo next-addrs...")
  (func $finish (export "finish") (result i32)
    (local $cur i32)
    (loop
      (local.set $cur (i32.load (local.get $cur))) ;; × 32
      (br 0)
    )
    i32.const 0
  )
)
