# cli-config-resolution

Framework-independent field contracts and ordered source resolution for Rust CLI configuration. Applications retain ownership of command structure, config layout, and public diagnostics.

## Usage

```rust
use cli_config_resolution::{
    DefaultMetadata, DefaultSource, FieldContract, FieldSet, Resolver, SourceCollection,
    SourceExtractor, SourceId, SourceKind, SourceSpec, Value, ValueKind,
};

let fields = FieldSet::builder()
    .add_field(
        FieldContract::builder("limit", ValueKind::Integer)
            .default(DefaultMetadata::Static(Value::Integer(20)))
            .build()
            .expect("limit field"),
    )
    .build()
    .expect("field set");
let defaults = SourceSpec::new(
    SourceId::new("defaults").expect("source id"),
    SourceKind::Default,
    0,
);
let sources = SourceCollection::new(vec![defaults.clone()]).expect("sources");
let candidates = DefaultSource::default().extract(&defaults, &fields);

let result = Resolver::resolve(&fields, &sources, candidates);
let values = result.materialize().expect("valid configuration");
assert_eq!(values.len(), 1);
```

## API status

The current workspace API is pre-1.0 and is not covered by a compatibility guarantee. Public items may change while the independent package boundary is audited.
