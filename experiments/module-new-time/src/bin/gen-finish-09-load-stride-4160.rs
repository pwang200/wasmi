//! Generates `finish_cases/09-load-stride-4160/case.wasm`.
//! Odd stride 4096+64: page step plus one cache line (anti-prefetch).

#[path = "../finish_common.rs"]
mod finish_common;

fn main() {
    let wasm = finish_common::module(&finish_common::fat_load_stride(4160));
    finish_common::write_case(
        "finish_cases/09-load-stride-4160/case.wasm",
        &wasm,
        "i32.load stride 4160",
    );
}
