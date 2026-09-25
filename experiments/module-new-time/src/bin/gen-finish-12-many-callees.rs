//! Generates `finish_cases/12-many-callees/case.wasm`.
//!
//! ~40-byte dummies; `finish` calls each once and returns 0. See `sketch.wat`.

use std::{fs, path::PathBuf};

const TARGET_SIZE: usize = 100_000;
const BODY_LEN: usize = 39;

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

fn dummy_body() -> Vec<u8> {
    let mut body = vec![0x00];
    body.extend(std::iter::repeat_n(0x01, BODY_LEN - 2));
    body.push(0x0b);
    debug_assert_eq!(body.len(), BODY_LEN);
    body
}

fn finish_body(dummy_count: u32) -> Vec<u8> {
    let mut body = vec![0x00];
    for i in 0..dummy_count {
        body.push(0x10);
        body.extend(leb128_u32(i));
    }
    body.extend_from_slice(&[0x41, 0x00, 0x0b]);
    body
}

fn module(dummy_count: u32) -> Vec<u8> {
    let func_count = dummy_count + 1;
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.extend(section(
        1,
        &[0x02, 0x60, 0x00, 0x00, 0x60, 0x00, 0x01, 0x7f],
    ));
    let mut funcs = leb128_u32(func_count);
    funcs.extend(std::iter::repeat_n(0x00, dummy_count as usize));
    funcs.push(0x01);
    wasm.extend(section(3, &funcs));
    let name = b"finish";
    let mut export = vec![0x01];
    export.extend(leb128_u32(name.len() as u32));
    export.extend_from_slice(name);
    export.extend_from_slice(&[0x00]);
    export.extend(leb128_u32(dummy_count));
    wasm.extend(section(7, &export));
    let mut code = leb128_u32(func_count);
    let dummy = sized_body(&dummy_body());
    for _ in 0..dummy_count {
        code.extend_from_slice(&dummy);
    }
    code.extend(sized_body(&finish_body(dummy_count)));
    wasm.extend(section(10, &code));
    wasm
}

fn main() {
    let (mut low, mut high) = (1u32, 10_000);
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
        "finish_cases/12-many-callees/case.wasm",
    ]
    .iter()
    .collect();
    fs::create_dir_all(out.parent().unwrap()).unwrap();
    fs::write(&out, &wasm).unwrap();
    println!(
        "wrote {}: {} bytes, {} functions ({} dummy + finish)",
        out.display(),
        wasm.len(),
        low + 1,
        low
    );
}
