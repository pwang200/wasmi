;; Isolated `memory.grow`. Memory is already 128 pages (8 MiB store cap),
;; so `grow 1` fails and returns -1. Loop until 1e6 fuel.
(module
  (memory 128)
  (func $finish (export "finish") (result i32)
    (loop
      (drop (memory.grow (i32.const 1)))
      (br 0)
    )
    i32.const 0
  )
)
