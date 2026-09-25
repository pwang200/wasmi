;; Readable shape only. gen-01-active-element emits the ~100 KB case.wasm.
;;
;; EscrowCreate: OK (Module::new + export walk). The table is unexported, so
;; XRPL screening does not see its size.
;; EscrowFinish: cannot run. Instantiate hits StoreLimits (table min >> 1024).
;;
;; Why this is expensive to load (relative to its file size):
;;
;; The element segment below is a long list of entries. In the binary file each
;; entry is just a single byte (an index that points at a function). That makes
;; the file very compact: N entries cost about N bytes on disk.
;;
;; But loading is not a byte-for-byte copy. For every entry the loader has to
;; (a) check that the index refers to a function that actually exists, and
;; (b) build a small stand-alone object in memory that represents "this slot is
;; initialized to that function". So one cheap input byte turns into a bounds
;; check plus a separate heap-allocated record.
;;
;; The result is a decompression effect: a small input forces a large amount of
;; per-entry work and many small allocations. The attacker pays one byte per
;; entry; the host pays validation plus object construction per entry. That
;; imbalance -- work done per input byte -- is what makes it a good stress test
;; for module loading, independent of running any code.
(module
  (func $finish (export "finish") (result i32)
    i32.const 0
  )
  ;; Unexported table, sized to hold the whole element list.
  (table $table ELEMENT_COUNT funcref)
  ;; Active segment: every entry points at the one function in this module, so
  ;; each entry encodes to a single byte in the binary.
  (elem (i32.const 0)
    $finish $finish ;; repeated ELEMENT_COUNT times
  )
)
