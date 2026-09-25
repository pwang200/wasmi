mod config;

use crate::config::{escrow_engine_config, escrow_store_limits};
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

    let mut store = Store::new(&engine, escrow_store_limits());
    store.limiter(|limits| limits);
    store.set_fuel(u64::MAX)?;
    let fuel0 = store.get_fuel()?;

    let t = Instant::now();
    match Linker::new(&engine).instantiate_and_start(&mut store, &module) {
        Ok(instance) => {
            let fuel1 = store.get_fuel()?;
            println!(
                "instantiate: {:?}  fuel {}",
                t.elapsed(),
                fuel0 - fuel1
            );
            match instance.get_typed_func::<(), i32>(&store, "finish") {
                Ok(func) => {
                    let t = Instant::now();
                    match func.call(&mut store, ()) {
                        Ok(result) => {
                            let fuel2 = store.get_fuel()?;
                            println!(
                                "finish -> {result}  {:?}  fuel {}  (lazy translate of this func + exec)",
                                t.elapsed(),
                                fuel1 - fuel2
                            );
                            println!("fuel remaining: {fuel2}");
                        }
                        Err(err) => println!("not runnable: finish: {err}"),
                    }
                }
                Err(err) => println!("not runnable: finish: {err}"),
            }
        }
        Err(err) => println!("not runnable: instantiate: {err}"),
    }
    Ok(())
}
