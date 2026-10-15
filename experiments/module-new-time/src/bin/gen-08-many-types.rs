//! Generates `create_cases/08-many-types/case.wasm`.
//!
//! A type section packed with distinct function types, shortest encodings
//! first, plus one exported `finish`. No function bodies to speak of, so the
//! `strict()` 40-byte average never applies. See `sketch.wat`.

use std::{fs, path::PathBuf};

const TARGET_SIZE: usize = 100_000;
const I32: u8 = 0x7f;
const I64: u8 = 0x7e;

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

/// One encoded function type: `0x60`, params, results (0 or 1, multi-value is off).
fn func_type(params: &[u8], result: Option<u8>) -> Vec<u8> {
    let mut out = vec![0x60];
    out.extend(leb128_u32(params.len() as u32));
    out.extend_from_slice(params);
    match result {
        None => out.push(0x00),
        Some(ty) => out.extend_from_slice(&[0x01, ty]),
    }
    out
}

/// Every distinct i32/i64 function type, in order of encoded size.
///
/// `() -> i32` (the `finish` type) comes second, so it is in every prefix the
/// size search picks.
fn distinct_types() -> impl Iterator<Item = Vec<u8>> {
    // An encoding of n params is 3 + n bytes with no result, 4 + n with one, so
    // walking encoded size and emitting both shapes that reach it is shortest-first.
    (3usize..).flat_map(|size| {
        let no_result = size.checked_sub(3).map(|n| (n, None));
        let one_result = size.checked_sub(4).map(|n| (n, Some(())));
        one_result
            .into_iter()
            .chain(no_result)
            .filter(|(n, _)| *n <= 32)
            .flat_map(|(n, result)| {
                (0u64..1 << n).flat_map(move |bits| {
                    let params: Vec<u8> = (0..n)
                        .map(|i| if bits >> i & 1 == 0 { I32 } else { I64 })
                        .collect();
                    let results: Vec<Option<u8>> = match result {
                        None => vec![None],
                        Some(()) => vec![Some(I32), Some(I64)],
                    };
                    results
                        .into_iter()
                        .map(move |result| func_type(&params, result))
                })
            })
    })
}

fn module(type_count: u32) -> Vec<u8> {
    let types: Vec<Vec<u8>> = distinct_types().take(type_count as usize).collect();
    let finish_type = func_type(&[], Some(I32));
    let finish_index = types
        .iter()
        .position(|ty| *ty == finish_type)
        .expect("`() -> i32` is in every prefix of at least two types") as u32;

    let mut wasm = b"\0asm\x01\0\0\0".to_vec();

    let mut type_payload = leb128_u32(type_count);
    for ty in &types {
        type_payload.extend_from_slice(ty);
    }
    wasm.extend(section(1, &type_payload));

    let mut funcs = leb128_u32(1);
    funcs.extend(leb128_u32(finish_index));
    wasm.extend(section(3, &funcs));

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
    let (mut low, mut high) = (2u32, 100_000);
    while low < high {
        let mid = (low + high).div_ceil(2);
        if module(mid).len() <= TARGET_SIZE {
            low = mid;
        } else {
            high = mid - 1;
        }
    }

    let wasm = module(low);
    let out: PathBuf = [env!("CARGO_MANIFEST_DIR"), "create_cases/08-many-types/case.wasm"]
        .iter()
        .collect();
    fs::create_dir_all(out.parent().expect("case dir")).expect("create case dir");
    fs::write(&out, &wasm).expect("write case.wasm");
    println!(
        "wrote {}: {} bytes, {} distinct function types",
        out.display(),
        wasm.len(),
        low
    );
}
