//! Generates `cases/05-fat-data/case.wasm`. One active data blob + tiny `finish`.

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

fn module(data_len: u32) -> Vec<u8> {
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.extend(section(1, &[0x01, 0x60, 0x00, 0x01, 0x7f]));
    wasm.extend(section(3, &[0x01, 0x00]));
    wasm.extend(section(5, &[0x01, 0x00, 0x02])); // 1 memory, min 2 pages

    let name = b"finish";
    let mut export = vec![0x01];
    export.extend(leb128_u32(name.len() as u32));
    export.extend_from_slice(name);
    export.extend_from_slice(&[0x00, 0x00]);
    wasm.extend(section(7, &export));

    wasm.extend(section(10, &[0x01, 0x04, 0x00, 0x41, 0x00, 0x0b]));

    let mut data = vec![0x01, 0x00, 0x41, 0x00, 0x0b];
    data.extend(leb128_u32(data_len));
    data.extend(std::iter::repeat_n(0x00, data_len as usize));
    wasm.extend(section(11, &data));
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
    let out: PathBuf = [env!("CARGO_MANIFEST_DIR"), "cases/05-fat-data/case.wasm"]
        .iter()
        .collect();
    fs::create_dir_all(out.parent().unwrap()).unwrap();
    fs::write(&out, &wasm).unwrap();
    println!("wrote {}: {} bytes, {low} data bytes", out.display(), wasm.len());
}
