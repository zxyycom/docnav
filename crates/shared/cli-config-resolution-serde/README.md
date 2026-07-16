# cli-config-resolution-serde

`serde_json::Value` config extraction for `cli-config-resolution`. This companion reads only config
paths declared by the canonical `FieldDefSet` and returns the shared core `Source` model.

## Entry point

`extract_config` maps values at declared config paths into normalized candidates. Missing declared
paths, non-object intermediate values, and undeclared config keys produce no candidate. Validation,
merge, defaults, and materialization remain owned by the canonical fields and resolution core.

## End-to-end example

The clap companion's runnable
[resolution flow](../cli-config-resolution-clap/examples/resolution_flow.rs) passes the same derived
canonical declaration through CLI, environment, structured config, defaults, resolution,
materialization, and provenance:

```console
cargo run --locked -p cli-config-resolution-clap --example resolution_flow
```

See the [`cli-config-resolution` core](../cli-config-resolution/README.md) for shared resolution
rules. Workspace-wide checks run from the Docnav repository root.
