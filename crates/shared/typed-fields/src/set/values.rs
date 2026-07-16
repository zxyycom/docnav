use std::collections::BTreeMap;
use std::fmt;

use crate::metadata::{FieldIdentity, TypedValue};
use crate::value::FieldValue;
use crate::ValidationFailure;

pub type FieldValueMap = BTreeMap<FieldIdentity, TypedValue>;

#[derive(Clone, Debug, PartialEq)]
pub struct FieldValidationErrors {
    failures: Vec<ValidationFailure>,
}

impl FieldValidationErrors {
    pub fn new(failures: Vec<ValidationFailure>) -> Self {
        Self { failures }
    }

    pub fn failures(&self) -> &[ValidationFailure] {
        &self.failures
    }

    pub fn into_failures(self) -> Vec<ValidationFailure> {
        self.failures
    }
}

impl fmt::Display for FieldValidationErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} field validation error(s)",
            self.failures.len()
        )
    }
}

impl std::error::Error for FieldValidationErrors {}

#[derive(Debug, PartialEq)]
#[doc(hidden)]
pub struct FieldValues {
    pub(crate) values: Vec<Option<TypedValue>>,
}

impl FieldValues {
    #[doc(hidden)]
    pub fn __typed_optional_slot<T: FieldValue>(&self, slot: usize) -> Option<T> {
        let value = self
            .values
            .get(slot)
            .expect("generated field definition slot is present");
        value.clone().map(|value| {
            T::from_typed_value(value).expect("generated field definition type is consistent")
        })
    }

    #[doc(hidden)]
    pub fn __typed_required_slot<T: FieldValue>(&self, slot: usize) -> T {
        self.__typed_optional_slot(slot)
            .expect("required field was extracted during validation")
    }
}
