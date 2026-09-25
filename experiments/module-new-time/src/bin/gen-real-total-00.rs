//! Real total T-00: cheapest counted drain of ~1e6 fuel, then return 1. No Create pad.

#[path = "../finish_common.rs"]
mod finish_common;

/// Measured ~6 fuel/iter + ~160 base. Leaves ~50k so `finish` returns 1.
const ITERS: u32 = 158_000;

fn main() {
    let wasm = finish_common::module_finish_only(&finish_common::finish_counted_br_loop(ITERS));
    finish_common::write_case(
        "real_total/t-00-br-loop/case.wasm",
        &wasm,
        &format!("counted decrement × {ITERS}, returns 1, no Create filler"),
    );
}
