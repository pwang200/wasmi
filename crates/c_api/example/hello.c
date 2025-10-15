#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include "../include/wasm.h"

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
    // 1. Get the export types from the module to find the name
    wasm_exporttype_vec_t export_types;
    wasm_module_exports(module, &export_types);

    // 2. Get the actual exported items from the instance
    wasm_extern_vec_t exports;
    wasm_instance_exports(instance, &exports);

    wasm_func_t* found_func = NULL;

    // Iterate through the export types to find the name
    for (size_t i = 0; i < export_types.size; ++i) {
        const wasm_name_t* name = wasm_exporttype_name(export_types.data[i]);

        // Check if the name matches
        if (name->size == strlen(name_str) && strncmp(name->data, name_str, name->size) == 0) {
            // Found the name! Now, find its corresponding extern in the instance exports
            // The index of the name in the module exports corresponds to the index
            // of the extern in the instance exports. This is guaranteed by the spec.
            if (i < exports.size) {
                wasm_extern_t* current_extern = exports.data[i];
                if (wasm_extern_kind(current_extern) == WASM_EXTERN_FUNC) {
                    printf("✅ Found function: %.*s\n", (int)name->size, name->data);
                    found_func = wasm_extern_as_func(current_extern);
                } else {
                    printf("⚠️ Found a matching export, but it's not a function.\n");
                }
            } else {
                 printf("❌ Mismatch between module exports and instance exports.\n");
            }
            break; // Exit the loop once the name is found
        }
    }

//    // Clean up all vectors
//    wasm_exporttype_vec_delete(&export_types);
//    wasm_extern_vec_delete(&exports);

    return found_func;
}


int main() {
    printf("=== Simple Hello WASM Test (No Fuel) ===\n\n");

    // 1. Create basic engine
    printf("1. Creating engine...\n");
    wasm_engine_t* engine = wasm_engine_new();
    printf("   ✅ Engine created\n");

    // 2. Create basic store (no fuel parameters)
    printf("2. Creating store...\n");
    wasm_store_t* store = wasm_store_new(engine);
    printf("   ✅ Store created\n");

    // 3. Convert hex to binary WASM
    printf("3. Converting hex to WASM binary...\n");
    const char* hex_wasm = "0061736d010000000105016000017f03020100070a010666696e69736800000a06010400412a0b";
    uint8_t* wasm_binary = malloc(strlen(hex_wasm) / 2);
    size_t binary_size = hex_to_binary(hex_wasm, wasm_binary);
    printf("   ✅ Converted %zu bytes\n", binary_size);

    // 4. Create WASM module from binary
    printf("4. Creating WASM module...\n");
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

    // 5. Create instance
    printf("5. Creating instance...\n");
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

    // 6. Find finish function by name
    printf("6. Finding finish function...\n");
    wasm_func_t* finish_func = find_function_by_name(module, instance, "finish");

    if (!finish_func) {
        printf("   ❌ finish function not found\n");
        wasm_instance_delete(instance);
        wasm_module_delete(module);
        free(wasm_binary);
        wasm_byte_vec_delete(&binary);
        return 1;
    }

    printf("   ✅ Function found at address: %p\n", (void*)finish_func);

    // 7. Get function type to verify signature
    printf("7. Checking function signature...\n");
    printf("   📋 About to call wasm_func_type()...\n");
    wasm_functype_t* func_type = wasm_func_type(finish_func);
    printf("   📋 wasm_func_type() returned: %p\n", (void*)func_type);
    const wasm_valtype_vec_t* params = wasm_functype_params(func_type);
    const wasm_valtype_vec_t* returns = wasm_functype_results(func_type);
    printf("   ✅ Function takes %zu params, returns %zu values\n", params->size, returns->size);
    // 8. Call finish function
    printf("8. Calling finish() function...\n");

    // An empty vector for arguments
    wasm_val_vec_t args = WASM_EMPTY_VEC;
    wasm_val_vec_t results;
    wasm_val_vec_new_uninitialized(&results, returns->size);

//    printf("   📋 Calling function... %p, %p, %d\n", &results.data[0], &(results.data[0].of), sizeof results.data[0]);
    results.data[0].of.i64 = 0;

    wasm_trap_t* trap = wasm_func_call(finish_func, &args, &results);
    if (trap) {
        printf("   ❌ Function call trapped\n");
        wasm_message_t message;
        wasm_trap_message(trap, &message);
        printf("   ❌ Trap message: %.*s\n", (int)message.size, message.data);
        wasm_byte_vec_delete(&message);
        wasm_trap_delete(trap);
    } else {
        printf("   ✅ Function executed successfully\n");
        printf("   ✅ Result: %p, %d\n", &(results.data[0].of.i32), (results.data[0].of.i32));
//        if (results.size > 0) {
//            printf("   📋 Result type: %d\n", results.data[0].kind);
//            printf("   ✅ Result: %d\n", results.data[0].of.i32);
//        } else {
//            printf("   ❌ No results returned\n");
//        }
    }

    // 9. Cleanup
    printf("9. Cleaning up...\n");
    wasm_functype_delete(func_type);
    wasm_val_vec_delete(&args);
    wasm_val_vec_delete(&results);
    wasm_instance_delete(instance);
    wasm_module_delete(module);
    wasm_byte_vec_delete(&binary);
    free(wasm_binary);
    wasm_store_delete(store);
    wasm_engine_delete(engine);
    printf("   ✅ Cleanup complete\n");

    printf("\n=== Simple Test Complete: SUCCESS ===\n");
    return 0;
}