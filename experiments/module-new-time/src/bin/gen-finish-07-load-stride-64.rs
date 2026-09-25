//! Generates `finish_cases/07-load-stride-64/case.wasm`. One load per cache line.

#[path = "../finish_common.rs"]
mod finish_common;

fn main() {
    let wasm = finish_common::module(&finish_common::fat_load_stride(64));
    finish_common::write_case(
        "finish_cases/07-load-stride-64/case.wasm",
        &wasm,
        "i32.load stride 64",
    );
}
