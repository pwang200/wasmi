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

fn locals_header() -> Vec<u8> {
    let mut body = vec![0x01];
    body.extend(leb128_u32(LOCALS));
    body.push(0x7f);
    body
}

fn finish_body() -> Vec<u8> {
    vec![
        0x00, 0x03, 0x40, 0x10, 0x00, 0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b,
    ]
}

pub fn fat_i64_div_u() -> Vec<u8> {
    let mut body = locals_header();
    body.extend_from_slice(&[0x42, 0x01, 0x42, 0x01, 0x80, 0x1a, 0x0b]);
    body
}

pub fn fat_load_same() -> Vec<u8> {
    let mut body = locals_header();
    // i32.const 0; i32.load align=2 offset=0; drop; end
    body.extend_from_slice(&[0x41, 0x00, 0x28, 0x02, 0x00, 0x1a, 0x0b]);
    body
}

/// Load `*cursor`, then `cursor = (cursor + stride) & ADDR_MASK` stored at mem[0].
pub fn fat_load_stride(stride: u32) -> Vec<u8> {
    let mut body = locals_header();
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

pub fn module(fat: &[u8]) -> Vec<u8> {
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
