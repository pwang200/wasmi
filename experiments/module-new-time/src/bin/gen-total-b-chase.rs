//! B payload probe: instantiate writes a Sattolo cycle; `finish` only chases it.
//! No 30k locals. Packed i32 next-addrs (working set ≈ data size, L2 on this CPU).

#[path = "../finish_common.rs"]
mod finish_common;

const UNROLL: u32 = 32;

fn main() {
    let (n, wasm) = finish_common::max_fit_up_to(|nodes| {
        if nodes < 2 {
            return vec![0; finish_common::TARGET_SIZE + 1];
        }
        finish_common::module_pointer_chase(nodes, UNROLL)
    }, 50_000);
    finish_common::write_case(
        "total_cases/b-pointer-chase/case.wasm",
        &wasm,
        &format!("{n} nodes, {UNROLL}× load/loop, data at instantiate"),
    );
}
