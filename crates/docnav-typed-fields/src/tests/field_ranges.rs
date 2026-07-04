use super::*;
use serde_json::json;

// @case WB-TYPED-FIELDS-RANGES-001
#[test]
fn set_build_rejects_invalid_default_metadata() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: InvalidDefaultDefaults,
    }

    #[derive(Debug, FieldDefs)]
    struct InvalidDefaultDefaults {
        #[field(
            FieldDef::builder("docnav.defaults.limit")
                .process(CONFIG_PROCESSING, config_json_path(["defaults", "limit"]))
                .validation(
                    FieldValidation::int()
                        .between(FieldBound::closed(1), FieldBound::closed(100_000)),
                )
                .default_static(0)
        )]
        limit: Option<i64>,
    }

    let error = Params::field_defs().expect_err("invalid static default fails");

    let FieldDefSetBuildError::Field(error) = error else {
        panic!("expected field build error");
    };
    assert_eq!(
        error.declaration_path,
        Some(vec!["defaults".to_string(), "limit".to_string()])
    );
    let BuildError::InvalidDefault(default_error) = error.error else {
        panic!("expected invalid default error");
    };
    assert_eq!(
        default_error.reason,
        ValidationReason::BelowMinimum {
            minimum: FieldNumericBound::Integer(FieldBound::closed(1))
        }
    );
}

#[test]
fn set_build_rejects_non_finite_number_default_metadata() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: NonFiniteDefaultDefaults,
    }

    #[derive(Debug, FieldDefs)]
    struct NonFiniteDefaultDefaults {
        #[field(
            FieldDef::builder("docnav.defaults.ratio")
                .process(CONFIG_PROCESSING, config_json_path(["defaults", "ratio"]))
                .validation(FieldValidation::num())
                .default_static(f64::INFINITY)
        )]
        ratio: Option<f64>,
    }

    let error = Params::field_defs().expect_err("non-finite static default fails");

    assert!(error
        .to_string()
        .contains("Rust f64 can represent non-finite values, but JSON numbers cannot"));

    let FieldDefSetBuildError::Field(error) = error else {
        panic!("expected field build error");
    };
    assert_eq!(
        error.declaration_path,
        Some(vec!["defaults".to_string(), "ratio".to_string()])
    );
    assert_eq!(error.error, BuildError::NonFiniteDefaultValue);
}

#[test]
fn single_sided_and_open_numeric_bounds_validate_values() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: OpenBoundDefaults,
    }

    #[derive(Debug, FieldDefs)]
    struct OpenBoundDefaults {
        #[field(
            FieldDef::builder("docnav.defaults.limit")
                .process(CONFIG_PROCESSING, config_json_path(["defaults", "limit"]))
                .validation(FieldValidation::int().min(FieldBound::open(1)))
        )]
        limit: Option<i64>,
    }

    let fields = Params::field_defs().expect("single-sided minimum builds");

    let error = fields
        .extract(CONFIG_PROCESSING, &json!({"defaults": {"limit": 1}}))
        .expect_err("open minimum excludes the endpoint");
    assert_eq!(
        validation_failures(&error)[0].reason,
        ValidationReason::BelowMinimum {
            minimum: FieldNumericBound::Integer(FieldBound::open(1))
        }
    );

    fields
        .extract(CONFIG_PROCESSING, &json!({"defaults": {"limit": 2}}))
        .expect("single-sided open minimum accepts larger values");
}

#[test]
fn integer_range_does_not_use_float_precision() {
    const MAXIMUM: i64 = 9_007_199_254_740_992;

    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: LargeIntegerDefaults,
    }

    #[derive(Debug, FieldDefs)]
    struct LargeIntegerDefaults {
        #[field(
            FieldDef::builder("docnav.defaults.limit")
                .process(CONFIG_PROCESSING, config_json_path(["defaults", "limit"]))
                .validation(FieldValidation::int().max(FieldBound::closed(MAXIMUM)))
        )]
        limit: Option<i64>,
    }

    let fields = Params::field_defs().expect("large integer maximum builds");

    let error = fields
        .extract(
            CONFIG_PROCESSING,
            &json!({"defaults": {"limit": MAXIMUM + 1}}),
        )
        .expect_err("integer comparison keeps exact precision");
    assert_eq!(
        validation_failures(&error)[0].reason,
        ValidationReason::AboveMaximum {
            maximum: FieldNumericBound::Integer(FieldBound::closed(MAXIMUM))
        }
    );
}

#[test]
fn set_build_rejects_non_finite_or_empty_range_metadata() {
    #[derive(Debug, FieldDefs)]
    struct NonFiniteParams {
        #[field(group)]
        defaults: NonFiniteDefaults,
    }

    #[derive(Debug, FieldDefs)]
    struct NonFiniteDefaults {
        #[field(
            FieldDef::builder("docnav.defaults.limit")
                .process(CONFIG_PROCESSING, config_json_path(["defaults", "limit"]))
                .validation(
                    FieldValidation::num()
                        .between(FieldBound::closed(1.0), FieldBound::closed(f64::INFINITY)),
                )
        )]
        limit: Option<f64>,
    }

    let error = NonFiniteParams::field_defs().expect_err("non-finite range fails");

    assert_eq!(
        error,
        FieldDefSetBuildError::Field(FieldDefBuildFailure {
            declaration_path: Some(vec!["defaults".to_string(), "limit".to_string()]),
            error: BuildError::NonFiniteRangeBound,
        })
    );

    #[derive(Debug, FieldDefs)]
    struct EmptyRangeParams {
        #[field(group)]
        defaults: EmptyRangeDefaults,
    }

    #[derive(Debug, FieldDefs)]
    struct EmptyRangeDefaults {
        #[field(
            FieldDef::builder("docnav.defaults.limit")
                .process(CONFIG_PROCESSING, config_json_path(["defaults", "limit"]))
                .validation(
                    FieldValidation::int()
                        .between(FieldBound::open(1), FieldBound::closed(1)),
                )
        )]
        limit: Option<i64>,
    }

    let error = EmptyRangeParams::field_defs().expect_err("empty open range fails");

    assert_eq!(
        error,
        FieldDefSetBuildError::Field(FieldDefBuildFailure {
            declaration_path: Some(vec!["defaults".to_string(), "limit".to_string()]),
            error: BuildError::InvalidRange,
        })
    );
}
