# cli-config-resolution-clap

`clap` integration for `cli-config-resolution`. It derives arguments from CLI field projections and converts `ArgMatches` into core source candidates; resolution remains in the core package.

## Usage

```rust
use clap::Command;
use cli_config_resolution::{
    FieldContract, FieldProjectionDeclaration, FieldSet, SourceId, SourceKind, SourceSpec,
    ValueKind,
};
use cli_config_resolution_clap::{augment_command, candidates_from_matches};

let fields = FieldSet::builder()
    .add_field(
        FieldContract::builder("limit", ValueKind::Integer)
            .projection(FieldProjectionDeclaration::cli_flag("--limit"))
            .build()
            .expect("limit field"),
    )
    .build()
    .expect("field set");
let command = augment_command(Command::new("demo"), &fields).expect("command");
let matches = command
    .try_get_matches_from(["demo", "--limit", "10"])
    .expect("arguments");
let cli = SourceSpec::new(
    SourceId::new("cli").expect("source id"),
    SourceKind::Cli,
    100,
);

let candidates = candidates_from_matches(&matches, &cli, &fields);
assert_eq!(candidates.len(), 1);
```

The runnable `examples/resolution_flow.rs` example combines CLI, environment, JSON config, defaults, merge strategies, provenance, and conflict diagnostics:

```console
cargo run -p cli-config-resolution-clap --example resolution_flow
```

## API status

The current workspace API is pre-1.0 and is not covered by a compatibility guarantee. Its surface may change with the core package while the independent package boundary is audited.
