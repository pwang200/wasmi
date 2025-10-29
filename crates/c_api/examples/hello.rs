// Simplified but proper translation of hello.c + fuel.c using C-API bridge functions
// Loads hex WASM code, executes it, and measures fuel consumption

use wasmi_c_api::{wasm_config_new, wasmi_config_consume_fuel_set, wasm_engine_new_with_config, wasm_store_new, wasm_store_set_fuel, wasm_store_get_fuel, wasm_module_new, wasm_instance_new, wasm_instance_exports, wasm_byte_vec_new, wasm_extern_vec_new_empty, wasm_val_vec_new_uninitialized, wasm_val_vec_t, wasm_byte_vec_t, wasm_extern_vec_t, wasm_func_call, wasm_extern_as_func, wasmi_config_compilation_mode_set, wasmi_compilation_mode_t};

// Convert hex string to binary (same logic as hello.c)
fn hex_to_binary(hex: &str) -> Vec<u8> {
    let mut binary = Vec::new();
    let chars: Vec<char> = hex.chars().collect();
    
    for i in (0..chars.len()).step_by(2) {
        if i + 1 < chars.len() {
            let hex_byte = format!("{}{}", chars[i], chars[i + 1]);
            if let Ok(byte) = u8::from_str_radix(&hex_byte, 16) {
                binary.push(byte);
            }
        }
    }
    binary
}

fn main() {
    println!("=== Hello WASM C-API Bridge Translation (Simplified) ===\n");

    unsafe {
        // 1. Create engine with fuel enabled (same as fuel.c)
        println!("1. Creating engine with fuel enabled...");
        let mut config = wasm_config_new();
        wasmi_config_consume_fuel_set(&mut *config, true);
        wasmi_config_compilation_mode_set(&mut *config, wasmi_compilation_mode_t::WASMI_COMPILATION_MODE_LAZY_TRANSLATION);
        let engine = wasm_engine_new_with_config(config);
        println!("   ✅ Engine created");

        // 2. Create store (same as hello.c)
        println!("2. Creating store...");
        let mut store = wasm_store_new(&*engine);
        println!("   ✅ Store created");

        // 3. Set initial fuel (your new function!)
        println!("3. Setting initial fuel...");
        let initial_fuel = 1000000u64;
        let error = wasm_store_set_fuel(&mut *store, initial_fuel);
        match error {
            Some(_) => {
                println!("   ❌ Error setting fuel");
                return;
            }
            None => println!("   ✅ Fuel set to {}", initial_fuel),
        }

        // 5. Convert hex to binary WASM (same as hello.c)
        println!("4. Converting hex to WASM binary...");
        //let hex_wasm = "0061736d010000000105016000017f03020100070a010666696e69736800000a06010400412a0b";
        let hex_wasm = "0061736d010000000108026000006000017f03030200010606017f0141000b070a010666696e69736800010801000a29022201017f41a08d0621000340200041016b2100200041004a0d000b410141016a24000b040023000b";

        let wasm_binary = hex_to_binary(hex_wasm);
        println!("   ✅ Converted {} bytes", wasm_binary.len());

        // 6. Create WASM module from binary (same as hello.c)
        println!("5. Creating WASM module...");
        
        // Create byte vector using the C-API
        let mut binary_vec = std::mem::zeroed::<wasm_byte_vec_t>();
        wasm_byte_vec_new(&mut binary_vec, wasm_binary.len(), wasm_binary.as_ptr());
        
        let module = wasm_module_new(&mut *store, &binary_vec);
        let module = match module {
            Some(m) => m,
            None => {
                println!("   ❌ Failed to create module");
                return;
            }
        };
        println!("   ✅ Module created");

        // 7. Create instance (same as hello.c)
        println!("6. Creating instance...");
        
        // Create empty imports vector using the C-API
        let mut imports = std::mem::zeroed::<wasm_extern_vec_t>();
        wasm_extern_vec_new_empty(&mut imports);
        
        let instance = wasm_instance_new(&mut *store, &*module, &imports, None);
        let mut instance = match instance {
            Some(i) => i,
            None => {
                println!("   ❌ Failed to create instance");
                return;
            }
        };
        println!("   ✅ Instance created");

        // 8. Get exports and find the "finish" function
        println!("7. Getting instance exports...");
        
        let mut exports = std::mem::zeroed::<wasm_extern_vec_t>();
        wasm_instance_exports(&mut instance, &mut exports);
        println!("   ✅ Got {} exports", exports.as_slice().len());

        // 9. Find the "finish" function (assuming it's the first export)
        println!("8. Finding 'finish' function...");
        
        let finish_func = if exports.as_slice().len() > 0 {
            // Get a mutable reference to the first export
            let export_ptr = exports.as_slice().as_ptr() as *mut Option<Box<_>>;
            let export_opt = unsafe { &mut *export_ptr };
            if let Some(ref mut export_ref) = export_opt {
                wasm_extern_as_func(export_ref)
            } else {
                println!("   ❌ First export is null");
                return;
            }
        } else {
            println!("   ❌ No exports found");
            return;
        };
        
        let finish_func = match finish_func {
            Some(f) => f,
            None => {
                println!("   ❌ First export is not a function");
                return;
            }
        };
        println!("   ✅ Found function");

        // 11. Call the function
        println!("9. Calling 'finish' function...");
        
        // Create empty parameters and result vectors
        let params = std::mem::zeroed::<wasm_val_vec_t>();
        let mut results = std::mem::zeroed::<wasm_val_vec_t>();
        wasm_val_vec_new_uninitialized(&mut results, 1); // Expect 1 i32 result
        
        let trap = wasm_func_call(finish_func, &params, &mut results);
        
        if !trap.is_null() {
            println!("   ❌ Function call trapped");
            return;
        }
        
        // Get the result value
        let result_slice = results.as_slice();
        if let Some(result_val) = result_slice.get(0) {
            // The hex WASM code should return 42 (0x2a)
            println!("   ✅ Function executed successfully. result = {:?}", result_val.of.i32);
        } else {
            println!("   ❌ No result returned");
            return;
        }

        // 12. Check fuel after execution (to show fuel consumption)
        let mut fuel_after = 0u64;
        let error = wasm_store_get_fuel(&*store, &mut fuel_after);
        match error {
            Some(_) => {
                println!("   ❌ Error getting fuel after execution");
                return;
            }
            None => {
                println!("   📊 Fuel after execution: {}", fuel_after);
                println!("   📊 Fuel consumed: {}", initial_fuel - fuel_after);
            }
        }
    }
}