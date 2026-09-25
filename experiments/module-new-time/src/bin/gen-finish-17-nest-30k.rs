//! Four 30k-local functions in a call chain; `$finish` loops `call $a`.
//! Four × 240 KB ≈ 960 KB, under the default 1 MB value-stack cap.

#[path = "../finish_common.rs"]
mod finish_common;

use finish_common::{LOCALS, MEMORY_PAGES, fat_empty_locals, leb128_u32, write_case};

/// 4 × 240 KB plus `$finish` blows the default 1 MB value stack.
const NEST: u32 = 3;

fn section(id: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = vec![id];
    out.extend(leb128_u32(payload.len() as u32));
    out.extend_from_slice(payload);
    out
}

fn sized_body(body: &[u8]) -> Vec<u8> {
    let mut out = leb128_u32(body.len() as u32);
    out.extend_from_slice(body);
    out
}

fn leaf_body() -> Vec<u8> {
    fat_empty_locals(LOCALS)
}

fn caller_body(callee: u32) -> Vec<u8> {
    let mut body = fat_empty_locals(LOCALS);
    body.pop(); // drop the `end`
    body.push(0x10);
    body.extend(leb128_u32(callee));
    body.push(0x0b);
    body
}

fn finish_body() -> Vec<u8> {
    // loop { call $a (index 0); br 0 }; i32.const 0
    vec![
        0x00, 0x03, 0x40, 0x10, 0x00, 0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b,
    ]
}

fn module() -> Vec<u8> {
    // funcs 0..2 = a,b,c (c is leaf); func 3 = finish
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.extend(section(
        1,
        &[0x02, 0x60, 0x00, 0x00, 0x60, 0x00, 0x01, 0x7f],
    ));
    wasm.extend(section(3, &[0x04, 0x00, 0x00, 0x00, 0x01]));
    let mut memory = vec![0x01, 0x00];
    memory.extend(leb128_u32(MEMORY_PAGES));
    wasm.extend(section(5, &memory));
    let name = b"finish";
    let mut export = vec![0x01];
    export.extend(leb128_u32(name.len() as u32));
    export.extend_from_slice(name);
    export.extend_from_slice(&[0x00, 0x03]);
    wasm.extend(section(7, &export));
    let mut code = vec![0x04];
    code.extend(sized_body(&caller_body(1))); // a -> b
    code.extend(sized_body(&caller_body(2))); // b -> c
    code.extend(sized_body(&leaf_body())); // c
    code.extend(sized_body(&finish_body()));
    wasm.extend(section(10, &code));
    wasm
}

fn main() {
    let wasm = module();
    write_case(
        "finish_cases/17-nest-30k/case.wasm",
        &wasm,
        &format!("{NEST} nested 30k-local funcs"),
    );
}
