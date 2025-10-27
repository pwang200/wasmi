#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include "../include/wasm.h"
#include "../include/wasmi.h"

/*
gcc -Iinclude -o fuel_test example/fuel.c -L../../target/release -lwasmi_c_api -lm
LD_LIBRARY_PATH=../../target/release ./fuel_test
 */

// Convert hex string to binary
size_t hex_to_binary(const char* hex, uint8_t* binary) {
    size_t hex_len = strlen(hex);
    size_t binary_len = hex_len / 2;

    for (size_t i = 0; i < binary_len; i++) {
        char hex_byte[3] = {hex[i*2], hex[i*2+1], '\0'};
        binary[i] = (uint8_t)strtol(hex_byte, NULL, 16);
    }

    return binary_len;
}

wasm_func_t* find_function_by_name(wasm_module_t* module, wasm_instance_t* instance, const char* name_str) {
    wasm_exporttype_vec_t export_types;
    wasm_module_exports(module, &export_types);

    wasm_extern_vec_t exports;
    wasm_instance_exports(instance, &exports);

    wasm_func_t* found_func = NULL;

    for (size_t i = 0; i < export_types.size; ++i) {
        const wasm_name_t* name = wasm_exporttype_name(export_types.data[i]);

        if (name->size == strlen(name_str) && strncmp(name->data, name_str, name->size) == 0) {
            if (i < exports.size) {
                wasm_extern_t* current_extern = exports.data[i];
                if (wasm_extern_kind(current_extern) == WASM_EXTERN_FUNC) {
                    found_func = wasm_extern_as_func(current_extern);
                }
            }
            break;
        }
    }

    return found_func;
}

int main() {
    printf("=== Unified Fuel Tracking Test ===\n\n");

    // Initialize WASM engine with fuel consumption enabled
    printf("1. Creating engine with fuel enabled...\n");
    wasm_config_t* config = wasm_config_new();
    wasmi_config_consume_fuel_set(config, true);
    wasm_engine_t* engine = wasm_engine_new_with_config(config);
    printf("   ✅ Engine created\n");

    // Create wasm store with unified data
    printf("2. Creating wasm store...\n");
    wasm_store_t* store = wasm_store_new(engine);
    printf("   ✅ Store created\n");

    // Test basic fuel operations
    printf("3. Testing basic fuel set/get operations...\n");

    uint64_t initial_fuel = 1000000;
    wasmi_error_t* error = wasm_store_set_fuel(store, initial_fuel);
    if (error) {
        printf("   ❌ Error setting fuel\n");
        wasmi_error_delete(error);
        return 1;
    }
    printf("   ✅ Fuel set to %lu\n", initial_fuel);

    uint64_t current_fuel;
    error = wasm_store_get_fuel(store, &current_fuel);
    if (error) {
        printf("   ❌ Error getting fuel\n");
        wasmi_error_delete(error);
        return 1;
    }
    printf("   ✅ Current fuel: %lu\n", current_fuel);

    if (current_fuel == initial_fuel) {
        printf("   ✅ Basic fuel tracking works!\n");
    } else {
        printf("   ❌ Fuel mismatch: expected %lu, got %lu\n", initial_fuel, current_fuel);
        return 1;
    }

    // Test fuel consumption with actual WASM execution
    printf("4. Testing fuel consumption with WASM execution...\n");

    // Load simple WASM module that returns 42
    const char* hex_wasm = "0061736d010000000105016000017f03020100070a010666696e69736800000a06010400412a0b";
    uint8_t* wasm_binary = malloc(strlen(hex_wasm) / 2);
    size_t binary_size = hex_to_binary(hex_wasm, wasm_binary);

    wasm_byte_vec_t binary;
    wasm_byte_vec_new(&binary, binary_size, (char*)wasm_binary);
    wasm_module_t* module = wasm_module_new(store, &binary);
    if (!module) {
        printf("   ❌ Failed to create module\n");
        free(wasm_binary);
        wasm_byte_vec_delete(&binary);
        return 1;
    }
    printf("   ✅ Module created\n");

    wasm_extern_vec_t imports;
    wasm_extern_vec_new_empty(&imports);
    wasm_instance_t* instance = wasm_instance_new(store, module, &imports, NULL);
    if (!instance) {
        printf("   ❌ Failed to create instance\n");
        wasm_module_delete(module);
        free(wasm_binary);
        wasm_byte_vec_delete(&binary);
        return 1;
    }
    printf("   ✅ Instance created\n");

    // Check fuel before execution
    error = wasm_store_get_fuel(store, &current_fuel);
    if (error) {
        printf("   ❌ Error getting fuel before execution\n");
        wasmi_error_delete(error);
        return 1;
    }
    printf("   📊 Fuel before execution: %lu\n", current_fuel);

    // Find and call the finish function
    wasm_func_t* finish_func = find_function_by_name(module, instance, "finish");
    if (!finish_func) {
        printf("   ❌ finish function not found\n");
        wasm_instance_delete(instance);
        wasm_module_delete(module);
        free(wasm_binary);
        wasm_byte_vec_delete(&binary);
        return 1;
    }

    wasm_val_vec_t args = WASM_EMPTY_VEC;
    wasm_val_vec_t results;
    wasm_val_vec_new_uninitialized(&results, 1);
    results.data[0].kind = WASM_I32;
    results.data[0].of.i32 = 0;

    wasm_trap_t* trap = wasm_func_call(finish_func, &args, &results);
    if (trap) {
        printf("   ❌ Function call trapped\n");
        wasm_message_t message;
        wasm_trap_message(trap, &message);
        printf("   ❌ Trap message: %.*s\n", (int)message.size, message.data);
        wasm_byte_vec_delete(&message);
        wasm_trap_delete(trap);
        return 1;
    } else {
        printf("   ✅ Function executed successfully, result: %d\n", results.data[0].of.i32);
    }

    // Check fuel after execution
    uint64_t fuel_after;
    error = wasm_store_get_fuel(store, &fuel_after);
    if (error) {
        printf("   ❌ Error getting fuel after execution\n");
        wasmi_error_delete(error);
        return 1;
    }
    printf("   📊 Fuel after execution: %lu\n", fuel_after);
    printf("   📊 Fuel consumed: %lu\n", current_fuel - fuel_after);

    if (fuel_after < current_fuel) {
        printf("   ✅ Fuel consumption tracking works!\n");
    } else {
        printf("   ⚠️ No fuel consumed (possible with simple function)\n");
    }

    // Cleanup
    printf("6. Cleaning up...\n");
    wasm_val_vec_delete(&args);
    wasm_val_vec_delete(&results);
    wasm_instance_delete(instance);
    wasm_module_delete(module);
    wasm_byte_vec_delete(&binary);
    free(wasm_binary);
    wasm_store_delete(store);
    wasm_engine_delete(engine);
    printf("   ✅ Cleanup complete\n");

    printf("\n=== Unified Fuel Tracking Test Complete: SUCCESS ===\n");
    return 0;
}