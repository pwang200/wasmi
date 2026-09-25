//! Real total T-B: unroll-32 `call_indirect` + create-7 dummies. Counted; returns 1.

#[path = "../finish_common.rs"]
mod finish_common;

const UNROLL: u32 = 32;
/// Measured ~102 fuel/iter + ~1.6k base. Leaves ~50k so `finish` returns 1.
const ITERS: u32 = 9_200;

fn main() {
    let (n, wasm) = finish_common::max_fit(|dummies| {
        finish_common::module_real_total_b(dummies, UNROLL, ITERS)
    });
    finish_common::write_case(
        "real_total/t-b-call-indirect-unroll/case.wasm",
        &wasm,
        &format!("{UNROLL}× call_indirect/loop, {ITERS} iters, {n} unused combo dummies, returns 1"),
    );
}
