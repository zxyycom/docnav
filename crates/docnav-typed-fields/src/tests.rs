#![allow(dead_code)]

use super::*;
use serde_json::json;

mod constraints;
mod set_projection;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputMode {
    ReadableView,
    ReadableJson,
    ProtocolJson,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EmptyMode {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DuplicateMode {
    ReadableView,
    ReadableViewAlias,
    ReadableJson,
    ProtocolJson,
}

impl FieldStringEnum for OutputMode {
    fn variants() -> &'static [Self] {
        &[Self::ReadableView, Self::ReadableJson, Self::ProtocolJson]
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::ReadableView => "readable-view",
            Self::ReadableJson => "readable-json",
            Self::ProtocolJson => "protocol-json",
        }
    }
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

fn limit_chars_validation() -> FieldValidation<i64> {
    FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(100_000))
}

fn output_mode_validation() -> FieldValidation<OutputMode> {
    FieldValidation::string_enum::<OutputMode>()
}

#[derive(Debug, FieldDefs)]
pub(crate) struct DocnavParams {
    #[field(group)]
    defaults: DefaultsParams,
}

#[derive(Debug, FieldDefs)]
pub(crate) struct DefaultsParams {
    #[field(
        FieldDef::builder("docnav.defaults.limit_chars")
            .path(["a", "b"])
            .validation(limit_chars_validation())
            .default_static(20_000)
    )]
    limit_chars: Option<i64>,

    #[field(
        FieldDef::builder("docnav.defaults.output")
            .path(["defaults", "output"])
            .validation(output_mode_validation().required())
            .default_static(OutputMode::ReadableView)
    )]
    output: OutputMode,
}

// @case WB-TYPED-FIELDS-001
fn limit_chars_field() -> FieldDef {
    FieldDef::builder("docnav.defaults.limit_chars")
        .path(["defaults", "limit_chars"])
        .validation(
            FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(100_000)),
        )
        .default_static(20_000)
        .build()
        .expect("limit_chars field builds")
}

#[test]
fn builder_exposes_schema_metadata_and_validates_values() {
    let field = limit_chars_field();

    let schema = field.schema_metadata();
    assert_eq!(schema.identity.as_str(), "docnav.defaults.limit_chars");
    assert_eq!(schema.path.segments(), ["defaults", "limit_chars"]);
    assert_eq!(schema.value_kind, ValueKind::Integer);
    assert_eq!(schema.default, DefaultMetadata::Static(json!(20_000)));
    assert_eq!(
        schema.constraints.numeric_range,
        FieldNumericRange::Integer(FieldRange::between(
            FieldBound::closed(1),
            FieldBound::closed(100_000),
        ))
    );

    let value = field
        .decode_without_default(&json!({"defaults": {"limit_chars": 4000}}))
        .expect("valid value decodes");
    assert_eq!(value, Some(TypedValue::Integer(4000)));
}

#[test]
fn validation_failures_keep_field_attribution() {
    let field = limit_chars_field();

    let missing = field
        .decode_without_default(&json!({"defaults": {}}))
        .unwrap();
    assert_eq!(missing, None);

    let error = field
        .decode_without_default(&json!({"defaults": {"limit_chars": "4000"}}))
        .expect_err("wrong type fails");
    assert_eq!(error.field.as_str(), "docnav.defaults.limit_chars");
    assert_eq!(error.path.segments(), ["defaults", "limit_chars"]);
    assert_eq!(
        error.reason,
        ValidationReason::WrongType {
            expected: ValueKind::Integer,
            actual: ActualValueKind::String
        }
    );

    let error = field
        .decode_without_default(&json!({"defaults": {"limit_chars": 0}}))
        .expect_err("range violation fails");
    assert_eq!(
        error.reason,
        ValidationReason::BelowMinimum {
            minimum: FieldNumericBound::Integer(FieldBound::closed(1))
        }
    );
}

#[test]
fn required_and_enum_constraints_are_field_level_validation() {
    let required = FieldDef::builder("docnav.defaults.output")
        .path(["defaults", "output"])
        .validation(FieldValidation::string_enum::<OutputMode>().required())
        .build()
        .expect("output field builds");

    let error = required
        .decode_without_default(&json!({"defaults": {}}))
        .expect_err("missing required field fails");
    assert_eq!(error.reason, ValidationReason::MissingRequired);

    let value = required
        .decode_without_default(&json!({"defaults": {"output": "readable-json"}}))
        .expect("allowed enum value passes");
    assert_eq!(value, Some(TypedValue::String("readable-json".to_string())));

    let error = required
        .decode_without_default(&json!({"defaults": {"output": "xml"}}))
        .expect_err("disallowed enum value fails");
    assert!(matches!(
        error.reason,
        ValidationReason::DisallowedEnumValue { .. }
    ));
}

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

#[test]
fn set_build_rejects_declaration_shape_mismatch() {
    #[derive(Debug, FieldDefs)]
    struct Params {
        #[field(group)]
        defaults: ShapeMismatchDefaults,
    }

    #[derive(Debug, FieldDefs)]
    struct ShapeMismatchDefaults {
        #[field(
            FieldDef::builder("docnav.defaults.limit_chars")
                .path(["defaults", "limit_chars"])
                .validation(FieldValidation::int())
        )]
        limit_chars: i64,
    }

    let error = Params::field_defs()
        .expect_err("declared required presence must match validation metadata");

    assert_eq!(
        error,
        FieldDefSetBuildError::DeclarationShape(FieldDeclarationShapeError {
            declaration_path: Some(vec!["defaults".to_string(), "limit_chars".to_string()]),
            expected: ExpectedFieldShape { required: true },
            actual: ExpectedFieldShape { required: false },
        })
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
