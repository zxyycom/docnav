use serde_json::Value;

use crate::metadata::{ActualValueKind, BuildError, TypedValue, ValueKind};

pub trait FieldStringEnum: Clone + Sized + 'static {
    fn variants() -> &'static [Self];
    fn as_str(&self) -> &'static str;
}

pub trait FieldValue: Sized {
    fn value_kind() -> ValueKind;
    fn into_json_value(self) -> Value;
    fn try_into_json_value(self) -> Result<Value, BuildError> {
        Ok(self.into_json_value())
    }
    fn from_typed_value(value: TypedValue) -> Result<Self, FieldValueError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldValueError {
    pub expected: ValueKind,
    pub actual: ActualValueKind,
}

impl FieldValue for String {
    fn value_kind() -> ValueKind {
        ValueKind::String
    }

    fn into_json_value(self) -> Value {
        Value::String(self)
    }

    fn from_typed_value(value: TypedValue) -> Result<Self, FieldValueError> {
        match value {
            TypedValue::String(value) => Ok(value),
            value => Err(FieldValueError {
                expected: Self::value_kind(),
                actual: value.actual_kind(),
            }),
        }
    }
}

impl FieldValue for i64 {
    fn value_kind() -> ValueKind {
        ValueKind::Integer
    }

    fn into_json_value(self) -> Value {
        Value::from(self)
    }

    fn from_typed_value(value: TypedValue) -> Result<Self, FieldValueError> {
        match value {
            TypedValue::Integer(value) => Ok(value),
            value => Err(FieldValueError {
                expected: Self::value_kind(),
                actual: value.actual_kind(),
            }),
        }
    }
}

impl FieldValue for f64 {
    fn value_kind() -> ValueKind {
        ValueKind::Number
    }

    fn into_json_value(self) -> Value {
        Value::from(self)
    }

    fn try_into_json_value(self) -> Result<Value, BuildError> {
        if self.is_finite() {
            Ok(Value::from(self))
        } else {
            Err(BuildError::NonFiniteDefaultValue)
        }
    }

    fn from_typed_value(value: TypedValue) -> Result<Self, FieldValueError> {
        match value {
            TypedValue::Number(value) => Ok(value),
            value => Err(FieldValueError {
                expected: Self::value_kind(),
                actual: value.actual_kind(),
            }),
        }
    }
}

impl FieldValue for bool {
    fn value_kind() -> ValueKind {
        ValueKind::Boolean
    }

    fn into_json_value(self) -> Value {
        Value::Bool(self)
    }

    fn from_typed_value(value: TypedValue) -> Result<Self, FieldValueError> {
        match value {
            TypedValue::Boolean(value) => Ok(value),
            value => Err(FieldValueError {
                expected: Self::value_kind(),
                actual: value.actual_kind(),
            }),
        }
    }
}

impl FieldValue for Vec<Value> {
    fn value_kind() -> ValueKind {
        ValueKind::Array
    }

    fn into_json_value(self) -> Value {
        Value::Array(self)
    }

    fn from_typed_value(value: TypedValue) -> Result<Self, FieldValueError> {
        match value {
            TypedValue::Array(value) => Ok(value),
            value => Err(FieldValueError {
                expected: Self::value_kind(),
                actual: value.actual_kind(),
            }),
        }
    }
}

impl FieldValue for serde_json::Map<String, Value> {
    fn value_kind() -> ValueKind {
        ValueKind::Object
    }

    fn into_json_value(self) -> Value {
        Value::Object(self)
    }

    fn from_typed_value(value: TypedValue) -> Result<Self, FieldValueError> {
        match value {
            TypedValue::Object(value) => Ok(value),
            value => Err(FieldValueError {
                expected: Self::value_kind(),
                actual: value.actual_kind(),
            }),
        }
    }
}

impl FieldValue for Value {
    fn value_kind() -> ValueKind {
        ValueKind::Json
    }

    fn into_json_value(self) -> Value {
        self
    }

    fn from_typed_value(value: TypedValue) -> Result<Self, FieldValueError> {
        match value {
            TypedValue::Json(value) => Ok(value),
            value => Err(FieldValueError {
                expected: Self::value_kind(),
                actual: value.actual_kind(),
            }),
        }
    }
}

impl<T: FieldStringEnum> FieldValue for T {
    fn value_kind() -> ValueKind {
        ValueKind::String
    }

    fn into_json_value(self) -> Value {
        Value::String(self.as_str().to_string())
    }

    fn from_typed_value(value: TypedValue) -> Result<Self, FieldValueError> {
        match value {
            TypedValue::String(value) => T::variants()
                .iter()
                .find(|variant| variant.as_str() == value)
                .cloned()
                .ok_or(FieldValueError {
                    expected: Self::value_kind(),
                    actual: ActualValueKind::String,
                }),
            value => Err(FieldValueError {
                expected: Self::value_kind(),
                actual: value.actual_kind(),
            }),
        }
    }
}

impl<T: FieldValue> FieldValue for Option<T> {
    fn value_kind() -> ValueKind {
        T::value_kind()
    }

    fn into_json_value(self) -> Value {
        self.map_or(Value::Null, T::into_json_value)
    }

    fn try_into_json_value(self) -> Result<Value, BuildError> {
        self.map_or(Ok(Value::Null), T::try_into_json_value)
    }

    fn from_typed_value(value: TypedValue) -> Result<Self, FieldValueError> {
        match value {
            TypedValue::Null => Ok(None),
            value => T::from_typed_value(value).map(Some),
        }
    }
}

impl TypedValue {
    pub fn to_json_value(&self) -> Value {
        match self {
            Self::String(value) => Value::String(value.clone()),
            Self::Integer(value) => Value::from(*value),
            Self::Number(value) => Value::from(*value),
            Self::Boolean(value) => Value::Bool(*value),
            Self::Array(value) => Value::Array(value.clone()),
            Self::Object(value) => Value::Object(value.clone()),
            Self::Json(value) => value.clone(),
            Self::Null => Value::Null,
        }
    }

    pub fn actual_kind(&self) -> ActualValueKind {
        match self {
            Self::String(_) => ActualValueKind::String,
            Self::Integer(_) => ActualValueKind::Integer,
            Self::Number(_) => ActualValueKind::Number,
            Self::Boolean(_) => ActualValueKind::Boolean,
            Self::Array(_) => ActualValueKind::Array,
            Self::Object(_) => ActualValueKind::Object,
            Self::Json(value) => crate::field::constraints::actual_value_kind(value),
            Self::Null => ActualValueKind::Null,
        }
    }
}
