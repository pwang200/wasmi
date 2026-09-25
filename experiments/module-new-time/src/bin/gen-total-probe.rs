//! One-shot probes for total step 1: nest on A, unroll/nest on B.

#[path = "../finish_common.rs"]
mod finish_common;

fn main() {
    let a_nest = finish_common::module_a_30k_nest(3, 1);
    finish_common::write_case(
        "total_cases/_probe/a-30k-nest/case.wasm",
        &a_nest,
        "A: 3× 30k nest, 1 call/loop",
    );
    let a_both = finish_common::module_a_30k_nest(3, 32);
    finish_common::write_case(
        "total_cases/_probe/a-30k-nest-unroll/case.wasm",
        &a_both,
        "A: 3× 30k nest, 32× unroll",
    );
    let b_unroll = finish_common::module_b_unroll(32);
    finish_common::write_case(
        "total_cases/_probe/b-unroll/case.wasm",
        &b_unroll,
        "B: 32× call_indirect/loop",
    );
    let b_nest = finish_common::module_b_nest(3, 1);
    finish_common::write_case(
        "total_cases/_probe/b-nest/case.wasm",
        &b_nest,
        "B: call_indirect to 3-deep empty nest",
    );
    let br_loop = finish_common::module_finish_only(&[
        0x00, 0x03, 0x40, 0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b,
    ]);
    finish_common::write_case(
        "total_cases/_probe/br-loop/case.wasm",
        &br_loop,
        "loop { br 0 } only",
    );
    let b_both = finish_common::module_b_nest(3, 32);
    finish_common::write_case(
        "total_cases/_probe/b-nest-unroll/case.wasm",
        &b_both,
        "B: 32× call_indirect to 3-deep empty nest",
    );
}
