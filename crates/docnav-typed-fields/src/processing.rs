use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProcessingId(String);

impl ProcessingId {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidProcessingId> {
        Self(value.into()).validate()
    }

    fn validate(self) -> Result<Self, InvalidProcessingId> {
        if self.0.trim().is_empty() {
            Err(InvalidProcessingId)
        } else {
            Ok(self)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ProcessingId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for ProcessingId {
    fn from(value: String) -> Self {
        Self(value)
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

#[derive(Clone)]
pub struct ProcessingBuild<'a, I, O> {
    id: ProcessingId,
    process: Arc<dyn Fn(I) -> O + 'a>,
    typed: PhantomData<fn(I) -> O>,
}

impl<'a, I, O> ProcessingBuild<'a, I, O> {
    pub fn new<F>(id: impl Into<ProcessingId>, process: F) -> Result<Self, InvalidProcessingId>
    where
        F: Fn(I) -> O + 'a,
    {
        Ok(Self {
            id: id.into().validate()?,
            process: Arc::new(process),
            typed: PhantomData,
        })
    }

    pub fn id(&self) -> &ProcessingId {
        &self.id
    }

    pub fn process(&self, raw_input: I) -> ProcessedValue<O> {
        ProcessedValue {
            processing_id: self.id.clone(),
            value: (self.process)(raw_input),
        }
    }
}

impl<I, O> fmt::Debug for ProcessingBuild<'_, I, O> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProcessingBuild")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProcessedValue<T> {
    processing_id: ProcessingId,
    value: T,
}

impl<T> ProcessedValue<T> {
    pub fn processing_id(&self) -> &ProcessingId {
        &self.processing_id
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn into_value(self) -> T {
        self.value
    }
}
