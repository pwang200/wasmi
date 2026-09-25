;; Real total T-00. Fuel-burn floor that still returns 1. No 100 KB Create filler.
;;
;; Counted decrement: 5 fuel/iter (`local.get` + `sub` + `tee` + `br_if`).
;; Iteration count is chosen so 1e6 fuel is not exhausted.
(module
  (func $finish (export "finish") (result i32)
    (local i32)
    (local.set 0 (i32.const 158000))
    (loop
      (local.tee 0 (i32.sub (local.get 0) (i32.const 1)))
      (br_if 0)
    )
    i32.const 1
  )
)
