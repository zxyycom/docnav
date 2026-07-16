# cli-config-resolution-clap

Thin `clap` integration for canonical `cli-config-resolution` fields. This companion consumes one
`FieldDefSet`, registers its declared CLI flags, and maps explicit matches directly into a core
`Source`.

## Entry points

- `augment_command` adds arguments for the selected canonical CLI processing profile.
- `extract_cli` converts explicitly supplied matches into a normalized CLI `Source`.

Consumers using `#[derive(FieldDefs)]` add `docnav-typed-fields` as a direct dependency for the derive macro and trait; the core facade intentionally re-exports the canonical parameter types, not the derive macro.

The companion supports strings, integers, finite numbers, `SetTrue` booleans, repeated string
arrays, and repeated `key=value` objects. Omitted arguments do not create candidates. Values that
cannot be decoded remain invalid candidate facts; unregistered flags retain `clap`'s native
`UnknownArgument` behavior. Resolution and canonical validation stay in the core and typed-fields
packages.

## End-to-end example

The runnable [resolution flow](examples/resolution_flow.rs) uses one derived canonical declaration
for CLI, environment, structured config, static defaults, merge, validation, materialization, and
provenance:

```console
cargo run --locked -p cli-config-resolution-clap --example resolution_flow
```

See the [`cli-config-resolution` core](../cli-config-resolution/README.md) for shared resolution
rules. Workspace-wide checks run from the Docnav repository root.
