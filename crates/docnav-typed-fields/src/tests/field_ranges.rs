use super::*;
use serde_json::json;

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
            FieldDef::builder("docnav.defaults.limit_chars")
                .path(["defaults", "limit_chars"])
                .validation(
                    FieldValidation::int()
                        .between(FieldBound::closed(1), FieldBound::closed(100_000)),
                )
                .default_static(0)
        )]
        limit_chars: Option<i64>,
    }

    let error = Params::field_defs().expect_err("invalid static default fails");

    let FieldDefSetBuildError::Field(error) = error else {
        panic!("expected field build error");
    };
    assert_eq!(
        error.declaration_path,
        Some(vec!["defaults".to_string(), "limit_chars".to_string()])
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
fn single_sided_and_open_numeric_bounds_validate_values() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: OpenBoundDefaults,
    }

    #[derive(Debug, FieldDefs)]
    struct OpenBoundDefaults {
        #[field(
            FieldDef::builder("docnav.defaults.limit_chars")
                .path(["defaults", "limit_chars"])
                .validation(FieldValidation::int().min(FieldBound::open(1)))
        )]
        limit_chars: Option<i64>,
    }

    let fields = Params::field_defs().expect("single-sided minimum builds");

    let error = fields
        .extract_without_default(&json!({"defaults": {"limit_chars": 1}}))
        .expect_err("open minimum excludes the endpoint");
    assert_eq!(
        error.failures()[0].reason,
        ValidationReason::BelowMinimum {
            minimum: FieldNumericBound::Integer(FieldBound::open(1))
        }
    );

    fields
        .extract_without_default(&json!({"defaults": {"limit_chars": 2}}))
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
            FieldDef::builder("docnav.defaults.limit_chars")
                .path(["defaults", "limit_chars"])
                .validation(FieldValidation::int().max(FieldBound::closed(MAXIMUM)))
        )]
        limit_chars: Option<i64>,
    }

    let fields = Params::field_defs().expect("large integer maximum builds");

    let error = fields
        .extract_without_default(&json!({"defaults": {"limit_chars": MAXIMUM + 1}}))
        .expect_err("integer comparison keeps exact precision");
    assert_eq!(
        error.failures()[0].reason,
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
            FieldDef::builder("docnav.defaults.limit_chars")
                .path(["defaults", "limit_chars"])
                .validation(
                    FieldValidation::num()
                        .between(FieldBound::closed(1.0), FieldBound::closed(f64::INFINITY)),
                )
        )]
        limit_chars: Option<f64>,
    }

    let error = NonFiniteParams::field_defs().expect_err("non-finite range fails");

    assert_eq!(
        error,
        FieldDefSetBuildError::Field(FieldDefBuildFailure {
            declaration_path: Some(vec!["defaults".to_string(), "limit_chars".to_string()]),
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
            FieldDef::builder("docnav.defaults.limit_chars")
                .path(["defaults", "limit_chars"])
                .validation(
                    FieldValidation::int()
                        .between(FieldBound::open(1), FieldBound::closed(1)),
                )
        )]
        limit_chars: Option<i64>,
    }

    let error = EmptyRangeParams::field_defs().expect_err("empty open range fails");

    assert_eq!(
        error,
        FieldDefSetBuildError::Field(FieldDefBuildFailure {
            declaration_path: Some(vec!["defaults".to_string(), "limit_chars".to_string()]),
            error: BuildError::InvalidRange,
        })
    );
}

#[test]
fn set_build_rejects_missing_field_path() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: MissingPathDefaults,
    }

    #[derive(Debug, FieldDefs)]
    struct MissingPathDefaults {
        #[field(FieldDef::builder("docnav.defaults.limit_chars").validation(FieldValidation::int()))]
        limit_chars: Option<i64>,
    }

    let error = Params::field_defs().expect_err("missing path fails at set build");

    assert_eq!(
        error,
        FieldDefSetBuildError::Field(FieldDefBuildFailure {
            declaration_path: Some(vec!["defaults".to_string(), "limit_chars".to_string()]),
            error: BuildError::MissingPath,
        })
    );
}
