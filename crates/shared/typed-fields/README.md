# docnav-typed-fields

Canonical parameter declarations shared by Docnav's root Rust workspace. This package owns
`FieldDef` / `FieldDefSet`, processing locators, value kinds, constraints, static defaults,
`MergeStrategy`, validation facts, typed values, and all-or-nothing materialization.

The main entry points are:

- `FieldDef::builder` and `FieldDefSet::builder` for canonical declarations.
- `FieldValidation` and `MergeStrategy` for field-level validation and merge metadata.
- `FieldValueMap` and `FieldDefs` for typed materialization.

Source priority, framework parsing, and application-specific diagnostics remain outside this
package. Consumers using the builder API can import these types through the primary
`cli-config-resolution` facade. Consumers using `#[derive(FieldDefs)]` depend on this package
directly. See the [`resolution_flow`](../cli-config-resolution-clap/examples/resolution_flow.rs)
example in the root workspace for the complete declaration-to-materialization path.
