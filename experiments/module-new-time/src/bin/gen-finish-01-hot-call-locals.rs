//! Generates `finish_cases/01-hot-call-locals/case.wasm`.
//!
//! 8 MiB min memory, a 30_000-local dummy, and `finish` that calls it in a
//! loop until fuel runs out. See `sketch.wat`.

use std::{fs, path::PathBuf};

const LOCALS: u32 = 30_000;
const MEMORY_PAGES: u32 = 128;

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
    // no locals; loop { call 0; br 0 }; i32.const 0; end
    vec![
        0x00, 0x03, 0x40, 0x10, 0x00, 0x0c, 0x00, 0x0b, 0x41, 0x00, 0x0b,
    ]
}

fn module() -> Vec<u8> {
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();

    // type 0: () -> (), type 1: () -> i32
    wasm.extend(section(
        1,
        &[0x02, 0x60, 0x00, 0x00, 0x60, 0x00, 0x01, 0x7f],
    ));
    // func 0 = $fat (type 0), func 1 = $finish (type 1)
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
    code.extend(sized_body(&fat_body()));
    code.extend(sized_body(&finish_body()));
    wasm.extend(section(10, &code));
    wasm
}

fn main() {
    let wasm = module();
    let out: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "finish_cases/01-hot-call-locals/case.wasm",
    ]
    .iter()
    .collect();
    fs::create_dir_all(out.parent().unwrap()).unwrap();
    fs::write(&out, &wasm).unwrap();
    println!(
        "wrote {}: {} bytes, {LOCALS} locals, {MEMORY_PAGES} memory pages",
        out.display(),
        wasm.len()
    );
}
