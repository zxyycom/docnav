use cli_config_resolution::{
    ExpectedFieldShape, FieldDef, FieldDefSet, FieldIdentity, FieldPath, FieldValidation,
    JsonValue, MergeStrategy, ProcessStrategy, Source, SourceCandidate, SourceId, SourceKind,
    SourceLocator,
};

pub(super) fn identity(value: &str) -> FieldIdentity {
    FieldIdentity::new(value).expect("field identity")
}

pub(super) fn candidate(field: &str, value: JsonValue) -> SourceCandidate {
    SourceCandidate::value(identity(field), direct_locator(field), value)
}

pub(super) fn invalid_candidate(field: &str, raw: JsonValue, reason: &str) -> SourceCandidate {
    SourceCandidate::invalid(identity(field), direct_locator(field), raw, reason)
}

pub(super) fn source(
    id: &str,
    priority: i32,
    candidates: impl IntoIterator<Item = SourceCandidate>,
) -> Source {
    Source::new(
        SourceId::new(id).expect("source id"),
        SourceKind::Direct,
        priority,
        candidates.into_iter().collect(),
    )
    .expect("valid source")
}

pub(super) fn direct_locator(path: &str) -> SourceLocator {
    SourceLocator::DirectPath(
        FieldPath::new(path.split('.')).expect("direct source path must be valid"),
    )
}

pub(super) fn custom_field_set(identity: &str, required: bool) -> FieldDefSet {
    let shape = if required {
        ExpectedFieldShape::required()
    } else {
        ExpectedFieldShape::optional()
    };
    FieldDefSet::builder()
        .field(
            FieldDef::builder(identity)
                .process("custom", ProcessStrategy::rust_field())
                .validation(FieldValidation::string()),
            shape,
        )
        .build()
        .expect("field set")
}

pub(super) fn mode_field_set() -> FieldDefSet {
    FieldDefSet::builder()
        .field(
            FieldDef::builder("mode")
                .process("custom", ProcessStrategy::rust_field())
                .validation(FieldValidation::string())
                .default_static("default"),
            ExpectedFieldShape::required(),
        )
        .build()
        .expect("mode field set")
}

pub(super) fn merge_field_set<T: 'static>(
    identity: &str,
    validation: FieldValidation<T>,
    strategy: MergeStrategy,
) -> FieldDefSet {
    FieldDefSet::builder()
        .field(
            FieldDef::builder(identity)
                .process("custom", ProcessStrategy::rust_field())
                .validation(validation)
                .merge(strategy),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("merge field set")
}
