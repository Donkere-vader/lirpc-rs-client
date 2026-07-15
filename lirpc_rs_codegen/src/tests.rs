use std::collections::HashMap;

use lirpc::{
    api_spec::{ApiSpec, LiRpcMethodSpec},
    codegen::CodeGen,
    translatable::Type,
    type_definition::{StructDefinition, StructFields, TypeDefinition},
};

use crate::RustCodeGen;

const EMPTY_CARGO_TOML: &str = r#"[package]
name = "my-app"
version = "0.1.0"
edition = "2024"

[dependencies]
lirpc_client = {}
"#;

const EMPTY_LIB_RS: &str = r#"TODO"#;

#[test]
fn test_empty_api_spec() {
    let spec = ApiSpec::new(
        "my-app".to_string(),
        "0.1.0".to_string(),
        HashMap::new(),
        HashMap::new(),
    )
    .unwrap();

    let mut package = RustCodeGen::generate_package(&spec);

    let cargo_toml = package.remove("Cargo.toml").unwrap();
    let lib_rs = package.remove("src/lib.rs").unwrap();

    assert!(package.is_empty());
    assert_eq!(cargo_toml, EMPTY_CARGO_TOML);
    assert_eq!(lib_rs, EMPTY_LIB_RS);
}

const GREETER_CARGO_TOML: &str = r#"[package]
name = "greeter"
version = "0.1.0"
edition = "2024"

[dependencies]
lirpc_client = {}
"#;

const GREETER_LIB_RS: &str = r#"TODO"#;

#[test]
fn test_greeter_api_spec() {
    let spec = ApiSpec::new(
        "greeter".to_string(),
        "0.1.0".to_string(),
        HashMap::from([(
            "greet".to_string(),
            LiRpcMethodSpec {
                messages: vec![Type::TypeRef("GreetingRequest".to_string())],
                returns: Type::TypeRef("GreetingResponse".to_string()),
            },
        )]),
        HashMap::from([
            (
                "GreetingRequest".to_string(),
                TypeDefinition::Struct(Box::new(StructDefinition {
                    ident: "GreetingRequest".to_string(),
                    fields: StructFields::Named(vec![("name".to_string(), Type::String)]),
                    generics: vec![],
                })),
            ),
            (
                "GreetingResponse".to_string(),
                TypeDefinition::Struct(Box::new(StructDefinition {
                    ident: "GreetingResponse".to_string(),
                    fields: StructFields::Named(vec![("msg".to_string(), Type::String)]),
                    generics: vec![],
                })),
            ),
        ]),
    )
    .unwrap();

    let mut package = RustCodeGen::generate_package(&spec);

    let cargo_toml = package.remove("Cargo.toml").unwrap();
    let lib_rs = package.remove("src/lib.rs").unwrap();

    assert!(package.is_empty());
    assert_eq!(cargo_toml, GREETER_CARGO_TOML);
    assert_eq!(lib_rs, GREETER_LIB_RS);
}
