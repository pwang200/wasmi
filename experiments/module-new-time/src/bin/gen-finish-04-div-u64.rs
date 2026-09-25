//! Generates `finish_cases/04-div-u64/case.wasm`.
//!
//! Case 1's loop, but `$fat` does `i64.div_u` of 1/1. See `sketch.wat`.

#[path = "../finish_common.rs"]
mod finish_common;

fn main() {
    let wasm = finish_common::module(&finish_common::fat_i64_div_u());
    finish_common::write_case(
        "finish_cases/04-div-u64/case.wasm",
        &wasm,
        "i64.div_u in $fat",
    );
}
