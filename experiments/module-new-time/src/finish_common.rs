//! Shared bits for Finish-case generators (included via `#[path]`).
#![allow(dead_code)]

use std::{fs, path::PathBuf};

pub const LOCALS: u32 = 30_000;
pub const MEMORY_PAGES: u32 = 128;
/// 8 MiB − 4, 4-byte aligned. Keeps `i32.load` inside the memory.
pub const ADDR_MASK: u32 = 0x007F_FFFC;

/// Signed LEB128 for `i32.const`. Unsigned LEB of 64..=127 is read as negative.
pub fn leb128_i32(mut value: i32) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let mut byte = (value as u8) & 0x7f;
        value >>= 7;
        let done = (value == 0 && byte & 0x40 == 0) || (value == -1 && byte & 0x40 != 0);
        if !done {
            byte |= 0x80;
        }
        out.push(byte);
        if done {
            return out;
        }
    }
}

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

/// Cheapest 1-fuel loop: `loop { br 0 }; i32.const 0`. `loop`/`end` are 0 fuel.
/// Infinite — OOFs. Prefer `finish_counted` when the host must see a return.
pub fn finish_br_loop() -> Vec<u8> {
    vec![0x00, 0x03, 0x40, 0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b]
}

/// Run `payload` `iters` times (`iters >= 1`), then `i32.const 1`.
/// `(local i32) i32.const iters local.set 0 (loop payload; i=i-1; br_if 0) i32.const 1`
pub fn finish_counted(iters: u32, payload: &[u8]) -> Vec<u8> {
    assert!(iters >= 1);
    let mut body = vec![0x01, 0x01, 0x7f, 0x41];
    body.extend(leb128_i32(iters as i32));
    body.extend_from_slice(&[0x21, 0x00, 0x03, 0x40]);
    body.extend_from_slice(payload);
    body.extend_from_slice(&[
        0x20, 0x00, 0x41, 0x01, 0x6b, 0x22, 0x00, 0x0d, 0x00, 0x0b, 0x41, 0x01, 0x0b,
    ]);
    body
}

pub fn finish_counted_br_loop(iters: u32) -> Vec<u8> {
    finish_counted(iters, &[])
}

pub fn finish_counted_unroll_calls(iters: u32, unroll: u32) -> Vec<u8> {
    let mut payload = Vec::new();
    for _ in 0..unroll {
        payload.extend_from_slice(&[0x10, 0x00]);
    }
    finish_counted(iters, &payload)
}

