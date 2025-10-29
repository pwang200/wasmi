(module
  ;; 1. Define a mutable global variable to store the final result
  (global $result (mut i32) (i32.const 0))

  ;; 2. Define the function that consumes fuel and performs initialization
  (func $init (local $counter i32) ;; Define a local variable for the loop counter
    ;; Set the loop counter's starting value (e.g., 100,000)
    (i32.const 100000)
    (local.set $counter)

    ;; Start a loop that executes many times to consume fuel
    (loop $fuel_loop

      ;; Decrement the counter
      (local.get $counter)
      (i32.const 1)
      (i32.sub)
      (local.set $counter)

      ;; Check if counter is zero
      (local.get $counter)
      (i32.const 0)
      (i32.gt_s)       ;; Returns 1 (true) if $counter > 0

      ;; If true, jump back to the start of $fuel_loop (consume more fuel)
      (br_if $fuel_loop)
    )

    ;; 3. Final calculation (observable side-effect that wasn't optimized out)
    (i32.const 1)
    (i32.const 1)
    (i32.add)             ;; Stack is now [2]
    (global.set $result)  ;; Sets $result to 2.

    ;; The loop and the final instruction sets ensure high fuel consumption
    ;; and an observable result, both metered by Wasmi during instantiation.
  )

  ;; 4. The CRUCIAL AUTOMATIC START SECTION
  (start $init)

  ;; 5. Define and EXPORT the function to retrieve the result
  (func (export "finish") (result i32)
    (global.get $result)
  )
)