//! Isolated self-recursion to just under Wasmi's 1000-frame cap.
//! `$finish` loops `call $f(998)`. Compare to 17 (static nest of 3).

#[path = "../finish_common.rs"]
mod finish_common;

/// `finish` + `$f(998)` … `$f(0)` is 1000 frames (the default recursion cap).
const DEPTH: u32 = 998;

fn main() {
    let wasm = finish_common::module_self_recurse(DEPTH);
    finish_common::write_case(
        "finish_cases/18-recurse/case.wasm",
        &wasm,
        &format!("self-recursion $f({DEPTH}) per loop"),
    );
}
