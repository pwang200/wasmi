//! Isolated unroll: 32 `call` of empty `$fat` (0 locals) per loop.
//! Compare to 13 (one empty call per loop). Not the 30k-zero trick.

#[path = "../finish_common.rs"]
mod finish_common;

const UNROLL: u32 = 32;

fn main() {
    let wasm = finish_common::module_with_finish(
        &finish_common::fat_empty_locals(0),
        &finish_common::finish_unroll_calls(UNROLL),
    );
    finish_common::write_case(
        "finish_cases/16-unroll/case.wasm",
        &wasm,
        &format!("{UNROLL} calls of empty $fat per loop"),
    );
}
