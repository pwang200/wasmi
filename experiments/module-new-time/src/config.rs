use wasmi::{CompilationMode, Config, EnforcedLimits, StoreLimits, StoreLimitsBuilder};

/// Engine config matching the XRPL Smart Escrow screening path.
///
/// `memory64` is off because this crate depends on `wasmi` without the
/// `memory64` cargo feature. SIMD is off the same way (feature not enabled).
pub fn escrow_engine_config() -> Config {
    let mut config = Config::default();
    config.consume_fuel(true);
    config.ignore_custom_sections(true);
    config.allow_start_fn(false);
    config.compilation_mode(CompilationMode::LazyTranslation);
    config.floats(false);
    config.wasm_mutable_global(false);
    config.wasm_multi_value(false);
    config.wasm_sign_extension(false);
    config.wasm_saturating_float_to_int(false);
    config.wasm_bulk_memory(false);
    config.wasm_reference_types(false);
    config.wasm_tail_call(false);
    config.wasm_extended_const(false);
    config.wasm_multi_memory(false);
    config.wasm_custom_page_sizes(false);
    config.wasm_wide_arithmetic(false);
    config.enforced_limits(EnforcedLimits::strict());
    config
}

/// Store limits used when instantiating (Finish / run). Not applied in `Module::new`.
pub fn escrow_store_limits() -> StoreLimits {
    StoreLimitsBuilder::new()
        .memory_size(8 * 1024 * 1024)
        .table_elements(1024)
        .instances(1)
        .memories(1)
        .tables(1)
        .build()
}
