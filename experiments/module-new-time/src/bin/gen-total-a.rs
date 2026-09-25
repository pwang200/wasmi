//! Total step 1 kernel A: 30k-local `$fat` + 32× unroll. No 100 KB Create filler.

#[path = "../finish_common.rs"]
mod finish_common;

const UNROLL: u32 = 32;

fn main() {
    let wasm = finish_common::module_with_finish(
        &finish_common::fat_empty_locals(finish_common::LOCALS),
        &finish_common::finish_unroll_calls(UNROLL),
    );
    finish_common::write_case(
        "total_cases/a-30k-unroll/case.wasm",
        &wasm,
        &format!("30k $fat, {UNROLL}× unroll, no Create filler"),
    );
}
