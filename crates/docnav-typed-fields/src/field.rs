use std::marker::PhantomData;

use serde_json::Value;

mod constraints;

use crate::metadata::{
    ActualValueKind, BuildError, DefaultMetadata, FieldConstraints, FieldIdentity, FieldPath,
    SchemaMetadataView, TypedValue, ValidationFailure, ValidationReason, ValueKind,
};
use crate::validation::FieldValidation;
use crate::value::FieldValue;
use constraints::{
    above_maximum, actual_value_kind, below_minimum, compile_regex_pattern,
    numeric_range_violation, validate_length_range, validate_numeric_range, value_at_path,
    value_length, NumericRangeViolation,
};

#[derive(Clone, Debug)]
pub struct FieldDef {
    pub(crate) identity: FieldIdentity,
    pub(crate) path: FieldPath,
    value_kind: ValueKind,
    constraints: FieldConstraints,
    default: DefaultMetadata,
    regex: Option<regex::Regex>,
}

impl FieldDef {
    pub fn builder(identity: impl Into<String>) -> FieldDefBuilder {
        FieldDefBuilder::new(identity)
    }

    pub(crate) fn identity(&self) -> &FieldIdentity {
        &self.identity
    }

    pub(crate) fn schema_metadata(&self) -> SchemaMetadataView {
        SchemaMetadataView {
            identity: self.identity.clone(),
            path: self.path.clone(),
            value_kind: self.value_kind,
            constraints: self.constraints.clone(),
            default: self.default.clone(),
        }
    }

    pub(crate) fn decode_without_default(
        &self,
        root: &Value,
    ) -> Result<Option<TypedValue>, ValidationFailure> {
        let Some(value) = value_at_path(root, &self.path) else {
            return if self.constraints.required {
                Err(self.failure(ValidationReason::MissingRequired))
            } else {
                Ok(None)
            };
        };
        self.validate_present_value(value).map(Some)
    }

    pub(crate) fn decode_with_static_default(
        &self,
        root: &Value,
    ) -> Result<Option<TypedValue>, ValidationFailure> {
        let Some(value) = value_at_path(root, &self.path) else {
            return if let Some(value) = self.static_default_value() {
                Ok(Some(value))
            } else if self.constraints.required {
                Err(self.failure(ValidationReason::MissingRequired))
            } else {
                Ok(None)
            };
        };
        self.validate_present_value(value).map(Some)
    }

    pub(crate) fn static_default_value(&self) -> Option<TypedValue> {
        let DefaultMetadata::Static(value) = &self.default else {
            return None;
        };
        Some(
            self.validate_present_value(value)
                .expect("static default metadata is validated during field build"),
        )
    }

    pub(crate) fn is_required(&self) -> bool {
        self.constraints.required
    }

    fn validate_present_value(&self, value: &Value) -> Result<TypedValue, ValidationFailure> {
        let typed = self.decode_present_value(value)?;
        if matches!(typed, TypedValue::Null) {
            return Ok(typed);
        }
        self.validate_enum_value(value)?;
        self.validate_regex(value)?;
        self.validate_numeric_constraints(&typed)?;
        self.validate_length_constraints(&typed)?;
        Ok(typed)
    }

    fn decode_present_value(&self, value: &Value) -> Result<TypedValue, ValidationFailure> {
        if value.is_null() {
            return self.decode_null();
        }

        match (&self.value_kind, value) {
            (ValueKind::String, Value::String(value)) => Ok(TypedValue::String(value.clone())),
            (ValueKind::Integer, Value::Number(value)) => value
                .as_i64()
                .map(TypedValue::Integer)
                .ok_or_else(|| self.wrong_type(ActualValueKind::Number)),
            (ValueKind::Number, Value::Number(value)) => value
                .as_f64()
                .map(TypedValue::Number)
                .ok_or_else(|| self.wrong_type(ActualValueKind::Number)),
            (ValueKind::Boolean, Value::Bool(value)) => Ok(TypedValue::Boolean(*value)),
            (ValueKind::Array, Value::Array(value)) => Ok(TypedValue::Array(value.clone())),
            (ValueKind::Object, Value::Object(value)) => Ok(TypedValue::Object(value.clone())),
            _ => Err(self.wrong_type(actual_value_kind(value))),
        }
    }

    fn decode_null(&self) -> Result<TypedValue, ValidationFailure> {
        if self.constraints.nullable {
            Ok(TypedValue::Null)
        } else {
            Err(self.wrong_type(ActualValueKind::Null))
        }
    }

    fn validate_enum_value(&self, value: &Value) -> Result<(), ValidationFailure> {
        let Some(enum_values) = &self.constraints.enum_values else {
            return Ok(());
        };

        if enum_values.iter().any(|allowed| allowed == value) {
            return Ok(());
        }

        Err(self.failure(ValidationReason::DisallowedEnumValue {
            allowed: enum_values.clone(),
        }))
    }

