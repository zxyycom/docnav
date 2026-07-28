use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProcessingId(String);

impl ProcessingId {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidProcessingId> {
        let value = value.into();
        if value.trim().is_empty() {
            Err(InvalidProcessingId)
        } else {
            Ok(Self(value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ProcessingId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl TryFrom<&str> for ProcessingId {
    type Error = InvalidProcessingId;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for ProcessingId {
    type Error = InvalidProcessingId;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Display for ProcessingId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidProcessingId;

impl fmt::Display for InvalidProcessingId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("processing id is empty")
    }
}

impl std::error::Error for InvalidProcessingId {}
