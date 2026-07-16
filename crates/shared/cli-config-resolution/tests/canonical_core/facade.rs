use cli_config_resolution::{
    extract_env, resolve, DefaultMetadata, ExpectedFieldShape, FieldBound, FieldDef, FieldDefSet,
    FieldLength, FieldStringEnum, FieldValidation, JsonValue, MergeStrategy, ProcessStrategy,
    ProcessingId, SourceId, TypedValue,
};
use serde_json::json;

use super::support::identity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FacadeMode {
    Readable,
    Json,
}

impl FieldStringEnum for FacadeMode {
    fn variants() -> &'static [Self] {
        &[Self::Readable, Self::Json]
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Readable => "readable",
            Self::Json => "json",
        }
    }
}

#[test]
fn primary_facade_builds_constrained_canonical_parameters() {
    let parameters = FieldDefSet::builder()
        .field(
            FieldDef::builder("items")
                .process("env", ProcessStrategy::env_var("APP_ITEMS"))
                .validation(FieldValidation::array().length(FieldLength::between(
                    FieldBound::closed(1),
                    FieldBound::closed(3),
                )))
                .default_static(vec![json!("default")])
                .merge(MergeStrategy::Append),
            ExpectedFieldShape::optional(),
        )
        .field(
            FieldDef::builder("mode")
                .process("env", ProcessStrategy::env_var("APP_MODE"))
                .validation(FieldValidation::string_enum::<FacadeMode>())
                .default_static(FacadeMode::Readable),
            ExpectedFieldShape::optional(),
        )
        .build()
        .expect("canonical parameter set");

    let field = parameters.field(&identity("items")).expect("parameter");
    assert_eq!(field.merge_strategy(), MergeStrategy::Append);
    assert!(matches!(
        field.schema_metadata().default,
        DefaultMetadata::Static(_)
    ));
    assert!(matches!(
        parameters
            .field(&identity("mode"))
            .expect("enum parameter")
            .schema_metadata()
            .default,
        DefaultMetadata::Static(JsonValue::String(ref value)) if value == "readable"
    ));
}

#[test]
fn canonical_parameter_set_drives_env_resolution() {
    let canonical = FieldDefSet::builder()
        .field(
            FieldDef::builder("limit")
                .process("env", ProcessStrategy::env_var("APP_LIMIT"))
                .validation(FieldValidation::int())
                .default_static(20),
            ExpectedFieldShape::required(),
        )
        .build()
        .expect("canonical definitions");
    let _: &FieldDef = canonical.field(&identity("limit")).expect("parameter");
    let env = extract_env(
        &canonical,
        &ProcessingId::new("env").expect("valid processing id"),
        SourceId::new("environment").expect("source id"),
        30,
        [("APP_LIMIT".to_owned(), "42".to_owned())],
    )
    .expect("env source");
    let result = resolve(&canonical, &[env]).expect("valid resolver input");

    assert_eq!(
        result.materialize().expect("canonical values")[&identity("limit")],
        TypedValue::Integer(42)
    );
}
