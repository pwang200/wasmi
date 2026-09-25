//! Real total T-0: create-7 combo dummies to 100 KB + `finish` returns 1.

#[path = "../finish_common.rs"]
mod finish_common;

fn main() {
    let (n, wasm) = finish_common::max_fit(finish_common::module_real_total_0);
    finish_common::write_case(
        "real_total/t-0-control/case.wasm",
        &wasm,
        &format!("{n} unused combo dummies, finish = i32.const 1"),
    );
}
