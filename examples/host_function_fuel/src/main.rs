use anyhow::Result;
use wasmi::*;

const HF_FUEL:u64=100;

fn main() -> Result<()> {
    println!("Wasmi Host Function Fuel Example");
    println!("=================================\n");

    // Step 1: Create engine with fuel metering enabled
    let mut config = Config::default();
    config.consume_fuel(true);
    let engine = Engine::new(&config);
    println!("✓ Created engine with fuel metering enabled");

    // Step 2: Create store with initial fuel
    let mut store = Store::new(&engine, ());
    let initial_fuel = 110u64;
    store.set_fuel(initial_fuel)?;
    println!("✓ Set initial fuel: {}", initial_fuel);

    // Step 3: Create host function with custom fuel cost
    let plus_1 = Func::wrap(&mut store, |mut caller: Caller<'_, ()>, a: i32| -> Result<i32, Error> {
        let fuel_before = caller.get_fuel()?;
        let new_fuel = fuel_before.saturating_sub(HF_FUEL);
        caller.set_fuel(new_fuel)?;
        println!("  [Host] plus_1({}) called, fuel: {} -> {}", a, fuel_before, new_fuel);
        Ok(a + 1)
    });
    println!("✓ Created host function 'plus_1' with custom fuel cost: {}", HF_FUEL);

    // Step 4: Define WAT module that imports plus_1 and exports finish
    let wat = r#"
        (module
            (import "env" "plus_1" (func $plus_1 (param i32) (result i32)))
            (func (export "finish") (result i32)
                ;; Call plus_1 three times: 0 -> 1 -> 2 -> 3
                i32.const 0
                call $plus_1
                call $plus_1
                call $plus_1
            )
        )
    "#;

    // Compile WAT to WASM
    let wasm = wat::parse_str(wat)?;
    println!("✓ Compiled WAT to WASM ({} bytes)", wasm.len());

    // Step 5: Create module and set up imports using Linker
    let module = Module::new(&engine, &wasm[..])?;
    println!("✓ Created module");

    // Use Linker to register host function
    let mut linker = <Linker<()>>::new(&engine);
    linker.define("env", "plus_1", plus_1)?;
    println!("✓ Registered host function in linker");

    // Step 6: Instantiate module
    let instance = linker.instantiate(&mut store, &module)?.start(&mut store)?;
    println!("✓ Instantiated module\n");

    // Step 7: Get the exported function
    let finish = instance.get_typed_func::<(), i32>(&store, "finish")?;
    println!("✓ Got exported function 'finish'\n");

    // Step 8: Call finish which calls plus_1 three times
    println!("Calling finish() which calls plus_1 three times...\n");

    let fuel_before = store.get_fuel()?;
    println!("Fuel before execution: {}", fuel_before);

    // Step 9: Execute the function
    let result = finish.call(&mut store, ())?;
    println!("\nFunction 'finish()' returned: {}", result);

    // Step 10: Check fuel after execution and report consumption
    let fuel_after = store.get_fuel()?;
    let fuel_consumed = fuel_before - fuel_after;

    println!("\nFuel after execution: {}", fuel_after);
    println!("Total fuel consumed: {}", fuel_consumed);
    // println!("  - Host function custom cost (3 calls): {}", custom_fuel_cost * 3);
    // println!("  - WASM instructions cost: {}", fuel_consumed - (custom_fuel_cost * 3));

    println!("\n✓ Example completed successfully!");

    Ok(())
}
