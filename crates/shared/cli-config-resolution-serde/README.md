# cli-config-resolution-serde

`serde_json::Value` integration for `cli-config-resolution`. It maps declared config paths to core source candidates without adding config-file layout policy to the resolver.

## Usage

```rust
use cli_config_resolution::{
    FieldContract, FieldProjectionDeclaration, FieldSet, SourceId, SourceKind, SourceSpec,
    ValueKind,
};
use cli_config_resolution_serde::candidates_from_json_value;
use serde_json::json;

let fields = FieldSet::builder()
    .add_field(
        FieldContract::builder("limit", ValueKind::Integer)
            .projection(
                FieldProjectionDeclaration::config_path(["read", "limit"])
                    .expect("config path"),
            )
            .build()
            .expect("limit field"),
    )
    .build()
    .expect("field set");
let config = SourceSpec::new(
    SourceId::new("config").expect("source id"),
    SourceKind::Config,
    20,
);

let candidates =
    candidates_from_json_value(&json!({ "read": { "limit": 10 } }), &config, &fields);
assert_eq!(candidates.len(), 1);
```

## API status

The current workspace API is pre-1.0 and is not covered by a compatibility guarantee. Its surface may change with the core package while the independent package boundary is audited.
