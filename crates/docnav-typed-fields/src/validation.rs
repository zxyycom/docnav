use std::marker::PhantomData;

use serde_json::Value;

use crate::metadata::{FieldConstraints, ValueKind};
use crate::range::{FieldBound, FieldLength, FieldNumericRange, FieldRange};
use crate::value::{FieldStringEnum, FieldValue};

#[derive(Clone, Debug, PartialEq)]
pub struct FieldValidation<T = ()> {
    value_kind: ValueKind,
    constraints: FieldConstraints,
    typed: PhantomData<T>,
}

impl FieldValidation<String> {
    pub fn string() -> Self {
        Self::new(ValueKind::String)
    }

    pub fn regex(mut self, pattern: impl Into<String>) -> Self {
        self.constraints.regex = Some(pattern.into());
        self
    }

    pub fn length(mut self, length: FieldLength) -> Self {
        self.constraints.length_range = Some(length);
        self
    }
}

impl FieldValidation<i64> {
    pub fn int() -> Self {
        Self::new(ValueKind::Integer)
    }

    pub fn min(mut self, minimum: FieldBound<i64>) -> Self {
        let mut range = self.integer_range();
        range.minimum = Some(minimum);
        self.constraints.numeric_range = FieldNumericRange::Integer(range);
        self
    }

    pub fn max(mut self, maximum: FieldBound<i64>) -> Self {
        let mut range = self.integer_range();
        range.maximum = Some(maximum);
        self.constraints.numeric_range = FieldNumericRange::Integer(range);
        self
    }

    pub fn between(mut self, minimum: FieldBound<i64>, maximum: FieldBound<i64>) -> Self {
        self.constraints.numeric_range =
            FieldNumericRange::Integer(FieldRange::between(minimum, maximum));
        self
    }

    fn integer_range(&self) -> FieldRange<i64> {
        match self.constraints.numeric_range {
            FieldNumericRange::Integer(range) => range,
            FieldNumericRange::None => FieldRange::default(),
            FieldNumericRange::Number(_) => {
                unreachable!("integer validation cannot contain number range")
            }
        }
    }
}

impl FieldValidation<f64> {
    pub fn num() -> Self {
        Self::new(ValueKind::Number)
    }

    pub fn min(mut self, minimum: FieldBound<f64>) -> Self {
        let mut range = self.number_range();
        range.minimum = Some(minimum);
        self.constraints.numeric_range = FieldNumericRange::Number(range);
        self
    }

    pub fn max(mut self, maximum: FieldBound<f64>) -> Self {
        let mut range = self.number_range();
        range.maximum = Some(maximum);
        self.constraints.numeric_range = FieldNumericRange::Number(range);
        self
    }

    pub fn between(mut self, minimum: FieldBound<f64>, maximum: FieldBound<f64>) -> Self {
        self.constraints.numeric_range =
            FieldNumericRange::Number(FieldRange::between(minimum, maximum));
        self
    }

    fn number_range(&self) -> FieldRange<f64> {
        match self.constraints.numeric_range {
            FieldNumericRange::Number(range) => range,
            FieldNumericRange::None => FieldRange::default(),
            FieldNumericRange::Integer(_) => {
                unreachable!("number validation cannot contain integer range")
            }
        }
    }
}

impl FieldValidation<()> {
    pub fn string_enum<T: FieldStringEnum>() -> FieldValidation<T> {
        let mut validation = FieldValidation::<T>::new(ValueKind::String);
        let mut enum_values = Vec::new();
        for value in T::variants()
            .iter()
            .cloned()
            .map(FieldValue::into_json_value)
        {
            if !enum_values.contains(&value) {
                enum_values.push(value);
            }
        }
        validation.constraints.enum_values = Some(enum_values);
        validation
    }
}

impl FieldValidation<bool> {
    pub fn boolean() -> Self {
        Self::new(ValueKind::Boolean)
    }
}

impl FieldValidation<Vec<Value>> {
    pub fn array() -> Self {
        Self::new(ValueKind::Array)
    }

    pub fn length(mut self, length: FieldLength) -> Self {
        self.constraints.length_range = Some(length);
        self
    }
}

impl FieldValidation<serde_json::Map<String, Value>> {
    pub fn object() -> Self {
        Self::new(ValueKind::Object)
    }
}

impl<T> FieldValidation<T> {
    fn new(value_kind: ValueKind) -> Self {
        Self {
            value_kind,
            constraints: FieldConstraints::default(),
            typed: PhantomData,
        }
    }

    pub(crate) fn into_parts(self) -> (ValueKind, FieldConstraints) {
        (self.value_kind, self.constraints)
    }
}
