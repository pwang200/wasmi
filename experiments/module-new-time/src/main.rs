mod config;

use crate::config::{escrow_engine_config, escrow_store_limits};
use std::{env, fs, time::Duration, time::Instant};
use wasmi::{Engine, Linker, Module, Store};

/// Finish-path fuel budget (XRPL-like). Out-of-fuel on `finish` is a valid outcome.
const FUEL_LIMIT: u64 = 1_000_000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .ok_or("usage: module-new-time <file.wasm>")?;
    let wasm = fs::read(&path)?;
    if wasm.get(..4) != Some(&[0x00, 0x61, 0x73, 0x6d]) {
        return Err(format!("{path} is not a Wasm binary; pass .wasm, not .wat").into());
    }

    let t = Instant::now();
    let engine = Engine::new(&escrow_engine_config());
    let engine_time = t.elapsed();

    let t = Instant::now();
    let module = match Module::new(&engine, &wasm) {
        Ok(module) => module,
        Err(err) => {
            print_report(
                wasm.len(),
                engine_time,
                t.elapsed(),
                None,
                None,
                None,
                None,
                None,
                Some(format!("Module::new: {err}")),
            );
            return Ok(());
        }
    };
    let create_time = t.elapsed();

    let t = Instant::now();
    let mut store = Store::new(&engine, escrow_store_limits());
    store.limiter(|limits| limits);
    store.set_fuel(FUEL_LIMIT)?;
    let fuel0 = store.get_fuel()?;
    let instantiate = Linker::new(&engine).instantiate_and_start(&mut store, &module);
    let instantiate_time = t.elapsed();

    let mut instantiate_fuel = None;
    let mut finish_time = None;
    let mut finish_fuel = None;
    let mut finish_result = None;
    let mut error = None;

    match instantiate {
        Ok(instance) => {
            let fuel1 = store.get_fuel()?;
            instantiate_fuel = Some(fuel0 - fuel1);
            match instance.get_typed_func::<(), i32>(&store, "finish") {
                Ok(func) => {
                    let t = Instant::now();
                    match func.call(&mut store, ()) {
                        Ok(result) => {
                            let fuel2 = store.get_fuel()?;
                            finish_time = Some(t.elapsed());
                            finish_fuel = Some(fuel1 - fuel2);
                            finish_result = Some(result);
                        }
                        Err(err) => {
                            finish_time = Some(t.elapsed());
                            if let Ok(fuel2) = store.get_fuel() {
                                finish_fuel = Some(fuel1.saturating_sub(fuel2));
                            }
                            error = Some(format!("finish: {err}"));
                        }
                    }
                }
                Err(err) => error = Some(format!("finish: {err}")),
            }
        }
        Err(err) => error = Some(format!("instantiate: {err}")),
    }

    print_report(
        wasm.len(),
        engine_time,
        create_time,
        Some(instantiate_time),
        instantiate_fuel,
        finish_time,
        finish_fuel,
        finish_result,
        error,
    );
    Ok(())
}

fn print_report(
    bytes: usize,
    engine_time: Duration,
    create_time: Duration,
    instantiate_time: Option<Duration>,
    instantiate_fuel: Option<u64>,
    finish_time: Option<Duration>,
    finish_fuel: Option<u64>,
    finish_result: Option<i32>,
    error: Option<String>,
) {
    println!("{bytes} bytes");
    println!("engine: {engine_time:?}");
    println!("Module::new: {create_time:?}");
    match (instantiate_time, instantiate_fuel) {
        (Some(t), Some(fuel)) => println!("instantiate: {t:?}  fuel {fuel}"),
        (Some(t), None) => println!("instantiate: {t:?}"),
        (None, _) => {}
    }
    if let Some(t) = finish_time {
        match (finish_result, finish_fuel) {
            (Some(result), Some(fuel)) => println!("finish -> {result}  {t:?}  fuel {fuel}"),
            (_, Some(fuel)) => println!("finish: {t:?}  fuel {fuel}"),
            _ => println!("finish: {t:?}"),
        }
    }
    let finish_shaped = [instantiate_time, finish_time]
        .into_iter()
        .flatten()
        .fold(Duration::ZERO, |a, b| a + b);
    if instantiate_time.is_some() || finish_time.is_some() {
        println!("instantiate + finish: {finish_shaped:?}");
    }
    let total = engine_time
        + create_time
        + instantiate_time.unwrap_or(Duration::ZERO)
        + finish_time.unwrap_or(Duration::ZERO);
    println!("total: {total:?}");
    if let Some(err) = error {
        println!("not runnable: {err}");
    }
}
