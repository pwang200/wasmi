;; Readable shape only. gen-09-many-exports emits the ~100 KB case.wasm.
;;
;; EscrowCreate: OK. strict() does not cap the number of exports. Names must be
;; unique and valid UTF-8; all 1-, 2- and 3-byte ASCII names qualify.
;; EscrowFinish: OK. Only `finish` runs, but instantiate also builds the
;; instance's export map, one entry per name.
;;
;; Why this escapes the 40-byte rule: the rule averages function *bodies*. An
;; export has no body, so the whole 100 KB can be export entries.
;;
;; One function, exported as "finish" plus about 19.4k short aliases. Module::new
;; checks every name is unique and stores each one in the module's export map.
(module
  (func $finish (result i32)
    i32.const 0
  )
  (export "finish" (func $finish))
  (export "\00" (func $finish))
  (export "\01" (func $finish))
  ;; ... every 1-byte ASCII name, then 2-byte names, then 3-byte, until ~100 KB
)
