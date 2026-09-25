//! Isolated nest: `a → b → c`, each 0 extra locals. `$finish` loops `call $a`.
//! Compare to 13 (one empty call). Not the 30k-zero trick.

#[path = "../finish_common.rs"]
mod finish_common;

const NEST: u32 = 3;

fn main() {
    let wasm = finish_common::module_nest(NEST);
    finish_common::write_case(
        "finish_cases/17-nest/case.wasm",
        &wasm,
        &format!("{NEST} nested empty funcs"),
    );
}