    fn validate_numeric_constraints(&self, value: &TypedValue) -> Result<(), ValidationFailure> {
        match numeric_range_violation(&self.constraints, value) {
            Some(NumericRangeViolation::Below(minimum)) => {
                Err(self.failure(ValidationReason::BelowMinimum { minimum }))
            }
            Some(NumericRangeViolation::Above(maximum)) => {
                Err(self.failure(ValidationReason::AboveMaximum { maximum }))
            }
            None => Ok(()),
        }
    }

    fn validate_length_constraints(&self, value: &TypedValue) -> Result<(), ValidationFailure> {
        let Some(length_range) = &self.constraints.length_range else {
            return Ok(());
        };
        let Some(length) = value_length(value) else {
            return Ok(());
        };
        if let Some(minimum) = length_range.minimum {
            if below_minimum(length, minimum) {
                return Err(self.failure(ValidationReason::BelowMinimumLength { minimum }));
            }
        }
        if let Some(maximum) = length_range.maximum {
            if above_maximum(length, maximum) {
                return Err(self.failure(ValidationReason::AboveMaximumLength { maximum }));
            }
        }
        Ok(())
    }

    fn validate_regex(&self, value: &Value) -> Result<(), ValidationFailure> {
        let Some(pattern) = &self.constraints.regex else {
            return Ok(());
        };
        let Value::String(value) = value else {
            return Ok(());
        };
        let regex = self
            .regex
            .as_ref()
            .expect("regex metadata is compiled during field build");
        if regex.is_match(value) {
            Ok(())
        } else {
            Err(self.failure(ValidationReason::RegexMismatch {
                pattern: pattern.clone(),
            }))
        }
    }

    fn wrong_type(&self, actual: ActualValueKind) -> ValidationFailure {
        self.failure(ValidationReason::WrongType {
            expected: self.value_kind,
            actual,
        })
    }

    fn failure(&self, reason: ValidationReason) -> ValidationFailure {
        ValidationFailure {
            field: self.identity.clone(),
            path: self.path.clone(),
            reason,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FieldDefBuilder<T = ()> {
    identity: String,
    path: Option<Vec<String>>,
    validation: Option<FieldValidation<T>>,
    default: DefaultMetadata,
    typed: PhantomData<T>,
}

impl FieldDefBuilder<()> {
    fn new(identity: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
            path: None,
            validation: None,
            default: DefaultMetadata::None,
            typed: PhantomData,
        }
    }
}

impl<T> FieldDefBuilder<T> {
    pub fn path<I, S>(mut self, segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.path = Some(segments.into_iter().map(Into::into).collect());
        self
    }

    pub fn validation<U>(self, validation: FieldValidation<U>) -> FieldDefBuilder<U> {
        FieldDefBuilder {
            identity: self.identity,
            path: self.path,
            validation: Some(validation),
            default: self.default,
            typed: PhantomData,
        }
    }

    pub fn default_static(mut self, value: impl Into<T>) -> Self
    where
        T: FieldValue,
    {
        self.default = DefaultMetadata::Static(value.into().into_json_value());
        self
    }

    pub(crate) fn build(self) -> Result<FieldDef, BuildError> {
        let identity = FieldIdentity::new(self.identity)?;
        let path = FieldPath::new(self.path.ok_or(BuildError::MissingPath)?)?;
        let validation = self.validation.ok_or(BuildError::MissingValidation)?;
        let (value_kind, constraints) = validation.into_parts();
        validate_numeric_range(&constraints)?;
        validate_length_range(&constraints)?;
        let regex = compile_regex_pattern(&constraints)?;

        let definition = FieldDef {
            identity,
            path,
            value_kind,
            constraints,
            default: self.default,
            regex,
        };

        definition.validate_enum_metadata()?;
        definition.validate_default_metadata()?;
        Ok(definition)
    }
}

impl FieldDef {
    fn validate_enum_metadata(&self) -> Result<(), BuildError> {
        let Some(enum_values) = &self.constraints.enum_values else {
            return Ok(());
        };
        if enum_values.is_empty() {
            return Err(BuildError::EmptyEnumValues);
        }
        for value in enum_values {
            self.validate_present_value(value)
                .map_err(BuildError::InvalidEnumValue)?;
        }
        Ok(())
    }

    fn validate_default_metadata(&self) -> Result<(), BuildError> {
        if let DefaultMetadata::Static(value) = &self.default {
            self.validate_present_value(value)
                .map_err(BuildError::InvalidDefault)?;
        }
        Ok(())
    }
}
