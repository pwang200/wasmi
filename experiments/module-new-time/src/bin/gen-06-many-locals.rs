//! Generates `cases/06-many-locals/case.wasm`.
//! 50_000 i32 locals in an unused function, nops to fill ~100 KB, tiny `finish`.

use std::{fs, path::PathBuf};

const TARGET_SIZE: usize = 100_000;
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

fn fat_body(nop_count: u32) -> Vec<u8> {
    let mut body = vec![0x01]; // 1 local group
    body.extend(leb128_u32(LOCALS));
    body.push(0x7f); // i32
    body.extend(std::iter::repeat_n(0x01, nop_count as usize));
    body.push(0x0b);
    body
}

fn module(nop_count: u32) -> Vec<u8> {
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.extend(section(
        1,
        &[0x02, 0x60, 0x00, 0x00, 0x60, 0x00, 0x01, 0x7f],
    ));
    wasm.extend(section(3, &[0x02, 0x00, 0x01]));

    let name = b"finish";
    let mut export = vec![0x01];
    export.extend(leb128_u32(name.len() as u32));
    export.extend_from_slice(name);
    export.extend_from_slice(&[0x00, 0x01]);
    wasm.extend(section(7, &export));

    let mut code = vec![0x02];
    code.extend(sized_body(&fat_body(nop_count)));
    code.extend(sized_body(&[0x00, 0x41, 0x00, 0x0b]));
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
    let out: PathBuf = [env!("CARGO_MANIFEST_DIR"), "cases/06-many-locals/case.wasm"]
        .iter()
        .collect();
    fs::create_dir_all(out.parent().unwrap()).unwrap();
    fs::write(&out, &wasm).unwrap();
    println!(
        "wrote {}: {} bytes, {LOCALS} locals, {low} nops",
        out.display(),
        wasm.len()
    );
}
