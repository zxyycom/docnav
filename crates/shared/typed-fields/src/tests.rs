#![allow(dead_code)]

use super::*;

mod field_model;
mod processing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExampleMode {
    Compact,
    Expanded,
    Detailed,
}

impl FieldStringEnum for ExampleMode {
    fn variants() -> &'static [Self] {
        &[Self::Compact, Self::Expanded, Self::Detailed]
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Expanded => "expanded",
            Self::Detailed => "detailed",
        }
    }
}

#[derive(Clone, Copy)]
struct TestProcessing(&'static str);

impl From<TestProcessing> for String {
    fn from(value: TestProcessing) -> Self {
        value.0.to_owned()
    }
}

impl From<TestProcessing> for ProcessingId {
    fn from(value: TestProcessing) -> Self {
        ProcessingId::new(value.0).expect("test processing id is valid")
    }
}

impl AsRef<str> for TestProcessing {
    fn as_ref(&self) -> &str {
        self.0
    }
}

const CONFIG_PROCESSING: TestProcessing = TestProcessing("config");

fn raw_json_processing(id: impl AsRef<str>) -> ProcessingBuild<'static, JsonValue, JsonValue> {
    ProcessingBuild::new(id, |raw| raw).expect("processing id is valid")
}

fn config_json_path<I, S>(segments: I) -> ProcessStrategy
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    ProcessStrategy::json_path(segments)
}

fn validation_failures(error: &FieldExtractionError) -> &[ValidationFailure] {
    let FieldExtractionError::Validation(errors) = error else {
        panic!("expected field validation error, got {error:?}");
    };
    errors.failures()
}
