//! Same loop as case 1, `$fat` has 10_000 i32 locals.

#[path = "../finish_common.rs"]
mod finish_common;

fn main() {
    let wasm = finish_common::module(&finish_common::fat_empty_locals(10_000));
    finish_common::write_case(
        "finish_cases/15-hot-call-10k-locals/case.wasm",
        &wasm,
        "10000 i32 locals",
    );
}
