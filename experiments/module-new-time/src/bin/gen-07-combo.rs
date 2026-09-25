//! Generates `create_cases/07-combo/case.wasm`.
//!
//! Max functions at the 40-byte floor. Each dummy declares 50_000 i32 locals
//! and spends the rest of the body on a small `br_table`. See `sketch.wat`.

use std::{fs, path::PathBuf};

const TARGET_SIZE: usize = 100_000;
const BODY_LEN: usize = 39;
const LOCALS: u32 = 50_000;

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

fn dummy_body() -> Vec<u8> {
    // 1 local group of 50_000 i32, then block + i32.const 0 + br_table + ends.
    let mut locals = vec![0x01];
    locals.extend(leb128_u32(LOCALS));
    locals.push(0x7f);
    let mut tail = vec![0x02, 0x40, 0x41, 0x00, 0x0e];
    let n = (BODY_LEN - locals.len() - tail.len() - 4) as u32; // leb(n), default, 2×end
    debug_assert!(n < 128);
    tail.extend(leb128_u32(n));
    tail.extend(std::iter::repeat_n(0x00, n as usize));
    tail.extend_from_slice(&[0x00, 0x0b, 0x0b]);
    let mut body = locals;
    body.extend(tail);
    debug_assert_eq!(body.len(), BODY_LEN, "{}", body.len());
    body
}

fn finish_body() -> Vec<u8> {
    let mut body = vec![0x00];
    body.extend(std::iter::repeat_n(0x01, BODY_LEN - 4));
    body.extend_from_slice(&[0x41, 0x00, 0x0b]);
    debug_assert_eq!(body.len(), BODY_LEN);
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
    code.extend(sized_body(&finish_body()));
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
    let out: PathBuf = [env!("CARGO_MANIFEST_DIR"), "create_cases/07-combo/case.wasm"]
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
