//! Generates `create_cases/10-duplicate-types/case.wasm`.
//!
//! Case 08's shape with the opposite choice of types: one 3-byte type,
//! `() -> ()`, repeated to fill ~100 KB, plus `() -> i32` for the exported
//! `finish`. See `sketch.wat`.

use std::{fs, path::PathBuf};

const TARGET_SIZE: usize = 100_000;

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

fn module(duplicate_count: u32) -> Vec<u8> {
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();

    // Type 0 is `() -> i32` for `finish`; the rest are copies of `() -> ()`.
    let mut types = leb128_u32(duplicate_count + 1);
    types.extend_from_slice(&[0x60, 0x00, 0x01, 0x7f]);
    for _ in 0..duplicate_count {
        types.extend_from_slice(&[0x60, 0x00, 0x00]);
    }
    wasm.extend(section(1, &types));

    wasm.extend(section(3, &[0x01, 0x00]));

    let name = b"finish";
    let mut export = vec![0x01];
    export.extend(leb128_u32(name.len() as u32));
    export.extend_from_slice(name);
    export.extend_from_slice(&[0x00, 0x00]);
    wasm.extend(section(7, &export));

    // One body: no locals; i32.const 0; end.
    let body = [0x00, 0x41, 0x00, 0x0b];
    let mut code = leb128_u32(1);
    code.extend(leb128_u32(body.len() as u32));
    code.extend_from_slice(&body);
    wasm.extend(section(10, &code));
    wasm
}

fn main() {
    let (mut low, mut high) = (0u32, 100_000);
    while low < high {
        let mid = (low + high).div_ceil(2);
        if module(mid).len() <= TARGET_SIZE {
            low = mid;
        } else {
            high = mid - 1;
        }
    }

    let wasm = module(low);
    let out: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "create_cases/10-duplicate-types/case.wasm",
    ]
    .iter()
    .collect();
    fs::create_dir_all(out.parent().expect("case dir")).expect("create case dir");
    fs::write(&out, &wasm).expect("write case.wasm");
    println!(
        "wrote {}: {} bytes, {} types ({} copies of `() -> ()` + `() -> i32`)",
        out.display(),
        wasm.len(),
        low + 1,
        low
    );
}
