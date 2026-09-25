//! Isolated sequential +4 walk of 8 MiB. One cursor local, not 30k.

#[path = "../finish_common.rs"]
mod finish_common;

fn main() {
    let wasm = finish_common::module(&finish_common::fat_load_stride(4));
    finish_common::write_case(
        "finish_cases/06-load-stride-4/case.wasm",
        &wasm,
        "i32.load stride 4",
    );
}