pub fn finish_counted_unroll_call_indirect(iters: u32, unroll: u32) -> Vec<u8> {
    let mut payload = Vec::new();
    for _ in 0..unroll {
        payload.extend_from_slice(&[0x41, 0x00, 0x11, 0x00, 0x00]);
    }
    finish_counted(iters, &payload)
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

/// Sattolo cycle of `n` next-addresses (`4*perm[i]` at slot `i`).
fn sattolo_addrs(n: u32) -> Vec<u8> {
    let n = n as usize;
    let mut idx: Vec<u32> = (0..n as u32).collect();
    let mut rng: u32 = 0xC0FF_EE01;
    for i in (1..n).rev() {
        rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
        let j = (rng as usize) % i;
        idx.swap(i, j);
    }
    let mut out = Vec::with_capacity(n * 4);
    for slot in idx {
        out.extend_from_slice(&(slot * 4).to_le_bytes());
    }
    out
}

fn finish_chase(unroll: u32) -> Vec<u8> {
    let mut body = vec![
        0x01, 0x01, 0x7f, // 1× i32 cur (starts 0)
        0x03, 0x40,       // loop
    ];
    for _ in 0..unroll {
        body.extend_from_slice(&[
            0x20, 0x00, // local.get 0
            0x28, 0x02, 0x00, // i32.load
            0x21, 0x00, // local.set 0
        ]);
    }
    body.extend_from_slice(&[0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b]);
    body
}

/// Packed pointer chase: permutation is an active data segment (instantiate, no fuel).
/// `$finish` only does `cur = mem[cur]`.
pub fn module_pointer_chase(nodes: u32, unroll: u32) -> Vec<u8> {
    let data = sattolo_addrs(nodes);
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
    code.extend(sized_body(&finish_chase(unroll)));
    wasm.extend(section(10, &code));
    let mut data_sec = vec![0x01, 0x00, 0x41, 0x00, 0x0b];
    data_sec.extend(leb128_u32(data.len() as u32));
    data_sec.extend_from_slice(&data);
    wasm.extend(section(11, &data_sec));
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

pub const TARGET_SIZE: usize = 100_000;
const DUMMY_BODY_LEN: usize = 39;
const UNUSED_LOCALS: u32 = 50_000;
const TABLE_LEN: u32 = 1024;

/// Unused Create filler: 50k i32 locals + short `br_table` (create case 7 dummy).
pub fn dummy_combo_body() -> Vec<u8> {
    let mut locals = vec![0x01];
    locals.extend(leb128_u32(UNUSED_LOCALS));
    locals.push(0x7f);
    let mut tail = vec![0x02, 0x40, 0x41, 0x00, 0x0e];
    let n = (DUMMY_BODY_LEN - locals.len() - tail.len() - 4) as u32;
    debug_assert!(n < 128);
    tail.extend(leb128_u32(n));
    tail.extend(std::iter::repeat_n(0x00, n as usize));
    tail.extend_from_slice(&[0x00, 0x0b, 0x0b]);
    let mut body = locals;
    body.extend(tail);
    debug_assert_eq!(body.len(), DUMMY_BODY_LEN);
    body
}

pub fn max_fit(build: impl Fn(u32) -> Vec<u8>) -> (u32, Vec<u8>) {
    max_fit_up_to(build, 10_000)
}

pub fn max_fit_up_to(build: impl Fn(u32) -> Vec<u8>, high0: u32) -> (u32, Vec<u8>) {
    let (mut low, mut high) = (0u32, high0);
    while low < high {
        let mid = (low + high).div_ceil(2);
        if build(mid).len() <= TARGET_SIZE {
            low = mid;
        } else {
            high = mid - 1;
        }
    }
    (low, build(low))
}

fn export_finish(func: u32) -> Vec<u8> {
    let name = b"finish";
    let mut export = vec![0x01];
    export.extend(leb128_u32(name.len() as u32));
    export.extend_from_slice(name);
    export.push(0x00);
    export.extend(leb128_u32(func));
    export
}

fn pad_body(mut body: Vec<u8>) -> Vec<u8> {
    debug_assert_eq!(body.last().copied(), Some(0x0b));
    while body.len() < DUMMY_BODY_LEN {
        let end = body.pop().unwrap();
        body.push(0x01);
        body.push(end);
    }
    body
}

fn finish_call_indirect() -> Vec<u8> {
    vec![
        0x00, 0x03, 0x40, 0x41, 0x00, 0x11, 0x00, 0x00, 0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b,
    ]
}

/// Isolated `call_indirect` loop (kernel B, no Create filler).
pub fn module_call_indirect() -> Vec<u8> {
    module_total_b(0)
}

fn finish_unroll_call_indirect(n: u32) -> Vec<u8> {
    let mut body = vec![0x00, 0x03, 0x40];
    for _ in 0..n {
        body.extend_from_slice(&[0x41, 0x00, 0x11, 0x00, 0x00]);
    }
    body.extend_from_slice(&[0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b]);
    body
}

fn fat_30k_call(callee: u32) -> Vec<u8> {
    let mut body = fat_empty_locals(LOCALS);
    body.pop();
    body.push(0x10);
    body.extend(leb128_u32(callee));
    body.push(0x0b);
    body
}

/// Kernel A + nest: `depth` × 30k-local funcs, `$finish` unrolls `call 0`.
pub fn module_a_30k_nest(depth: u32, unroll: u32) -> Vec<u8> {
    assert!(depth >= 1);
    let n_code = depth + 1;
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
    wasm.extend(section(7, &export_finish(depth)));
    let mut code = leb128_u32(n_code);
    for i in 0..depth.saturating_sub(1) {
        code.extend(sized_body(&fat_30k_call(i + 1)));
    }
    code.extend(sized_body(&fat_empty_locals(LOCALS)));
    code.extend(sized_body(&finish_unroll_calls(unroll)));
    wasm.extend(section(10, &code));
    wasm
}

/// Kernel B + unroll: `n` `call_indirect`s per loop, empty `$fat`.
pub fn module_b_unroll(n: u32) -> Vec<u8> {
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.extend(section(
        1,
        &[0x02, 0x60, 0x00, 0x00, 0x60, 0x00, 0x01, 0x7f],
    ));
    wasm.extend(section(3, &[0x02, 0x00, 0x01]));
    let mut table = vec![0x01, 0x70, 0x00];
    table.extend(leb128_u32(TABLE_LEN));
    wasm.extend(section(4, &table));
    let mut memory = vec![0x01, 0x00];
    memory.extend(leb128_u32(MEMORY_PAGES));
    wasm.extend(section(5, &memory));
    wasm.extend(section(7, &export_finish(1)));
    let mut element = vec![0x01, 0x00, 0x41, 0x00, 0x0b];
    element.extend(leb128_u32(TABLE_LEN));
    element.extend(std::iter::repeat_n(0x00, TABLE_LEN as usize));
    wasm.extend(section(9, &element));
    let mut code = vec![0x02];
    code.extend(sized_body(&fat_empty_locals(0)));
    code.extend(sized_body(&finish_unroll_call_indirect(n)));
    wasm.extend(section(10, &code));
    wasm
}

/// Kernel B + nest: `call_indirect` to `$a → … → leaf`, each empty.
pub fn module_b_nest(depth: u32, unroll: u32) -> Vec<u8> {
    assert!(depth >= 1);
    let n_code = depth + 1;
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
    let mut table = vec![0x01, 0x70, 0x00];
    table.extend(leb128_u32(TABLE_LEN));
    wasm.extend(section(4, &table));
    let mut memory = vec![0x01, 0x00];
    memory.extend(leb128_u32(MEMORY_PAGES));
    wasm.extend(section(5, &memory));
    wasm.extend(section(7, &export_finish(depth)));
    let mut element = vec![0x01, 0x00, 0x41, 0x00, 0x0b];
    element.extend(leb128_u32(TABLE_LEN));
    element.extend(std::iter::repeat_n(0x00, TABLE_LEN as usize));
    wasm.extend(section(9, &element));
    let mut code = leb128_u32(n_code);
    for i in 0..depth.saturating_sub(1) {
        code.extend(sized_body(&fat_call(i + 1)));
    }
    code.extend(sized_body(&fat_empty_locals(0)));
    code.extend(sized_body(&finish_unroll_call_indirect(unroll)));
    wasm.extend(section(10, &code));
    wasm
}

/// Kernel A + Create filler: `$fat` is 30k locals (func 0), unused combo dummies, unroll-32 `finish`.
pub fn module_total_a(dummy_count: u32, unroll: u32) -> Vec<u8> {
    let n_code = dummy_count + 2;
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.extend(section(
        1,
        &[0x02, 0x60, 0x00, 0x00, 0x60, 0x00, 0x01, 0x7f],
    ));
    let mut funcs = leb128_u32(n_code);
    funcs.push(0x00);
    funcs.extend(std::iter::repeat_n(0x00, dummy_count as usize));
    funcs.push(0x01);
    wasm.extend(section(3, &funcs));
    let mut memory = vec![0x01, 0x00];
    memory.extend(leb128_u32(MEMORY_PAGES));
    wasm.extend(section(5, &memory));
    wasm.extend(section(7, &export_finish(dummy_count + 1)));
    let mut code = leb128_u32(n_code);
    code.extend(sized_body(&fat_empty_locals(LOCALS)));
    let dummy = sized_body(&dummy_combo_body());
    for _ in 0..dummy_count {
        code.extend_from_slice(&dummy);
    }
    code.extend(sized_body(&finish_unroll_calls(unroll)));
    wasm.extend(section(10, &code));
    wasm
}

/// Real total T-0: create-7 combo dummies + trivial `finish` (control).
pub fn module_real_total_0(dummy_count: u32) -> Vec<u8> {
    let n_code = dummy_count + 1;
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.extend(section(
        1,
        &[0x02, 0x60, 0x00, 0x00, 0x60, 0x00, 0x01, 0x7f],
    ));
    let mut funcs = leb128_u32(n_code);
    funcs.extend(std::iter::repeat_n(0x00, dummy_count as usize));
    funcs.push(0x01);
    wasm.extend(section(3, &funcs));
    wasm.extend(section(7, &export_finish(dummy_count)));
    let mut code = leb128_u32(n_code);
    let dummy = sized_body(&dummy_combo_body());
    for _ in 0..dummy_count {
        code.extend_from_slice(&dummy);
    }
    code.extend(sized_body(&pad_body(vec![0x00, 0x41, 0x01, 0x0b])));
    wasm.extend(section(10, &code));
    wasm
}

/// Real total T-A: nest-`depth` × 30k + unroll + unused combo dummies to 100 KB.
/// Counted `iters` so `finish` returns 1 instead of OOF.
pub fn module_real_total_a(dummy_count: u32, depth: u32, unroll: u32, iters: u32) -> Vec<u8> {
    assert!(depth >= 1);
    let n_code = dummy_count + depth + 1;
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.extend(section(
        1,
        &[0x02, 0x60, 0x00, 0x00, 0x60, 0x00, 0x01, 0x7f],
    ));
    let mut funcs = leb128_u32(n_code);
    for _ in 0..depth {
        funcs.push(0x00);
    }
    funcs.extend(std::iter::repeat_n(0x00, dummy_count as usize));
    funcs.push(0x01);
    wasm.extend(section(3, &funcs));
    let mut memory = vec![0x01, 0x00];
    memory.extend(leb128_u32(MEMORY_PAGES));
    wasm.extend(section(5, &memory));
    wasm.extend(section(7, &export_finish(dummy_count + depth)));
    let mut code = leb128_u32(n_code);
    for i in 0..depth.saturating_sub(1) {
        code.extend(sized_body(&pad_body(fat_30k_call(i + 1))));
    }
    code.extend(sized_body(&pad_body(fat_empty_locals(LOCALS))));
    let dummy = sized_body(&dummy_combo_body());
    for _ in 0..dummy_count {
        code.extend_from_slice(&dummy);
    }
    code.extend(sized_body(&pad_body(finish_counted_unroll_calls(
        iters, unroll,
    ))));
    wasm.extend(section(10, &code));
    wasm
}

/// Real total T-B: empty `$fat` + unroll `call_indirect` + unused combo dummies.
/// Counted `iters` so `finish` returns 1 instead of OOF.
pub fn module_real_total_b(dummy_count: u32, unroll: u32, iters: u32) -> Vec<u8> {
    let n_code = dummy_count + 2;
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.extend(section(
        1,
        &[0x02, 0x60, 0x00, 0x00, 0x60, 0x00, 0x01, 0x7f],
    ));
    let mut funcs = leb128_u32(n_code);
    funcs.push(0x00);
    funcs.extend(std::iter::repeat_n(0x00, dummy_count as usize));
    funcs.push(0x01);
    wasm.extend(section(3, &funcs));
    let mut table = vec![0x01, 0x70, 0x00];
    table.extend(leb128_u32(TABLE_LEN));
    wasm.extend(section(4, &table));
    let mut memory = vec![0x01, 0x00];
    memory.extend(leb128_u32(MEMORY_PAGES));
    wasm.extend(section(5, &memory));
    wasm.extend(section(7, &export_finish(dummy_count + 1)));
    let mut element = vec![0x01, 0x00, 0x41, 0x00, 0x0b];
    element.extend(leb128_u32(TABLE_LEN));
    element.extend(std::iter::repeat_n(0x00, TABLE_LEN as usize));
    wasm.extend(section(9, &element));
    let mut code = leb128_u32(n_code);
    code.extend(sized_body(&pad_body(fat_empty_locals(0))));
    let dummy = sized_body(&dummy_combo_body());
    for _ in 0..dummy_count {
        code.extend_from_slice(&dummy);
    }
    code.extend(sized_body(&pad_body(finish_counted_unroll_call_indirect(
        iters, unroll,
    ))));
    wasm.extend(section(10, &code));
    wasm
}

/// Kernel B + Create filler: empty `$fat` (func 0), `call_indirect` loop, unused combo dummies.
pub fn module_total_b(dummy_count: u32) -> Vec<u8> {
    let n_code = dummy_count + 2;
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.extend(section(
        1,
        &[0x02, 0x60, 0x00, 0x00, 0x60, 0x00, 0x01, 0x7f],
    ));
    let mut funcs = leb128_u32(n_code);
    funcs.push(0x00);
    funcs.extend(std::iter::repeat_n(0x00, dummy_count as usize));
    funcs.push(0x01);
    wasm.extend(section(3, &funcs));
    let mut table = vec![0x01, 0x70, 0x00];
    table.extend(leb128_u32(TABLE_LEN));
    wasm.extend(section(4, &table));
    let mut memory = vec![0x01, 0x00];
    memory.extend(leb128_u32(MEMORY_PAGES));
    wasm.extend(section(5, &memory));
    wasm.extend(section(7, &export_finish(dummy_count + 1)));
    let mut element = vec![0x01, 0x00, 0x41, 0x00, 0x0b];
    element.extend(leb128_u32(TABLE_LEN));
    element.extend(std::iter::repeat_n(0x00, TABLE_LEN as usize));
    wasm.extend(section(9, &element));
    let mut code = leb128_u32(n_code);
    code.extend(sized_body(&pad_body(fat_empty_locals(0))));
    let dummy = sized_body(&dummy_combo_body());
    for _ in 0..dummy_count {
        code.extend_from_slice(&dummy);
    }
    code.extend(sized_body(&finish_call_indirect()));
    wasm.extend(section(10, &code));
    wasm
}

pub fn write_case(rel_wasm: &str, wasm: &[u8], note: &str) {
    let out: PathBuf = [env!("CARGO_MANIFEST_DIR"), rel_wasm].iter().collect();
    fs::create_dir_all(out.parent().unwrap()).unwrap();
    fs::write(&out, wasm).unwrap();
    println!("wrote {}: {} bytes, {note}", out.display(), wasm.len());
}
