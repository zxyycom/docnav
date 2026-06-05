use std::fmt;
use std::num::NonZeroU32;

use crate::PositiveInteger;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PositiveIntegerError {
    value: u32,
}

impl PositiveIntegerError {
    pub const fn value(self) -> u32 {
        self.value
    }
}

impl fmt::Display for PositiveIntegerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "positive integer must be non-zero, got {}",
            self.value
        )
    }
}

impl std::error::Error for PositiveIntegerError {}

pub fn try_positive(value: u32) -> Option<PositiveInteger> {
    NonZeroU32::new(value)
}

pub fn positive_result(value: u32) -> Result<PositiveInteger, PositiveIntegerError> {
    try_positive(value).ok_or(PositiveIntegerError { value })
}
