//! Generates `finish_cases/03-div-u/case.wasm`.
//!
//! Isolated live `i32.div_u` in `$finish`. One i32 local is the loop-carried
//! numerator so the translator cannot fold `1/1`. No `$fat`, no 30k locals.

#[path = "../finish_common.rs"]
mod finish_common;

fn main() {
    let wasm = finish_common::module_finish_only(&finish_common::finish_live_i32_div());
    finish_common::write_case(
        "finish_cases/03-div-u/case.wasm",
        &wasm,
        "live i32.div_u (i=i+1)/3 in $finish, 1 i32 local",
    );
}
