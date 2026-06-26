use super::*;
use serde_json::json;

#[test]
fn string_regex_and_length_constraints_validate_present_values() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: Defaults,
    }

    #[derive(Debug, FieldDefs)]
    struct Defaults {
        #[field(
            FieldDef::builder("docnav.defaults.slug")
                .extract(CONFIG_STRATEGY, config_json_path(["defaults", "slug"]))
                .validation(
                    FieldValidation::string()
                        .regex(r"^[a-z][a-z0-9-]*$")
                        .length(FieldLength::between(FieldBound::closed(3), FieldBound::open(8))),
                )
        )]
        slug: Option<String>,
    }

    let fields = Params::field_defs().expect("string regex and length constraints build");

    fields
        .extract(CONFIG_STRATEGY, &json!({"defaults": {"slug": "doc-1"}}))
        .expect("valid slug passes");

    let error = fields
        .extract(CONFIG_STRATEGY, &json!({"defaults": {"slug": "Doc"}}))
        .expect_err("regex mismatch fails");
    assert_eq!(
        validation_failures(&error)[0].reason,
        ValidationReason::RegexMismatch {
            pattern: r"^[a-z][a-z0-9-]*$".to_string(),
        }
    );

    let error = fields
        .extract(CONFIG_STRATEGY, &json!({"defaults": {"slug": "do"}}))
        .expect_err("short string fails");
    assert_eq!(
        validation_failures(&error)[0].reason,
        ValidationReason::BelowMinimumLength {
            minimum: FieldBound::closed(3)
        }
    );

    let error = fields
        .extract(CONFIG_STRATEGY, &json!({"defaults": {"slug": "docnav-x"}}))
        .expect_err("open upper length bound excludes endpoint");
    assert_eq!(
        validation_failures(&error)[0].reason,
        ValidationReason::AboveMaximumLength {
            maximum: FieldBound::open(8)
        }
    );
}

#[test]
fn array_length_constraints_validate_present_values() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: Defaults,
    }

    #[derive(Debug, FieldDefs)]
    struct Defaults {
        #[field(
            FieldDef::builder("docnav.defaults.items")
                .extract(CONFIG_STRATEGY, config_json_path(["defaults", "items"]))
                .validation(
                    FieldValidation::array()
                        .length(FieldLength::max(FieldBound::closed(2))),
                )
        )]
        items: Option<Vec<JsonValue>>,
    }

    let fields = Params::field_defs().expect("array length constraints build");

    fields
        .extract(CONFIG_STRATEGY, &json!({"defaults": {"items": [1, 2]}}))
        .expect("array at the maximum length passes");

    let error = fields
        .extract(CONFIG_STRATEGY, &json!({"defaults": {"items": [1, 2, 3]}}))
        .expect_err("array above maximum length fails");
    assert_eq!(
        validation_failures(&error)[0].reason,
        ValidationReason::AboveMaximumLength {
            maximum: FieldBound::closed(2)
        }
    );
}

#[test]
fn set_build_rejects_invalid_regex_metadata() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: Defaults,
    }

    #[derive(Debug, FieldDefs)]
    struct Defaults {
        #[field(
            FieldDef::builder("docnav.defaults.slug")
                .extract(CONFIG_STRATEGY, config_json_path(["defaults", "slug"]))
                .validation(FieldValidation::string().regex("["))
        )]
        slug: Option<String>,
    }

    let error = Params::field_defs().expect_err("invalid regex pattern fails at set build");

    assert!(matches!(
        error,
        FieldDefSetBuildError::Field(FieldDefBuildFailure {
            error: BuildError::InvalidRegexPattern { .. },
            ..
        })
    ));
}
