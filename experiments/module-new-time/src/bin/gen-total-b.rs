//! Total step 1 kernel B: `call_indirect` only. No 100 KB Create filler.

#[path = "../finish_common.rs"]
mod finish_common;

fn main() {
    let wasm = finish_common::module_call_indirect();
    finish_common::write_case(
        "total_cases/b-call-indirect/case.wasm",
        &wasm,
        "call_indirect, no Create filler",
    );
}
