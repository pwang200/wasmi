//! Generates `finish_cases/00-baseline/case.wasm`.
//!
//! Smallest host-runnable module: exported `finish` returns 0. See `sketch.wat`.

use std::{fs, path::PathBuf};

fn leb128_u32(mut value: u32) -> Vec<u8> {
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

fn module() -> Vec<u8> {
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.extend(section(1, &[0x01, 0x60, 0x00, 0x01, 0x7f]));
    wasm.extend(section(3, &[0x01, 0x00]));
    let name = b"finish";
    let mut export = vec![0x01];
    export.extend(leb128_u32(name.len() as u32));
    export.extend_from_slice(name);
    export.extend_from_slice(&[0x00, 0x00]);
    wasm.extend(section(7, &export));
    wasm.extend(section(10, &[0x01, 0x04, 0x00, 0x41, 0x00, 0x0b]));
    wasm
}

fn main() {
    let wasm = module();
    let out: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "finish_cases/00-baseline/case.wasm",
    ]
    .iter()
    .collect();
    fs::create_dir_all(out.parent().unwrap()).unwrap();
    fs::write(&out, &wasm).unwrap();
    println!("wrote {}: {} bytes", out.display(), wasm.len());
}
