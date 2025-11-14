#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include "../include/wasm.h"
#include "../include/wasmi.h"

// Convert hex string to binary
size_t hex_to_binary(const char* hex, uint8_t* binary) {
    if (!hex || !binary) {
        return 0;
    }
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
                    printf("       ✅ Found function: %.*s\n", (int)name->size, name->data);
                    found_func = wasm_extern_as_func(current_extern);
                }
            }
            break;
        }
    }

    // Cleanup
    wasm_exporttype_vec_delete(&export_types);
    wasm_extern_vec_delete(&exports);
    
    return found_func;
}

int main() {
    printf("=== Memory Limit Demo ===\n\n");

    // 1. Create engine
    printf("1. Creating engine...\n");
    wasm_engine_t* engine = wasm_engine_new();
    printf("   ✅ Engine created\n");

    // 2. Create store
    printf("2. Creating store...\n");
    wasm_store_t* store = wasm_store_new(engine);
    printf("   ✅ Store created\n");

    // 3. Convert hex to binary WASM
    printf("3. Converting hex to WASM binary...\n");
    // This WASM module contains:
    // - 1 memory (starts at 0 pages, and grows 1 page, no maximum limit specified)
    // - 1 exported function "finish" that:
    //   - Calls memory.grow with 1 (grows memory by 1 page = 64KB)
    //   - Returns i32 value 1
    // The hex decodes to a valid WebAssembly module that will test memory growth limits
    const char* hex_wasm = "0061736d010000000105016000017f030201000503010000071302066d656d6f727902000666696e69736800000a0b010900410140001a41010b";

    uint8_t* wasm_binary = malloc(strlen(hex_wasm) / 2);
    if (!wasm_binary) {
        printf("   ❌ Failed to allocate memory for WASM binary\n");
        wasm_store_delete(store);
        wasm_engine_delete(engine);
        return 1;
    }
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
        wasm_store_delete(store);
        wasm_engine_delete(engine);
        return 1;
    }
    printf("   ✅ Module created\n");

    // === SCENARIO 1: Memory limit = 0 pages (should trap) ===
    printf("\n=== SCENARIO 1: Memory limit = 0 pages ===\n");
    
    // 5. Create store with 0-page memory limit
    printf("5. Creating store with 0-page memory limit...\n");
    wasm_store_t* store_0_pages = wasm_store_new_with_memory_max_pages(engine, 0);
    if (!store_0_pages) {
        printf("   ❌ Failed to create store with 0-page limit\n");
        wasm_module_delete(module);
        free(wasm_binary);
        wasm_byte_vec_delete(&binary);
        wasm_store_delete(store);
        wasm_engine_delete(engine);
        return 1;
    }
    printf("   ✅ Store created with 0-page memory limit\n");

    // 6. Create instance with 0-page limit
    printf("6. Creating instance with 0-page memory limit...\n");
    wasm_extern_vec_t imports1;
    wasm_extern_vec_new_empty(&imports1);
    wasm_instance_t* instance1 = wasm_instance_new(store_0_pages, module, &imports1, NULL);
    if (!instance1) {
        printf("   ❌ Failed to create instance\n");
        wasm_extern_vec_delete(&imports1);
        wasm_store_delete(store_0_pages);
        wasm_module_delete(module);
        free(wasm_binary);
        wasm_byte_vec_delete(&binary);
        wasm_store_delete(store);
        wasm_engine_delete(engine);
        return 1;
    }
    printf("   ✅ Instance created\n");

    // 7. Check memory before function call
    printf("7. Checking memory before function call...\n");
    wasm_extern_vec_t exports1;
    wasm_instance_exports(instance1, &exports1);
    
    wasm_memory_t* memory1 = NULL;
    for (size_t i = 0; i < exports1.size; i++) {
        if (wasm_extern_kind(exports1.data[i]) == WASM_EXTERN_MEMORY) {
            memory1 = wasm_extern_as_memory(exports1.data[i]);
            break;
        }
    }
    
    if (memory1) {
        printf("   📋 Memory before function call: %u pages\n", wasm_memory_size(memory1));
    }

    // 8. Test finish function with 0-page limit (should return -1)
    printf("8. Testing finish function with 0-page limit (memory.grow should return -1)...\n");
    wasm_func_t* finish_func1 = find_function_by_name(module, instance1, "finish");

    if (!finish_func1) {
        printf("   ❌ finish function not found\n");
    } else {
        wasm_val_vec_t args1 = WASM_EMPTY_VEC;
        wasm_val_vec_t results1;
        wasm_val_vec_new_uninitialized(&results1, 1);
        
        wasm_trap_t* trap1 = wasm_func_call(finish_func1, &args1, &results1);
        if (trap1) {
            printf("   ❌ Function trapped unexpectedly! Should return -1 instead\n");
            wasm_message_t message1;
            wasm_trap_message(trap1, &message1);
            printf("   📋 Trap message: %.*s\n", (int)message1.size, message1.data);
            wasm_byte_vec_delete(&message1);
            wasm_trap_delete(trap1);
        } else {
            printf("   📋 Function executed successfully\n");
            printf("   📋 Function result: %d\n", results1.data[0].of.i32);
            
            // Check if memory actually grew
            if (memory1) {
                wasm_memory_pages_t new_size = wasm_memory_size(memory1);
                printf("   📋 Memory after function call: %u pages\n", new_size);
                
                if (new_size == 0) {
                    printf("   ✅ Memory growth blocked as expected (stayed at 0 pages)\n");
                    printf("   ✅ Security limit working correctly\n");
                    printf("   📋 Function returned: %d (note: this may be -1 if function returns memory.grow result, or 1 if hardcoded)\n", results1.data[0].of.i32);
                } else if (new_size == 1) {
                    printf("   ❌ Memory grew unexpectedly despite 0-page limit\n");
                } else {
                    printf("   ❌ Unexpected behavior: memory=%u pages, result=%d\n", new_size, results1.data[0].of.i32);
                }
            }
        }
        
        wasm_val_vec_delete(&results1);
    }
    
    wasm_extern_vec_delete(&exports1);

    // Cleanup scenario 1
    wasm_extern_vec_delete(&imports1);
    wasm_instance_delete(instance1);
    wasm_store_delete(store_0_pages);

    // === SCENARIO 2: Memory limit = 1 page (should succeed) ===
    printf("\n=== SCENARIO 2: Memory limit = 1 page ===\n");
    
    // 5. Create store with 1-page memory limit
    printf("5. Creating store with 1-page memory limit...\n");
    wasm_store_t* store_1_page = wasm_store_new_with_memory_max_pages(engine, 1);
    if (!store_1_page) {
        printf("   ❌ Failed to create store with 1-page limit\n");
        wasm_module_delete(module);
        free(wasm_binary);
        wasm_byte_vec_delete(&binary);
        wasm_store_delete(store);
        wasm_engine_delete(engine);
        return 1;
    }
    printf("   ✅ Store created with 1-page memory limit (64KB)\n");

    // 6. Create instance with 1-page limit
    printf("6. Creating instance with 1-page memory limit...\n");
    wasm_extern_vec_t imports2;
    wasm_extern_vec_new_empty(&imports2);
    wasm_instance_t* instance2 = wasm_instance_new(store_1_page, module, &imports2, NULL);
    if (!instance2) {
        printf("   ❌ Failed to create instance\n");
        wasm_extern_vec_delete(&imports2);
        wasm_store_delete(store_1_page);
        wasm_module_delete(module);
        free(wasm_binary);
        wasm_byte_vec_delete(&binary);
        wasm_store_delete(store);
        wasm_engine_delete(engine);
        return 1;
    }
    printf("   ✅ Instance created\n");

    // 7. Check memory before function call
    printf("7. Checking memory before function call...\n");
    wasm_extern_vec_t exports2;
    wasm_instance_exports(instance2, &exports2);
    
    wasm_memory_t* memory2 = NULL;
    for (size_t i = 0; i < exports2.size; i++) {
        if (wasm_extern_kind(exports2.data[i]) == WASM_EXTERN_MEMORY) {
            memory2 = wasm_extern_as_memory(exports2.data[i]);
            break;
        }
    }
    
    if (memory2) {
        printf("   📋 Memory before function call: %u pages\n", wasm_memory_size(memory2));
    }

    // 8. Test finish function with 1-page limit (memory.grow should succeed)
    printf("8. Testing finish function with 1-page limit (memory.grow should succeed)...\n");
    wasm_func_t* finish_func2 = find_function_by_name(module, instance2, "finish");

    if (!finish_func2) {
        printf("   ❌ finish function not found\n");
    } else {
        wasm_val_vec_t args2 = WASM_EMPTY_VEC;
        wasm_val_vec_t results2;
        wasm_val_vec_new_uninitialized(&results2, 1);
        
        wasm_trap_t* trap2 = wasm_func_call(finish_func2, &args2, &results2);
        if (trap2) {
            printf("   ❌ Function trapped unexpectedly\n");
            wasm_message_t message2;
            wasm_trap_message(trap2, &message2);
            printf("   📋 Trap message: %.*s\n", (int)message2.size, message2.data);
            wasm_byte_vec_delete(&message2);
            wasm_trap_delete(trap2);
        } else {
            printf("   📋 Function executed successfully\n");
            printf("   📋 Function result: %d\n", results2.data[0].of.i32);
            
            // Check if memory actually grew
            if (memory2) {
                wasm_memory_pages_t new_size = wasm_memory_size(memory2);
                printf("   📋 Memory after function call: %u pages\n", new_size);
                
                if (new_size == 1) {
                    printf("   ✅ Memory grew to 1 page as expected (within limit)\n");
                    printf("   ✅ Security limit working correctly\n");
                } else {
                    printf("   ❌ Memory growth failed unexpectedly\n");
                }
            }
        }
        
        wasm_val_vec_delete(&results2);
    }
    
    wasm_extern_vec_delete(&exports2);

    // Cleanup scenario 2
    wasm_extern_vec_delete(&imports2);
    wasm_instance_delete(instance2);
    wasm_store_delete(store_1_page);

    // === SCENARIO 3: Default store (no limits, should succeed) ===
    printf("\n=== SCENARIO 3: Default store (no memory limits) ===\n");
    
    // 5. Create instance with default store (no memory limits)
    printf("5. Creating instance with default store (no memory limits)...\n");
    wasm_extern_vec_t imports3;
    wasm_extern_vec_new_empty(&imports3);
    wasm_instance_t* instance3 = wasm_instance_new(store, module, &imports3, NULL);
    if (!instance3) {
        printf("   ❌ Failed to create instance\n");
        wasm_extern_vec_delete(&imports3);
        wasm_module_delete(module);
        free(wasm_binary);
        wasm_byte_vec_delete(&binary);
        wasm_store_delete(store);
        wasm_engine_delete(engine);
        return 1;
    }
    printf("   ✅ Instance created\n");

    // 6. Check memory before function call
    printf("6. Checking memory before function call...\n");
    wasm_extern_vec_t exports3;
    wasm_instance_exports(instance3, &exports3);
    
    wasm_memory_t* memory3 = NULL;
    for (size_t i = 0; i < exports3.size; i++) {
        if (wasm_extern_kind(exports3.data[i]) == WASM_EXTERN_MEMORY) {
            memory3 = wasm_extern_as_memory(exports3.data[i]);
            break;
        }
    }
    
    if (memory3) {
        printf("   📋 Memory before function call: %u pages\n", wasm_memory_size(memory3));
    }

    // 7. Test finish function with default store (memory.grow should succeed)
    printf("7. Testing finish function with default store (memory.grow should succeed)...\n");
    wasm_func_t* finish_func3 = find_function_by_name(module, instance3, "finish");

    if (!finish_func3) {
        printf("   ❌ finish function not found\n");
    } else {
        wasm_val_vec_t args3 = WASM_EMPTY_VEC;
        wasm_val_vec_t results3;
        wasm_val_vec_new_uninitialized(&results3, 1);
        
        wasm_trap_t* trap3 = wasm_func_call(finish_func3, &args3, &results3);
        if (trap3) {
            printf("   ❌ Function trapped unexpectedly\n");
            wasm_message_t message3;
            wasm_trap_message(trap3, &message3);
            printf("   📋 Trap message: %.*s\n", (int)message3.size, message3.data);
            wasm_byte_vec_delete(&message3);
            wasm_trap_delete(trap3);
        } else {
            printf("   📋 Function executed successfully\n");
            printf("   📋 Function result: %d\n", results3.data[0].of.i32);
            
            // Check if memory actually grew
            if (memory3) {
                wasm_memory_pages_t new_size = wasm_memory_size(memory3);
                printf("   📋 Memory after function call: %u pages\n", new_size);
                
                if (new_size == 1) {
                    printf("   ✅ Memory grew to 1 page as expected (no limits applied)\n");
                    printf("   ✅ Default store behavior working correctly\n");
                } else {
                    printf("   ❌ Memory growth failed unexpectedly\n");
                }
            }
        }
        
        wasm_val_vec_delete(&results3);
    }
    
    wasm_extern_vec_delete(&exports3);

    // 8. Final cleanup
    printf("8. Final cleanup...\n");
    wasm_extern_vec_delete(&imports3);
    wasm_instance_delete(instance3);
    wasm_module_delete(module);
    wasm_byte_vec_delete(&binary);
    free(wasm_binary);
    wasm_store_delete(store);   // Clean up default store
    wasm_engine_delete(engine);
    printf("   ✅ Cleanup complete\n");

    printf("\n=== Memory Limit Demo Complete ===\n");
    return 0;
}