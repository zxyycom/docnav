use super::*;
use serde_json::json;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EmptyMode {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DuplicateMode {
    ReadableView,
    ReadableViewAlias,
    ReadableJson,
    ProtocolJson,
}

impl FieldStringEnum for EmptyMode {
    fn variants() -> &'static [Self] {
        &[]
    }

    fn as_str(&self) -> &'static str {
        match *self {}
    }
}

impl FieldStringEnum for DuplicateMode {
    fn variants() -> &'static [Self] {
        &[
            Self::ReadableView,
            Self::ReadableViewAlias,
            Self::ReadableJson,
            Self::ProtocolJson,
        ]
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::ReadableView | Self::ReadableViewAlias => "readable-view",
            Self::ReadableJson => "readable-json",
            Self::ProtocolJson => "protocol-json",
        }
    }
}

// @case WB-TYPED-FIELDS-METADATA-001
#[test]
fn set_build_rejects_duplicate_identity() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: DuplicateDefaults,
    }

    #[derive(Debug, FieldDefs)]
    struct DuplicateDefaults {
        #[field(
            FieldDef::builder("docnav.defaults.limit_chars")
                .process(CONFIG_PROCESSING, config_json_path(["defaults", "limit_chars"]))
                .validation(FieldValidation::int())
        )]
        limit_chars: Option<i64>,

        #[field(
            FieldDef::builder("docnav.defaults.limit_chars")
                .process(CONFIG_PROCESSING, config_json_path(["defaults", "max_chars"]))
                .validation(FieldValidation::int())
        )]
        max_chars: Option<i64>,
    }

    let error = Params::field_defs().expect_err("duplicate identity fails");

    let FieldDefSetBuildError::DuplicateIdentity(error) = error else {
        panic!("expected duplicate identity error");
    };
    assert_eq!(error.field.as_str(), "docnav.defaults.limit_chars");
    assert_eq!(
        error.previous_declaration_path,
        Some(vec!["defaults".to_string(), "limit_chars".to_string()])
    );
    assert_eq!(
        error.declaration_path,
        Some(vec!["defaults".to_string(), "max_chars".to_string()])
    );
    assert_eq!(error.previous_path.segments(), ["defaults", "limit_chars"]);
    assert_eq!(error.path.segments(), ["defaults", "max_chars"]);
}

#[test]
fn string_enum_metadata_deduplicates_allowed_values() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: DuplicateEnumDefaults,
    }

    #[derive(Debug, FieldDefs)]
    struct DuplicateEnumDefaults {
        #[field(
            FieldDef::builder("docnav.defaults.output")
                .process(CONFIG_PROCESSING, config_json_path(["defaults", "output"]))
                .validation(FieldValidation::string_enum::<DuplicateMode>())
        )]
        output: Option<DuplicateMode>,
    }

    let fields = Params::field_defs().expect("duplicate enum string aliases build");

    let output = fields
        .schema_metadata()
        .into_iter()
        .find(|metadata| metadata.identity.as_str() == "docnav.defaults.output")
        .expect("output metadata exists");
    assert_eq!(
        output.constraints.enum_values,
        Some(vec![
            json!("readable-view"),
            json!("readable-json"),
            json!("protocol-json")
        ])
    );
}

#[test]
fn string_enum_metadata_must_have_allowed_values() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: EmptyEnumDefaults,
    }

    #[derive(Debug, FieldDefs)]
    struct EmptyEnumDefaults {
        #[field(
            FieldDef::builder("docnav.defaults.output")
                .process(CONFIG_PROCESSING, config_json_path(["defaults", "output"]))
                .validation(FieldValidation::string_enum::<EmptyMode>())
        )]
        output: Option<EmptyMode>,
    }

    let error = Params::field_defs().expect_err("empty enum metadata fails");

    assert_eq!(
        error,
        FieldDefSetBuildError::Field(FieldDefBuildFailure {
            declaration_path: Some(vec!["defaults".to_string(), "output".to_string()]),
            error: BuildError::EmptyEnumValues,
        })
    );
}
