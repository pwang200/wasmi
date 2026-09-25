//! Isolated `memory.grow 1` at the 8 MiB cap (returns -1). No 30k locals.
//! `grow(0)` is an early return in Wasmi; the fail path is the real host call.

#[path = "../finish_common.rs"]
mod finish_common;

fn main() {
    let wasm = finish_common::module_finish_with_memory(&finish_common::finish_memory_grow());
    finish_common::write_case(
        "finish_cases/20-memory-grow/case.wasm",
        &wasm,
        "memory.grow 1 at 8 MiB cap",
    );
}
