#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use lirpc::{api_spec::ApiSpec, codegen::CodeGen};

pub struct RustCodeGen;

impl CodeGen for RustCodeGen {
    fn generate_package(spec: &ApiSpec) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("Cargo.toml".to_string(), Self::generate_cargo_toml(spec)),
            ("src/lib.rs".to_string(), Self::generate_rust_code(spec)),
        ])
    }
}

impl RustCodeGen {
    fn generate_cargo_toml(spec: &ApiSpec) -> String {
        format!(
            "[package]\nname = \"{}\"\nversion = \"{}\"\nedition = \"2024\"\n\n[dependencies]\nlirpc_client = {{}}\n",
            spec.name, spec.version,
        )
    }

    fn generate_rust_code(_spec: &ApiSpec) -> String {
        "TODO".to_string()
    }
}
