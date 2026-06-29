pub type PositiveInteger = std::num::NonZeroU32;
pub type Options = serde_json::Map<String, serde_json::Value>;
pub type ErrorDetails = std::collections::BTreeMap<String, serde_json::Value>;

pub use docnav_diagnostics::ProtocolDiagnosticCode;

mod constants;
mod contract_validation;
mod decode;
mod envelope;
mod error;
mod manifest;
mod operation;
mod operation_result;
mod positive_integer;
mod probe;
mod request_context;
mod request_id;
mod schema;
mod version;

pub use constants::{MANIFEST_VERSION, PROBE_VERSION, PROTOCOL_VERSION, UNKNOWN_REQUEST_ID};
pub use decode::{
    decode_manifest_value, decode_probe_result_value, decode_protocol_request_value,
    decode_protocol_response_value, decode_value, DecodePipelineError, DecodePipelineStage,
    ProtocolRequestDecodeError,
};
pub use envelope::{
    Document, FailureResponse, FindArguments, InfoArguments, OperationArguments, OutlineArguments,
    ProtocolResponse, ProtocolValidationError, RawRequestEnvelope, ReadArguments, RequestEnvelope,
    SuccessResponse,
};
pub use error::{
    protocol_error_category, protocol_error_default_message, protocol_error_record_draft,
    protocol_error_record_draft_with_summary, InvalidErrorDetail, ProtocolError,
    ProtocolErrorCategory,
};
pub use manifest::{AdapterIdentity, FormatDescriptor, Manifest, ManifestValidationError};
pub use operation::{Operation, OperationParseError, PagedOperation};
pub use operation_result::{
    Cost, Entry, FindResult, InfoAdapter, InfoDocument, InfoResult, Location, Measurement,
    OperationResult, OutlineResult, ReadResult,
};
pub use positive_integer::{positive_result, try_positive, PositiveIntegerError};
pub use probe::{ProbeReason, ProbeReasonCode, ProbeResult, ProbeValidationError};
pub use request_context::{
    extract_request_context, extract_request_context_from_value, PartialRequestContext,
};
pub use request_id::{generate_request_id, GENERATED_REQUEST_ID_PREFIX};
pub use schema::{
    validate_manifest_value, validate_probe_result_value, validate_protocol_request_value,
    validate_protocol_response_value, SchemaValidationError,
};
pub use version::{ProtocolVersion, VersionParseError};

#[cfg(test)]
mod tests;
