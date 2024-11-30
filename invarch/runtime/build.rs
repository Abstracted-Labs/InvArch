use substrate_wasm_builder::WasmBuilder;

#[cfg(all(not(feature = "metadata-hash"), feature = "std"))]
fn main() {
    WasmBuilder::new()
        .with_current_project()
        .export_heap_base()
        .import_memory()
        .build()
}

#[cfg(all(feature = "metadata-hash", feature = "std"))]
fn main() {
    WasmBuilder::new()
        .with_current_project()
        .export_heap_base()
        .import_memory()
        .enable_metadata_hash("VARCH", 12)
        .build()
}

/// The wasm builder is deactivated when compiling
/// this crate for wasm to speed up the compilation.
#[cfg(not(feature = "std"))]
fn main() {}
