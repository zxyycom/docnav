use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

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

#[derive(Clone)]
pub struct ProcessingBuild<'a, I, O> {
    id: ProcessingId,
    process: Arc<dyn Fn(I) -> O + 'a>,
    typed: PhantomData<fn(I) -> O>,
}

impl<'a, I, O> ProcessingBuild<'a, I, O> {
    pub fn new<F>(id: impl AsRef<str>, process: F) -> Result<Self, InvalidProcessingId>
    where
        F: Fn(I) -> O + 'a,
    {
        Ok(Self {
            id: ProcessingId::new(id.as_ref())?,
            process: Arc::new(process),
            typed: PhantomData,
        })
    }

    pub fn id(&self) -> &ProcessingId {
        &self.id
    }

    pub fn process(&self, raw_input: I) -> ProcessedValue<O> {
        ProcessedValue::new(self.id.clone(), (self.process)(raw_input))
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
    pub(crate) fn new(processing_id: ProcessingId, value: T) -> Self {
        Self {
            processing_id,
            value,
        }
    }

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

#[derive(Clone, Debug, PartialEq)]
pub struct ProcessedExtraction<E, O> {
    extraction: E,
    processing: ProcessedValue<O>,
}

impl<E, O> ProcessedExtraction<E, O> {
    pub fn new(extraction: E, processing: ProcessedValue<O>) -> Self {
        Self {
            extraction,
            processing,
        }
    }

    pub fn extraction(&self) -> &E {
        &self.extraction
    }

    pub fn processing(&self) -> &ProcessedValue<O> {
        &self.processing
    }

    pub fn into_parts(self) -> (E, ProcessedValue<O>) {
        (self.extraction, self.processing)
    }
}
