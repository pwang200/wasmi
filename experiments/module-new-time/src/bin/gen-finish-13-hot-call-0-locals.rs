//! Same loop as case 1, but `$fat` has no extra locals.

#[path = "../finish_common.rs"]
mod finish_common;

fn main() {
    let wasm = finish_common::module(&finish_common::fat_empty_locals(0));
    finish_common::write_case(
        "finish_cases/13-hot-call-0-locals/case.wasm",
        &wasm,
        "0 extra locals",
    );
}
