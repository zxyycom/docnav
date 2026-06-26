use super::*;
use serde_json::json;

// @case WB-TYPED-FIELDS-PRESENCE-001
#[derive(Debug, FieldDefs)]
struct PresenceParams {
    #[field(group)]
    defaults: PresenceDefaults,
}

#[derive(Debug, FieldDefs)]
struct PresenceDefaults {
    #[field(
        FieldDef::builder("docnav.defaults.title")
            .process(CONFIG_PROCESSING, config_json_path(["defaults", "title"]))
            .validation(FieldValidation::string())
    )]
    title: String,

    #[field(
        FieldDef::builder("docnav.defaults.subtitle")
            .process(CONFIG_PROCESSING, config_json_path(["defaults", "subtitle"]))
            .validation(FieldValidation::string())
    )]
    subtitle: Option<String>,
}

fn presence_fields() -> <PresenceParams as FieldDefs>::DefinitionSet {
    PresenceParams::field_defs().expect("presence defaults build")
}

#[test]
fn declaration_type_projects_required_and_nullable_metadata() {
    let fields = presence_fields();
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
}

#[test]
fn required_declaration_reports_missing_and_null_failures() {
    let fields = presence_fields();
    let missing_required = fields
        .extract(
            CONFIG_PROCESSING,
            &json!({"defaults": {"subtitle": "optional"}}),
        )
        .expect_err("required field must be present");
    assert_eq!(
        validation_failures(&missing_required)[0].reason,
        ValidationReason::MissingRequired
    );

    let null_required = fields
        .extract(CONFIG_PROCESSING, &json!({"defaults": {"title": null}}))
        .expect_err("required field rejects null");
    assert_eq!(
        validation_failures(&null_required)[0].reason,
        ValidationReason::WrongType {
            expected: ValueKind::String,
            actual: ActualValueKind::Null,
        }
    );
}

#[test]
fn optional_declaration_extracts_absent_null_and_present_values() {
    let fields = presence_fields();
    let missing_optional = fields
        .extract(
            CONFIG_PROCESSING,
            &json!({"defaults": {"title": "required"}}),
        )
        .expect("missing optional field extracts");
    assert_eq!(missing_optional.defaults.subtitle, None);

    let null_optional = fields
        .extract(
            CONFIG_PROCESSING,
            &json!({"defaults": {"title": "required", "subtitle": null}}),
        )
        .expect("optional null field extracts as None");
    assert_eq!(null_optional.defaults.subtitle, None);

    let present_optional = fields
        .extract(
            CONFIG_PROCESSING,
            &json!({"defaults": {"title": "required", "subtitle": "value"}}),
        )
        .expect("present optional field extracts");
    assert_eq!(
        present_optional.defaults.subtitle,
        Some("value".to_string())
    );
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
