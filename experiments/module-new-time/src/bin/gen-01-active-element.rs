//! Generates `cases/01-active-element/case.wasm`.
//!
//! One function, one unexported funcref table, and one active element segment
//! filled with as many single-byte function-index entries as fit under the
//! target size. See `sketch.wat` for why this shape is costly to load.

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

fn module(element_count: u32) -> Vec<u8> {
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();

    // type 0: () -> i32
    wasm.extend(section(1, &[0x01, 0x60, 0x00, 0x01, 0x7f]));

    // One function of type 0.
    wasm.extend(section(3, &[0x01, 0x00]));

    // One unexported MVP funcref table, large enough for instantiation.
    let mut table = vec![0x01, 0x70, 0x00];
    table.extend(leb128_u32(element_count));
    wasm.extend(section(4, &table));

    // Export function 0 as escrow_finish.
    let name = b"escrow_finish";
    let mut export = vec![0x01];
    export.extend(leb128_u32(name.len() as u32));
    export.extend_from_slice(name);
    export.extend_from_slice(&[0x00, 0x00]);
    wasm.extend(section(7, &export));

    // One active MVP element segment at table offset 0. Every packed function
    // index is function 0 and therefore encodes to a single byte.
    let mut element = vec![0x01, 0x00, 0x41, 0x00, 0x0b];
    element.extend(leb128_u32(element_count));
    element.extend(std::iter::repeat_n(0x00, element_count as usize));
    wasm.extend(section(9, &element));

    // escrow_finish: no locals; i32.const 0; end.
    wasm.extend(section(10, &[0x01, 0x04, 0x00, 0x41, 0x00, 0x0b]));
    wasm
}

fn main() {
    // Module size grows monotonically with the entry count, so binary search
    // for the largest count that still fits under the target.
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
    let out: PathBuf = [env!("CARGO_MANIFEST_DIR"), "cases/01-active-element/case.wasm"]
        .iter()
        .collect();
    fs::write(&out, &wasm).expect("write case.wasm");
    println!(
        "wrote {}: {} bytes, {low} element entries",
        out.display(),
        wasm.len()
    );
}
