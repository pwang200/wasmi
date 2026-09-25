//! Same 30k-local `$fat` as case 1, but `$finish` calls it 32 times per loop.

#[path = "../finish_common.rs"]
mod finish_common;

const UNROLL: u32 = 32;

fn main() {
    let wasm = finish_common::module_with_finish(
        &finish_common::fat_empty_locals(finish_common::LOCALS),
        &finish_common::finish_unroll_calls(UNROLL),
    );
    finish_common::write_case(
        "finish_cases/16-unroll-30k/case.wasm",
        &wasm,
        &format!("{} calls of 30k-local $fat per loop", UNROLL),
    );
}
