pub type PositiveInteger = std::num::NonZeroU32;
pub type Options = serde_json::Map<String, serde_json::Value>;
pub type ErrorDetails = std::collections::BTreeMap<String, serde_json::Value>;

mod constants;
mod context;
mod envelope;
mod error;
mod generated;
mod integer;
mod manifest;
mod operation;
mod probe;
mod result;
mod schema;
mod version;

pub use constants::{MANIFEST_VERSION, PROBE_VERSION, PROTOCOL_VERSION, UNKNOWN_REQUEST_ID};
pub use context::{
    extract_request_context, extract_request_context_from_value, PartialRequestContext,
};
pub use envelope::{
    Document, FailureResponse, FindArguments, InfoArguments, OperationArguments, OutlineArguments,
    ProtocolResponse, ProtocolValidationError, ReadArguments, RequestEnvelope, SuccessResponse,
};
pub use error::{MissingErrorDetail, StableError, StableErrorCode};
pub use integer::{positive_result, try_positive, PositiveIntegerError};
pub use manifest::{
    AdapterIdentity, FormatDescriptor, Manifest, ManifestValidationError, RecommendedParameters,
};
pub use operation::{Operation, OperationParseError, PagedOperation};
pub use probe::{ProbeReason, ProbeReasonCode, ProbeResult, ProbeValidationError};
pub use result::{Entry, FindResult, InfoResult, OperationResult, OutlineResult, ReadResult};
pub use schema::{
    validate_manifest_value, validate_probe_result_value, validate_protocol_request_value,
    validate_protocol_response_value, SchemaValidationError,
};
pub use version::{
    ensure_supported_protocol, select_highest_compatible, ProtocolRange, ProtocolRangeError,
    ProtocolVersion, VersionParseError,
};

#[cfg(test)]
mod tests;
