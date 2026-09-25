//! Generates `finish_cases/05-load-same/case.wasm`.
//!
//! `$fat` does `i32.load` of address 0 every call (L1 baseline). See `sketch.wat`.

#[path = "../finish_common.rs"]
mod finish_common;

fn main() {
    let wasm = finish_common::module(&finish_common::fat_load_same());
    finish_common::write_case(
        "finish_cases/05-load-same/case.wasm",
        &wasm,
        "i32.load address 0",
    );
}
