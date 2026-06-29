#![allow(dead_code)]

use super::*;

mod constraints;
mod field_metadata;
mod field_model;
mod field_presence;
mod field_ranges;
mod processing;
mod set_projection;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputMode {
    ReadableView,
    ReadableJson,
    ProtocolJson,
}

impl FieldStringEnum for OutputMode {
    fn variants() -> &'static [Self] {
        &[Self::ReadableView, Self::ReadableJson, Self::ProtocolJson]
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::ReadableView => "readable-view",
            Self::ReadableJson => "readable-json",
            Self::ProtocolJson => "protocol-json",
        }
    }
}

const CONFIG_PROCESSING: &str = "config";

fn raw_json_processing(
    id: impl Into<ProcessingId>,
) -> ProcessingBuild<'static, JsonValue, JsonValue> {
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
