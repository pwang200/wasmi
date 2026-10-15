//! Generates `create_cases/11-many-imports/case.wasm`.
//!
//! The XRPL host function `host_lib::float_pow` imported again and again until
//! ~100 KB, plus an exported `finish`. `float_pow` is the last entry in the
//! XRPL host-function list, so XRPL's import screening walks the whole list for
//! every copy. See `sketch.wat`.
//!
//! This harness has no host functions, so instantiate fails here; only
//! `Module::new` is meaningful. The XRPL-side check is timed outside this crate.

use std::{fs, path::PathBuf};

const TARGET_SIZE: usize = 100_000;
const HOST_MODULE: &[u8] = b"host_lib";
const HOST_FUNCTION: &[u8] = b"float_pow";

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

fn module(import_count: u32) -> Vec<u8> {
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();

    // Type 0: float_pow's lowered signature, six i32 params and an i32 result
    // (x ptr/len, n, out ptr/len, mode). Type 1: `() -> i32` for `finish`.
    let mut types = vec![0x02, 0x60, 0x06];
    types.extend(std::iter::repeat_n(0x7f, 6));
    types.extend_from_slice(&[0x01, 0x7f]);
    types.extend_from_slice(&[0x60, 0x00, 0x01, 0x7f]);
    wasm.extend(section(1, &types));

    let mut entry = leb128_u32(HOST_MODULE.len() as u32);
    entry.extend_from_slice(HOST_MODULE);
    entry.extend(leb128_u32(HOST_FUNCTION.len() as u32));
    entry.extend_from_slice(HOST_FUNCTION);
    entry.extend_from_slice(&[0x00, 0x00]); // a function of type 0
    let mut imports = leb128_u32(import_count);
    for _ in 0..import_count {
        imports.extend_from_slice(&entry);
    }
    wasm.extend(section(2, &imports));

    wasm.extend(section(3, &[0x01, 0x01]));

    // `finish` follows the imports in the function index space.
    let name = b"finish";
    let mut export = vec![0x01];
    export.extend(leb128_u32(name.len() as u32));
    export.extend_from_slice(name);
    export.push(0x00);
    export.extend(leb128_u32(import_count));
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
    // strict() caps functions at 10k, imported ones included; 100 KB binds first.
    let (mut low, mut high) = (0u32, 9_999);
    while low < high {
        let mid = (low + high).div_ceil(2);
        if module(mid).len() <= TARGET_SIZE {
            low = mid;
        } else {
            high = mid - 1;
        }
    }

    let wasm = module(low);
    let out: PathBuf = [env!("CARGO_MANIFEST_DIR"), "create_cases/11-many-imports/case.wasm"]
        .iter()
        .collect();
    fs::create_dir_all(out.parent().expect("case dir")).expect("create case dir");
    fs::write(&out, &wasm).expect("write case.wasm");
    println!(
        "wrote {}: {} bytes, {} imports of host_lib::float_pow + finish",
        out.display(),
        wasm.len(),
        low
    );
}
