mod field;
pub mod json;
mod metadata;
mod process_strategy;
mod processing;
mod range;
mod set;
mod validation;
mod value;

pub use field::{FieldDef, FieldDefBuilder};
pub use json::JsonFieldSet;
pub use metadata::{
    ActualValueKind, BuildError, DefaultMetadata, FieldConstraints, FieldDuplicateIdentityError,
    FieldIdentity, FieldPath, MergeStrategy, ProcessingMetadataView, SchemaMetadataView,
    TypedValue, ValidationFailure, ValidationReason, ValueKind,
};
pub use process_strategy::{
    CliBooleanEncoding, CliProcessingMetadata, ProcessStrategy, ProcessingInputKind,
    ProcessingLocator,
};
pub use processing::{
    InvalidProcessingId, ProcessedExtraction, ProcessedValue, ProcessingBuild, ProcessingId,
};
pub use range::{
    FieldBound, FieldBoundKind, FieldLength, FieldNumericBound, FieldNumericRange, FieldRange,
};
pub use serde_json::Value as JsonValue;
pub use set::{
    ExpectedFieldShape, FieldDefBuildFailure, FieldDefSet, FieldDefSetBuildError,
    FieldDefSetBuilder, FieldDuplicateProcessingLocatorError, FieldDuplicateProcessingPathError,
    FieldExtractionError, FieldValidationErrors, FieldValueMap,
};
pub use validation::FieldValidation;
pub use value::{FieldStringEnum, FieldValue, FieldValueError};

pub type JsonPassthroughProcessing<'a> = ProcessingBuild<'a, JsonValue, JsonValue>;

#[cfg(test)]
mod tests;
