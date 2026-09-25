//! Shared bits for Finish-case generators (included via `#[path]`).
#![allow(dead_code)]

use std::{fs, path::PathBuf};

pub const LOCALS: u32 = 30_000;
pub const MEMORY_PAGES: u32 = 128;
/// 8 MiB − 4, 4-byte aligned. Keeps `i32.load` inside the memory.
pub const ADDR_MASK: u32 = 0x007F_FFFC;

pub fn leb128_u32(mut value: u32) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            return out;
        }
    }
}

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

fn locals_header_n(n: u32) -> Vec<u8> {
    if n == 0 {
        return vec![0x00];
    }
    let mut body = vec![0x01];
    body.extend(leb128_u32(n));
    body.push(0x7f);
    body
}

/// Empty `$fat` with `n` i32 locals (0 means no local group).
pub fn fat_empty_locals(n: u32) -> Vec<u8> {
    let mut body = locals_header_n(n);
    body.push(0x0b);
    body
}

fn finish_body() -> Vec<u8> {
    finish_unroll_calls(1)
}

/// `loop { call 0; … × n; br 0 }; i32.const 0`
pub fn finish_unroll_calls(n: u32) -> Vec<u8> {
    let mut body = vec![0x00, 0x03, 0x40];
    for _ in 0..n {
        body.extend_from_slice(&[0x10, 0x00]);
    }
    body.extend_from_slice(&[0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b]);
    body
}

/// One i32 local. Loop: `i = i+1; drop(i / 3)`. Numerator is loop-carried so
/// the translator cannot fold the `idiv` away. No `$fat`, no 30k locals.
pub fn finish_live_i32_div() -> Vec<u8> {
    vec![
        0x01, 0x01, 0x7f, // 1× i32 local
        0x03, 0x40,       // loop
        0x20, 0x00,       // local.get 0
        0x41, 0x01,       // i32.const 1
        0x6a,             // i32.add
        0x22, 0x00,       // local.tee 0
        0x41, 0x03,       // i32.const 3
        0x6e,             // i32.div_u
        0x1a,             // drop
        0x0c, 0x00,       // br 0
        0x0b,             // end loop
        0x41, 0x00,       // i32.const 0
        0x0b,             // end
    ]
}

/// Same as `finish_live_i32_div`, but `i64`.
pub fn finish_live_i64_div() -> Vec<u8> {
    vec![
        0x01, 0x01, 0x7e, // 1× i64 local
        0x03, 0x40,       // loop
        0x20, 0x00,       // local.get 0
        0x42, 0x01,       // i64.const 1
        0x7c,             // i64.add
        0x22, 0x00,       // local.tee 0
        0x42, 0x03,       // i64.const 3
        0x80,             // i64.div_u
        0x1a,             // drop
        0x0c, 0x00,       // br 0
        0x0b,             // end loop
        0x41, 0x00,       // i32.const 0
        0x0b,             // end
    ]
}

/// One `i32.load` of address 0. No extra locals.
pub fn fat_load_same() -> Vec<u8> {
    let mut body = locals_header_n(0);
    body.extend_from_slice(&[0x41, 0x00, 0x28, 0x02, 0x00, 0x1a, 0x0b]);
    body
}

/// Load `*cursor`, then `cursor = (cursor + stride) & ADDR_MASK` stored at mem[0].
/// One i32 local is the cursor (not a 30k-zero).
pub fn fat_load_stride(stride: u32) -> Vec<u8> {
    let mut body = locals_header_n(1);
    // c = mem[0]
    body.extend_from_slice(&[0x41, 0x00, 0x28, 0x02, 0x00, 0x21, 0x00]);
    // drop mem[c]
    body.extend_from_slice(&[0x20, 0x00, 0x28, 0x02, 0x00, 0x1a]);
    // mem[0] = (c + stride) & MASK
    body.push(0x41);
    body.push(0x00);
    body.push(0x20);
    body.push(0x00);
    body.push(0x41);
    body.extend(leb128_u32(stride));
    body.push(0x6a);
    body.push(0x41);
    body.extend(leb128_u32(ADDR_MASK));
    body.extend_from_slice(&[0x71, 0x36, 0x02, 0x00, 0x0b]);
    body
}

/// `call $callee` then end. No extra locals.
pub fn fat_call(callee: u32) -> Vec<u8> {
    let mut body = vec![0x00, 0x10];
    body.extend(leb128_u32(callee));
    body.push(0x0b);
    body
}

pub fn module(fat: &[u8]) -> Vec<u8> {
    module_with_finish(fat, &finish_body())
}

pub fn module_with_finish(fat: &[u8], finish: &[u8]) -> Vec<u8> {
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.extend(section(
        1,
        &[0x02, 0x60, 0x00, 0x00, 0x60, 0x00, 0x01, 0x7f],
    ));
    wasm.extend(section(3, &[0x02, 0x00, 0x01]));
    let mut memory = vec![0x01, 0x00];
    memory.extend(leb128_u32(MEMORY_PAGES));
    wasm.extend(section(5, &memory));
    let name = b"finish";
    let mut export = vec![0x01];
    export.extend(leb128_u32(name.len() as u32));
    export.extend_from_slice(name);
    export.extend_from_slice(&[0x00, 0x01]);
    wasm.extend(section(7, &export));
    let mut code = vec![0x02];
    code.extend(sized_body(fat));
    code.extend(sized_body(finish));
    wasm.extend(section(10, &code));
    wasm
}

