//! Real total T-A: nest-3 × 30k + unroll 32 + create-7 dummies. Counted; returns 1.

#[path = "../finish_common.rs"]
mod finish_common;

const NEST: u32 = 3;
const UNROLL: u32 = 32;
/// Measured ~198 fuel/iter + ~1.4k base. Leaves ~50k so `finish` returns 1.
const ITERS: u32 = 4_700;

fn main() {
    let (n, wasm) = finish_common::max_fit(|dummies| {
        finish_common::module_real_total_a(dummies, NEST, UNROLL, ITERS)
    });
    finish_common::write_case(
        "real_total/t-a-30k-nest-unroll/case.wasm",
        &wasm,
        &format!("{NEST}× 30k nest, {UNROLL}× unroll, {ITERS} iters, {n} unused combo dummies, returns 1"),
    );
}
