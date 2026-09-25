//! Same loop as case 1, `$fat` has 1_000 i32 locals.

#[path = "../finish_common.rs"]
mod finish_common;

fn main() {
    let wasm = finish_common::module(&finish_common::fat_empty_locals(1_000));
    finish_common::write_case(
        "finish_cases/14-hot-call-1k-locals/case.wasm",
        &wasm,
        "1000 i32 locals",
    );
}
