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
