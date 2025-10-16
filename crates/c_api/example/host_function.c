#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <sha2.h>
#include "../include/wasm.h"

// A simple helper function for error handling
#define FATAL(msg, ...) { fprintf(stderr, "Fatal error: " msg "\n", ##__VA_ARGS__); exit(1); }

// Context for the host functions
typedef struct {
  wasm_store_t* store;
  wasm_memory_t* memory;
  void* user_data;
} host_context_t;

// Forward declaration of the host function
static wasm_trap_t* compute_sha512_half(
  void* env,
  const wasm_val_vec_t* args,
  wasm_val_vec_t* results
);

// Corresponds to the Rust sha512_half function.
void sha512_half(const uint8_t* data, size_t len, uint8_t* out) {
  SHA512_CTX ctx;
  SHA512_Init(&ctx);
  SHA512_Update(&ctx, data, len);
  SHA512_Final(out, &ctx);
}

// Corresponds to the Rust get_data function.
void get_data(const wasm_memory_t* memory, uint32_t pointer, uint32_t len, uint8_t** data, size_t* data_len) {
  uint8_t *mem_data = wasm_memory_data(memory);
  size_t mem_size = wasm_memory_size(memory);

  if ((uint64_t)pointer + len > mem_size) {
    FATAL("Read operation is out of memory bounds.");
  }

  *data = mem_data + pointer;
  *data_len = len;
}

// Corresponds to the Rust set_data function.
void set_data(wasm_memory_t* memory, const uint8_t* src_data, size_t src_len, uint32_t output_pointer, uint32_t output_len) {
  if (src_len != output_len) {
    FATAL("Data size does not match specified length.");
  }

  uint8_t *mem_data = wasm_memory_data(memory);
  size_t mem_size = wasm_memory_size(memory);

  uint64_t end_index = (uint64_t)output_pointer + output_len;
  if (end_index > mem_size) {
    FATAL("Write operation is out of memory bounds.");
  }

  memcpy(mem_data + output_pointer, src_data, src_len);
}

// Corresponds to the Rust compute_sha512_half host function.
static wasm_trap_t* compute_sha512_half(
  void* env,
  const wasm_val_vec_t* args,
  wasm_val_vec_t* results
) {
  host_context_t* context = (host_context_t*)env;

  uint32_t in_buf_ptr = wasm_val_i32(&args->data[0]);
  uint32_t in_buf_len = wasm_val_i32(&args->data[1]);
  uint32_t out_buf_ptr = wasm_val_i32(&args->data[2]);
  uint32_t out_buf_cap = wasm_val_i32(&args->data[3]);

  if (32 > out_buf_cap) {
    wasm_val_set_i32(&results->data[0], -1);
    return NULL;
  }
  if (in_buf_len > 1024) {
    wasm_val_set_i32(&results->data[0], -2);
    return NULL;
  }

  uint8_t* in_data = NULL;
  size_t in_len = 0;
  get_data(context->memory, in_buf_ptr, in_buf_len, &in_data, &in_len);

  uint8_t hash_full[64];
  sha512_half(in_data, in_len, hash_full);

  set_data(context->memory, hash_full, 32, out_buf_ptr, 32);

  wasm_val_set_i32(&results->data[0], 32);

  return NULL;
}

// Helper to load a file into a wasm_byte_vec_t.
static void read_wasm_file(const char* file_name, wasm_byte_vec_t* out) {
  FILE* file = fopen(file_name, "rb");
  if (!file) FATAL("Unable to open file: %s", file_name);
  fseek(file, 0, SEEK_END);
  long file_size = ftell(file);
  fseek(file, 0, SEEK_SET);
  wasm_byte_vec_new_uninitialized(out, file_size);
  if (fread(out->data, 1, file_size, file) != (size_t)file_size) {
    FATAL("Unable to read file: %s", file_name);
  }
  fclose(file);
}

int main(int argc, const char* argv[]) {
  wasm_engine_t* engine = wasm_engine_new();
  wasm_store_t* store = wasm_store_new(engine);

  wasm_byte_vec_t wasm_file;
  read_wasm_file("/home/pwang/wasm/rx-wasm-prototype/wat/sha_1KB_10kLoop.wasm", &wasm_file);
  wasm_module_t* module = wasm_module_new(store, &wasm_file);
  wasm_byte_vec_delete(&wasm_file);

  wasm_valtype_vec_t args_types, results_types;
  wasm_valtype_vec_new_uninitialized(&args_types, 4);
  wasm_valtype_vec_new_uninitialized(&results_types, 1);
  args_types.data[0] = wasm_valtype_new_i32();
  args_types.data[1] = wasm_valtype_new_i32();
  args_types.data[2] = wasm_valtype_new_i32();
  args_types.data[3] = wasm_valtype_new_i32();
  results_types.data[0] = wasm_valtype_new_i32();

  wasm_functype_t* func_type = wasm_functype_new(&args_types, &results_types);
  wasm_valtype_vec_delete(&args_types);
  wasm_valtype_vec_delete(&results_types);

  wasm_func_t* host_func = wasm_func_new(store, func_type, compute_sha512_half);
  wasm_functype_delete(func_type);

  wasm_extern_t* imports[] = { wasm_func_as_extern(host_func) };
  wasm_extern_vec_t import_vec = WASM_ARRAY_VEC(imports);
  wasm_instance_t* instance = wasm_instance_new(store, module, &import_vec, NULL);
  if (!instance) FATAL("Failed to instantiate module");

  // Get exports from the instance
  wasm_extern_vec_t exports;
  wasm_instance_exports(instance, &exports);

  // Manually find the memory export
  wasm_memory_t* memory = NULL;
  for (size_t i = 0; i < exports.size; ++i) {
      if (wasm_extern_kind(exports.data[i]) == WASM_EXTERN_MEMORY) {
          memory = wasm_extern_as_memory(exports.data[i]);
          break;
      }
  }
  if (!memory) FATAL("Failed to find memory export");

  // Find the 'finish' function export
  wasm_func_t* finish_func = NULL;
  for (size_t i = 0; i < exports.size; ++i) {
      // Assuming 'finish' is the first function export
      if (wasm_extern_kind(exports.data[i]) == WASM_EXTERN_FUNC) {
          finish_func = wasm_extern_as_func(exports.data[i]);
          break;
      }
  }
  if (!finish_func) FATAL("Failed to find 'finish' function export");

  // Create and set the context
  host_context_t context = {
      .store = store,
      .memory = memory,
      .user_data = NULL
  };
  wasm_func_set_env(host_func, &context, NULL); // Note: The third argument is a finalizer function, can be NULL

  wasm_val_vec_t args = WASM_EMPTY_VEC;
  wasm_val_vec_t results;
  wasm_val_vec_new_uninitialized(&results, 1);

  wasm_trap_t* trap = wasm_func_call(finish_func, &args, &results);
  if (trap) {
    wasm_byte_vec_t msg;
    wasm_trap_message(trap, &msg);
    fprintf(stderr, "Trap occurred: %s\n", msg.data);
    wasm_byte_vec_delete(&msg);
    wasm_trap_delete(trap);
    FATAL("Wasm execution trapped");
  }

  printf("Result: %d\n", wasm_val_i32(&results.data[0]));

  wasm_val_vec_delete(&results);
  wasm_extern_vec_delete(&exports);
  wasm_func_delete(host_func);
  wasm_instance_delete(instance);
  wasm_module_delete(module);
  wasm_store_delete(store);
  wasm_engine_delete(engine);

  return 0;
}