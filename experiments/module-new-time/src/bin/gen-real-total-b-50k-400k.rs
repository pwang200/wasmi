//! Real total T-B at 50 KB / 400k fuel: same kernel and filler as T-B, smaller caps.
//! Counted; returns 1. Time with `module-new-time <case.wasm> 400000`.

#[path = "../finish_common.rs"]
mod finish_common;

const TARGET_SIZE: usize = 50_000;
const UNROLL: u32 = 32;
/// ~102 fuel/iter + ~1.6k base (as T-B). Leaves ~20k of 400k so `finish` returns 1.
const ITERS: u32 = 3_700;

fn main() {
    let (n, wasm) = finish_common::max_fit_within(
        |dummies| finish_common::module_real_total_b(dummies, UNROLL, ITERS),
        10_000,
        TARGET_SIZE,
    );
    finish_common::write_case(
        "real_total/t-b-50k-400k/case.wasm",
        &wasm,
        &format!("{UNROLL}× call_indirect/loop, {ITERS} iters, {n} unused combo dummies, returns 1"),
    );
}
