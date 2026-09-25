//! Generates `finish_cases/04-div-u64/case.wasm`.
//!
//! Isolated live `i64.div_u` in `$finish`. Same shape as case 3, wider idiv.

#[path = "../finish_common.rs"]
mod finish_common;

fn main() {
    let wasm = finish_common::module_finish_only(&finish_common::finish_live_i64_div());
    finish_common::write_case(
        "finish_cases/04-div-u64/case.wasm",
        &wasm,
        "live i64.div_u (i=i+1)/3 in $finish, 1 i64 local",
    );
}
