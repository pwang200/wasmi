//! Generates `finish_cases/08-load-stride-4096/case.wasm`. Page-stride walk of 8 MiB.

#[path = "../finish_common.rs"]
mod finish_common;

fn main() {
    let wasm = finish_common::module(&finish_common::fat_load_stride(4096));
    finish_common::write_case(
        "finish_cases/08-load-stride-4096/case.wasm",
        &wasm,
        "i32.load stride 4096",
    );
}
