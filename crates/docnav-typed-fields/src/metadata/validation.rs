use serde_json::Value;

use crate::field::constraints::{
    above_maximum, actual_value_kind, below_minimum, numeric_range_violation, value_length,
    NumericRangeViolation,
};

use super::{
    ActualValueKind, DefaultMetadata, SchemaMetadataView, TypedValue, ValidationFailure,
    ValidationReason, ValueKind,
};

impl SchemaMetadataView {
    pub fn validate_optional_value(
        &self,
        value: Option<&Value>,
    ) -> Result<Option<TypedValue>, ValidationFailure> {
        let Some(value) = value else {
            return if self.constraints.required {
                Err(self.failure(ValidationReason::MissingRequired))
            } else {
                Ok(None)
            };
        };
        if value.is_null() && !self.constraints.required && self.value_kind != ValueKind::Json {
            return Ok(None);
        }
        self.validate_value(value).map(Some)
    }

    pub fn validate_optional_value_with_static_default(
        &self,
        value: Option<&Value>,
    ) -> Result<Option<TypedValue>, ValidationFailure> {
        match value {
            Some(value) => self.validate_optional_value(Some(value)),
            None => match self.static_default_value()? {
                Some(value) => Ok(Some(value)),
                None if self.constraints.required => {
                    Err(self.failure(ValidationReason::MissingRequired))
                }
                None => Ok(None),
            },
        }
    }

    pub fn validate_value(&self, value: &Value) -> Result<TypedValue, ValidationFailure> {
        let typed = self.decode_value(value)?;
        if matches!(typed, TypedValue::Null) {
            return Ok(typed);
        }
        self.validate_enum_value(value)?;
        self.validate_regex(value)?;
        self.validate_numeric_constraints(&typed)?;
        self.validate_length_constraints(&typed)?;
        self.validate_unique_items(&typed)?;
        Ok(typed)
    }

    pub fn static_default_value(&self) -> Result<Option<TypedValue>, ValidationFailure> {
        match &self.default {
            DefaultMetadata::None => Ok(None),
            DefaultMetadata::Static(value) => self.validate_value(value).map(Some),
        }
    }

    pub fn is_required(&self) -> bool {
        self.constraints.required
    }

    fn decode_value(&self, value: &Value) -> Result<TypedValue, ValidationFailure> {
        if value.is_null() && self.value_kind == ValueKind::Json {
            return Ok(TypedValue::Json(Value::Null));
        }
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
            (ValueKind::Json, value) => Ok(TypedValue::Json(value.clone())),
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

    fn validate_regex(&self, value: &Value) -> Result<(), ValidationFailure> {
        let Some(pattern) = &self.constraints.regex else {
            return Ok(());
        };
        let Value::String(value) = value else {
            return Ok(());
        };
        let Ok(regex) = regex::Regex::new(pattern) else {
            return Err(self.failure(ValidationReason::RegexMismatch {
                pattern: pattern.clone(),
            }));
        };
        if regex.is_match(value) {
            Ok(())
        } else {
            Err(self.failure(ValidationReason::RegexMismatch {
                pattern: pattern.clone(),
            }))
        }
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

    fn validate_unique_items(&self, value: &TypedValue) -> Result<(), ValidationFailure> {
        if !self.constraints.unique_items {
            return Ok(());
        }
        let TypedValue::Array(items) = value else {
            return Ok(());
        };
        for (duplicate_index, item) in items.iter().enumerate() {
            if let Some(first_index) = items[..duplicate_index]
                .iter()
                .position(|previous| previous == item)
            {
                return Err(self.failure(ValidationReason::DuplicateArrayItem {
                    first_index,
                    duplicate_index,
                }));
            }
        }
        Ok(())
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
