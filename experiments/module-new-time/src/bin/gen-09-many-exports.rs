//! Generates `create_cases/09-many-exports/case.wasm`.
//!
//! One function, `finish`, exported under its own name plus as many short
//! unique names as fit in ~100 KB, shortest names first. No function bodies to
//! speak of, so the `strict()` 40-byte average never applies. See `sketch.wat`.

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

/// Every name over the 128 ASCII bytes, shortest first: all 1-byte names, then
/// all 2-byte names, and so on. ASCII is valid UTF-8, which export names must be.
///
/// "finish" is 6 bytes long and this never gets past 3, so it never collides.
fn short_names() -> impl Iterator<Item = Vec<u8>> {
    (1u32..).flat_map(|len| {
        (0u64..128u64.pow(len)).map(move |mut index| {
            let mut name = Vec::with_capacity(len as usize);
            for _ in 0..len {
                name.push((index % 128) as u8);
                index /= 128;
            }
            name
        })
    })
}

fn export_entry(name: &[u8]) -> Vec<u8> {
    let mut out = leb128_u32(name.len() as u32);
    out.extend_from_slice(name);
    // Function 0.
    out.extend_from_slice(&[0x00, 0x00]);
    out
}

fn module(alias_count: u32) -> Vec<u8> {
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();

    // One type: () -> i32.
    wasm.extend(section(1, &[0x01, 0x60, 0x00, 0x01, 0x7f]));
    wasm.extend(section(3, &[0x01, 0x00]));

    let mut export = leb128_u32(alias_count + 1);
    export.extend(export_entry(b"finish"));
    for name in short_names().take(alias_count as usize) {
        export.extend(export_entry(&name));
    }
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
    let out: PathBuf = [env!("CARGO_MANIFEST_DIR"), "create_cases/09-many-exports/case.wasm"]
        .iter()
        .collect();
    fs::create_dir_all(out.parent().expect("case dir")).expect("create case dir");
    fs::write(&out, &wasm).expect("write case.wasm");
    println!(
        "wrote {}: {} bytes, {} exports ({} short names + finish)",
        out.display(),
        wasm.len(),
        low + 1,
        low
    );
}