/// `$f(n)` calls `$f(n-1)` down to 0. No extra locals (the `i32` param is the counter).
pub fn recurse_body() -> Vec<u8> {
    vec![
        0x00, // no extra locals
        0x20, 0x00, // local.get 0
        0x04, 0x40, // if
        0x20, 0x00, // local.get 0
        0x41, 0x01, // i32.const 1
        0x6b, // i32.sub
        0x10, 0x00, // call 0
        0x0b, // end if
        0x0b, // end
    ]
}

/// `loop { call $f(depth); br 0 }; i32.const 0`
pub fn finish_recurse(depth: u32) -> Vec<u8> {
    let mut body = vec![0x00, 0x03, 0x40, 0x41];
    body.extend(leb128_u32(depth));
    body.extend_from_slice(&[0x10, 0x00, 0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b]);
    body
}

/// `loop { drop(memory.grow 1); br 0 }; i32.const 0`
pub fn finish_memory_grow() -> Vec<u8> {
    vec![
        0x00, 0x03, 0x40, 0x41, 0x01, 0x40, 0x00, 0x1a, 0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b,
    ]
}

pub fn module_finish_only(finish: &[u8]) -> Vec<u8> {
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.extend(section(1, &[0x01, 0x60, 0x00, 0x01, 0x7f]));
    wasm.extend(section(3, &[0x01, 0x00]));
    let name = b"finish";
    let mut export = vec![0x01];
    export.extend(leb128_u32(name.len() as u32));
    export.extend_from_slice(name);
    export.extend_from_slice(&[0x00, 0x00]);
    wasm.extend(section(7, &export));
    let mut code = vec![0x01];
    code.extend(sized_body(finish));
    wasm.extend(section(10, &code));
    wasm
}

/// `$f: (i32) -> ()` plus `$finish` that loops `call $f(depth)`.
pub fn module_self_recurse(depth: u32) -> Vec<u8> {
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.extend(section(
        1,
        &[0x02, 0x60, 0x01, 0x7f, 0x00, 0x60, 0x00, 0x01, 0x7f],
    ));
    wasm.extend(section(3, &[0x02, 0x00, 0x01]));
    let name = b"finish";
    let mut export = vec![0x01];
    export.extend(leb128_u32(name.len() as u32));
    export.extend_from_slice(name);
    export.extend_from_slice(&[0x00, 0x01]);
    wasm.extend(section(7, &export));
    let mut code = vec![0x02];
    code.extend(sized_body(&recurse_body()));
    code.extend(sized_body(&finish_recurse(depth)));
    wasm.extend(section(10, &code));
    wasm
}

/// `$finish` only, plus 8 MiB memory (for `memory.grow`).
pub fn module_finish_with_memory(finish: &[u8]) -> Vec<u8> {
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.extend(section(1, &[0x01, 0x60, 0x00, 0x01, 0x7f]));
    wasm.extend(section(3, &[0x01, 0x00]));
    let mut memory = vec![0x01, 0x00];
    memory.extend(leb128_u32(MEMORY_PAGES));
    wasm.extend(section(5, &memory));
    let name = b"finish";
    let mut export = vec![0x01];
    export.extend(leb128_u32(name.len() as u32));
    export.extend_from_slice(name);
    export.extend_from_slice(&[0x00, 0x00]);
    wasm.extend(section(7, &export));
    let mut code = vec![0x01];
    code.extend(sized_body(finish));
    wasm.extend(section(10, &code));
    wasm
}

/// `depth` empty-local funcs: 0 calls 1, …, leaf is `depth-1`. `$finish` loops `call 0`.
pub fn module_nest(depth: u32) -> Vec<u8> {
    assert!(depth >= 1);
    let n_code = depth + 1; // + finish
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.extend(section(
        1,
        &[0x02, 0x60, 0x00, 0x00, 0x60, 0x00, 0x01, 0x7f],
    ));
    let mut funcs = leb128_u32(n_code);
    for _ in 0..depth {
        funcs.push(0x00);
    }
    funcs.push(0x01);
    wasm.extend(section(3, &funcs));
    let mut memory = vec![0x01, 0x00];
    memory.extend(leb128_u32(MEMORY_PAGES));
    wasm.extend(section(5, &memory));
    let name = b"finish";
    let mut export = vec![0x01];
    export.extend(leb128_u32(name.len() as u32));
    export.extend_from_slice(name);
    export.push(0x00);
    export.extend(leb128_u32(depth));
    wasm.extend(section(7, &export));
    let mut code = leb128_u32(n_code);
    for i in 0..depth.saturating_sub(1) {
        code.extend(sized_body(&fat_call(i + 1)));
    }
    code.extend(sized_body(&fat_empty_locals(0)));
    code.extend(sized_body(&finish_body()));
    wasm.extend(section(10, &code));
    wasm
}

pub fn write_case(rel_wasm: &str, wasm: &[u8], note: &str) {
    let out: PathBuf = [env!("CARGO_MANIFEST_DIR"), rel_wasm].iter().collect();
    fs::create_dir_all(out.parent().unwrap()).unwrap();
    fs::write(&out, wasm).unwrap();
    println!("wrote {}: {} bytes, {note}", out.display(), wasm.len());
}
