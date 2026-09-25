//! Generates `finish_cases/02-call-indirect/case.wasm`.
//!
//! Case 1's loop, but `call_indirect` through a 1024-slot table filled with
//! `$fat`. See `sketch.wat`.

use std::{fs, path::PathBuf};

const LOCALS: u32 = 30_000;
const MEMORY_PAGES: u32 = 128;
const TABLE_LEN: u32 = 1024;

fn leb128_u32(value: u32) -> Vec<u8> {
    let mut value = value;
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

fn fat_body() -> Vec<u8> {
    let mut body = vec![0x01];
    body.extend(leb128_u32(LOCALS));
    body.push(0x7f);
    body.push(0x0b);
    body
}

fn finish_body() -> Vec<u8> {
    // no locals; loop { i32.const 0; call_indirect 0 0; br 0 }; i32.const 0; end
    vec![
        0x00, 0x03, 0x40, 0x41, 0x00, 0x11, 0x00, 0x00, 0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b,
    ]
}

fn module() -> Vec<u8> {
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

    let name = b"finish";
    let mut export = vec![0x01];
    export.extend(leb128_u32(name.len() as u32));
    export.extend_from_slice(name);
    export.extend_from_slice(&[0x00, 0x01]);
    wasm.extend(section(7, &export));

    // Active MVP element segment: 1024 copies of function 0 at offset 0.
    let mut element = vec![0x01, 0x00, 0x41, 0x00, 0x0b];
    element.extend(leb128_u32(TABLE_LEN));
    element.extend(std::iter::repeat_n(0x00, TABLE_LEN as usize));
    wasm.extend(section(9, &element));

    let mut code = vec![0x02];
    code.extend(sized_body(&fat_body()));
    code.extend(sized_body(&finish_body()));
    wasm.extend(section(10, &code));
    wasm
}

fn main() {
    let wasm = module();
    let out: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "finish_cases/02-call-indirect/case.wasm",
    ]
    .iter()
    .collect();
    fs::create_dir_all(out.parent().unwrap()).unwrap();
    fs::write(&out, &wasm).unwrap();
    println!(
        "wrote {}: {} bytes, {LOCALS} locals, table {TABLE_LEN}, {MEMORY_PAGES} memory pages",
        out.display(),
        wasm.len()
    );
}
