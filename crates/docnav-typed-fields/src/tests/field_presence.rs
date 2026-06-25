use super::*;
use serde_json::json;

#[test]
fn field_declaration_type_controls_requiredness_and_optional_nulls() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: PresenceDefaults,
    }

    #[derive(Debug, FieldDefs)]
    struct PresenceDefaults {
        #[field(
            FieldDef::builder("docnav.defaults.title")
                .path(["defaults", "title"])
                .validation(FieldValidation::string())
        )]
        title: String,

        #[field(
            FieldDef::builder("docnav.defaults.subtitle")
                .path(["defaults", "subtitle"])
                .validation(FieldValidation::string())
        )]
        subtitle: Option<String>,
    }

    let fields = Params::field_defs().expect("presence defaults build");
    let schema = fields.schema_metadata();
    let title_schema = schema
        .iter()
        .find(|metadata| metadata.identity.as_str() == "docnav.defaults.title")
        .expect("title schema exists");
    assert!(title_schema.constraints.required);
    assert!(!title_schema.constraints.nullable);

    let subtitle_schema = schema
        .iter()
        .find(|metadata| metadata.identity.as_str() == "docnav.defaults.subtitle")
        .expect("subtitle schema exists");
    assert!(!subtitle_schema.constraints.required);
    assert!(subtitle_schema.constraints.nullable);

    let missing_required = fields
        .extract_without_default(&json!({"defaults": {"subtitle": "optional"}}))
        .expect_err("required field must be present");
    assert_eq!(
        missing_required.failures()[0].reason,
        ValidationReason::MissingRequired
    );

    let null_required = fields
        .extract_without_default(&json!({"defaults": {"title": null}}))
        .expect_err("required field rejects null");
    assert_eq!(
        null_required.failures()[0].reason,
        ValidationReason::WrongType {
            expected: ValueKind::String,
            actual: ActualValueKind::Null,
        }
    );

    let missing_optional = fields
        .extract_without_default(&json!({"defaults": {"title": "required"}}))
        .expect("missing optional field extracts");
    assert_eq!(missing_optional.defaults.subtitle, None);

    let null_optional = fields
        .extract_without_default(&json!({"defaults": {"title": "required", "subtitle": null}}))
        .expect("optional null field extracts as None");
    assert_eq!(null_optional.defaults.subtitle, None);

    let present_optional = fields
        .extract_without_default(&json!({"defaults": {"title": "required", "subtitle": "value"}}))
        .expect("present optional field extracts");
    assert_eq!(
        present_optional.defaults.subtitle,
        Some("value".to_string())
    );
}

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
                .path(["defaults", "limit_chars"])
                .validation(FieldValidation::int())
        )]
        limit_chars: Option<i64>,

        #[field(
            FieldDef::builder("docnav.defaults.limit_chars")
                .path(["defaults", "limit_chars"])
                .validation(FieldValidation::int())
        )]
        max_chars: Option<i64>,
    }

    let error = Params::field_defs().expect_err("duplicate identity fails");

    let FieldDefSetBuildError::DuplicateIdentity(error) = error else {
        panic!("expected duplicate identity error");
    };
    assert_eq!(error.field.as_str(), "docnav.defaults.limit_chars");
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
                .path(["defaults", "output"])
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
                .path(["defaults", "output"])
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
