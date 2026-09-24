mod config;

use crate::config::escrow_engine_config;
use std::{env, fs, time::Instant};
use wasmi::{Engine, Linker, Module, Store};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).ok_or("usage: module-new-time <file.wasm>")?;
    let wasm = fs::read(&path)?;
    if wasm.get(..4) != Some(&[0x00, 0x61, 0x73, 0x6d]) {
        return Err(format!("{path} is not a Wasm binary; pass .wasm, not .wat").into());
    }

    let engine = Engine::new(&escrow_engine_config());
    let t = Instant::now();
    let module = Module::new(&engine, &wasm)?;
    println!("{} bytes", wasm.len());
    println!(
        "Module::new: {:?}  (no fuel: metering starts with Store)",
        t.elapsed()
    );

    let mut store = Store::new(&engine, ());
    store.set_fuel(u64::MAX)?;
    let fuel0 = store.get_fuel()?;

    let t = Instant::now();
    let instance = Linker::new(&engine).instantiate_and_start(&mut store, &module)?;
    let fuel1 = store.get_fuel()?;
    println!(
        "instantiate: {:?}  fuel {}",
        t.elapsed(),
        fuel0 - fuel1
    );

    let func = instance.get_typed_func::<(), i32>(&store, "escrow_finish")?;
    let t = Instant::now();
    let result = func.call(&mut store, ())?;
    let fuel2 = store.get_fuel()?;
    println!(
        "escrow_finish -> {result}  {:?}  fuel {}  (lazy translate of this func + exec)",
        t.elapsed(),
        fuel1 - fuel2
    );
    println!("fuel remaining: {fuel2}");
    Ok(())
}
