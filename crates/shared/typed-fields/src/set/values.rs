use std::collections::BTreeMap;
use std::fmt;

use crate::metadata::{FieldIdentity, TypedValue};
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
