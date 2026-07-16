# cli-config-resolution

Framework-independent source extraction and resolution for canonical Rust CLI/config parameters.
`docnav-typed-fields::FieldDef` and `FieldDefSet` remain the single parameter model; this crate
re-exports them as well as the stateless `Parameter` and `ParameterSet` aliases.

The companion packages live beside this crate under `crates/shared` and participate in Docnav's
root Rust workspace.

## Usage

```rust
use cli_config_resolution::{
    extract_env, ExpectedFieldShape, FieldValidation, Parameter, ParameterSet, ProcessStrategy,
    ProcessingId, Resolver, SourceId, TypedValue,
};

let parameters = ParameterSet::builder()
    .field(
        Parameter::builder("limit")
            .process("env", ProcessStrategy::env_var("APP_LIMIT"))
            .validation(FieldValidation::int())
            .default_static(20),
        ExpectedFieldShape::required(),
    )
    .build()
    .expect("parameter set");

let env = extract_env(
    &parameters,
    &ProcessingId::from("env"),
    SourceId::new("environment").expect("source id"),
    30,
    [("APP_LIMIT", "42")],
)
.expect("environment source");
let result = Resolver::resolve(&parameters, &[env]).expect("valid resolver input");
let values = result.materialize().expect("valid configuration");

assert_eq!(
    values[&cli_config_resolution::FieldIdentity::new("limit").expect("identity")],
    TypedValue::Integer(42),
);
```

`Source` owns its id, kind, priority, and candidates. A missing CLI/env/config value simply does
not create a candidate. Extractor decode failures retain raw input, locator, and reason. Resolution
uses higher numeric priority, then later source registration for ties; `Append` and `MapMerge`
apply candidates from low to high precedence. Static field defaults are automatic fallbacks, while
runtime defaults can be supplied as an explicit `SourceKind::Default` source.

## Workspace validation

Run the focused package tests from the repository root:

```console
cargo test --locked -p cli-config-resolution
```
