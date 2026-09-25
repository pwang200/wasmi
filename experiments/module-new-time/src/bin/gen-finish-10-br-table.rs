//! Generates `finish_cases/10-br-table/case.wasm`.
//!
//! Exported `finish` is a fat `br_table` plus `i32.const 0`. See `sketch.wat`.

use std::{fs, path::PathBuf};

const TARGET_SIZE: usize = 100_000;

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

fn sized_body(body: &[u8]) -> Vec<u8> {
    let mut out = leb128_u32(body.len() as u32);
    out.extend_from_slice(body);
    out
}

fn finish_body(target_count: u32) -> Vec<u8> {
    let mut body = vec![
        0x00, // no locals
        0x02, 0x40, // block
        0x41, 0x00, // i32.const 0
        0x0e, // br_table
    ];
    body.extend(leb128_u32(target_count));
    body.extend(std::iter::repeat_n(0x00, target_count as usize));
    body.push(0x00); // default
    body.push(0x0b); // end block
    body.extend_from_slice(&[0x41, 0x00, 0x0b]);
    body
}

fn module(target_count: u32) -> Vec<u8> {
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
    code.extend(sized_body(&finish_body(target_count)));
    wasm.extend(section(10, &code));
    wasm
}

fn main() {
    let (mut low, mut high) = (0u32, TARGET_SIZE as u32);
    while low < high {
        let mid = (low + high).div_ceil(2);
        if module(mid).len() <= TARGET_SIZE {
            low = mid;
        } else {
            high = mid - 1;
        }
    }
    let wasm = module(low);
    let out: PathBuf = [env!("CARGO_MANIFEST_DIR"), "finish_cases/10-br-table/case.wasm"]
        .iter()
        .collect();
    fs::create_dir_all(out.parent().unwrap()).unwrap();
    fs::write(&out, &wasm).unwrap();
    println!(
        "wrote {}: {} bytes, {low} br_table targets",
        out.display(),
        wasm.len()
    );
}
