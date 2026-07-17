# cli-config-resolution-serde

`serde_json::Value` config extraction for `cli-config-resolution`. This companion reads only config
paths declared by the canonical `FieldDefSet` and returns the shared core `Source` model.

## Entry point

`extract_config` maps values at declared config paths into normalized candidates. Missing declared
paths, non-object intermediate values, and undeclared config keys produce no candidate. Validation,
merge, defaults, and materialization remain owned by the canonical fields and resolution core.

## Ownership boundary

This crate owns only Serde-backed config extraction. Direct field construction, environment
extraction, the four merge strategies, layered validation, defaults, materialization, diagnostics,
and provenance remain in `docnav-typed-fields` and the resolution core.

See the [`cli-config-resolution` core](../cli-config-resolution/README.md) for shared resolution
rules. Workspace-wide checks run from the Docnav repository root.
