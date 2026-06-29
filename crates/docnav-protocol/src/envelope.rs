use serde::{Deserialize, Serialize};
use std::fmt;

use crate::constants::fields;
use crate::{
    InvalidErrorDetail, Operation, OperationResult, Options, PositiveInteger, ProtocolError,
    PROTOCOL_VERSION, UNKNOWN_REQUEST_ID,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub path: String,
}

/// Protocol request after an entry point has mapped raw `arguments` through
/// standard-parameter defaults, validation, and passthrough policy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestEnvelope {
    pub protocol_version: String,
    pub request_id: String,
    pub operation: Operation,
    pub document: Document,
    pub arguments: OperationArguments,
}

impl RequestEnvelope {
    pub fn operation_arguments(&self) -> Result<&OperationArguments, ProtocolError> {
        if self.arguments.operation() == self.operation {
            Ok(&self.arguments)
        } else {
            Err(ProtocolError::invalid_request(
                fields::ARGUMENTS,
                format!("arguments do not match operation {}", self.operation),
            ))
        }
    }
}

/// Schema-decoded protocol request that preserves raw `arguments` for the
/// entry-point standard-parameter pipeline.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRequestEnvelope {
    pub protocol_version: String,
    pub request_id: String,
    pub operation: Operation,
    pub document: Document,
    pub arguments: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OperationArguments {
    Outline(OutlineArguments),
    Read(ReadArguments),
    Find(FindArguments),
    Info(InfoArguments),
}

impl OperationArguments {
    pub const fn operation(&self) -> Operation {
        match self {
            Self::Outline(_) => Operation::Outline,
            Self::Read(_) => Operation::Read,
            Self::Find(_) => Operation::Find,
            Self::Info(_) => Operation::Info,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutlineArguments {
    pub limit_chars: PositiveInteger,
    pub page: PositiveInteger,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Options>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadArguments {
    #[serde(rename = "ref")]
    pub ref_id: String,
    pub limit_chars: PositiveInteger,
    pub page: PositiveInteger,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Options>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindArguments {
    pub query: String,
    pub limit_chars: PositiveInteger,
    pub page: PositiveInteger,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Options>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoArguments {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Options>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProtocolResponse {
    Success(SuccessResponse),
    Failure(FailureResponse),
}

impl ProtocolResponse {
    pub fn success(
        protocol_version: impl Into<String>,
        request_id: impl Into<String>,
        result: OperationResult,
    ) -> Self {
        Self::Success(SuccessResponse::new(protocol_version, request_id, result))
    }

    pub fn failure(
        protocol_version: impl Into<String>,
        request_id: impl Into<String>,
        operation: Option<Operation>,
        error: ProtocolError,
    ) -> Self {
        Self::Failure(FailureResponse::new(
            protocol_version,
            request_id,
            operation,
            error,
        ))
    }

    pub fn failure_for_request(request: &RequestEnvelope, error: ProtocolError) -> Self {
        Self::failure(
            request.protocol_version.clone(),
            request.request_id.clone(),
            Some(request.operation),
            error,
        )
    }

    pub fn validate(&self) -> Result<(), ProtocolValidationError> {
        match self {
            Self::Success(response) => response.validate(),
            Self::Failure(response) => response.validate(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessResponse {
    pub protocol_version: String,
    pub request_id: String,
    pub operation: Operation,
    pub ok: bool,
    pub result: OperationResult,
}

impl SuccessResponse {
    pub fn new(
        protocol_version: impl Into<String>,
        request_id: impl Into<String>,
        result: OperationResult,
    ) -> Self {
        Self {
            protocol_version: protocol_version.into(),
            request_id: request_id.into(),
            operation: result.operation(),
            ok: true,
            result,
        }
    }

    pub fn validate(&self) -> Result<(), ProtocolValidationError> {
        if !self.ok {
            return Err(ProtocolValidationError::InvalidOkFlag);
        }
        if self.operation != self.result.operation() {
            return Err(ProtocolValidationError::ResultOperationMismatch {
                operation: self.operation,
                result_operation: self.result.operation(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureResponse {
    pub protocol_version: String,
    pub request_id: String,
    pub operation: Option<Operation>,
    pub ok: bool,
    pub error: ProtocolError,
}

impl FailureResponse {
    pub fn new(
        protocol_version: impl Into<String>,
        request_id: impl Into<String>,
        operation: Option<Operation>,
        error: ProtocolError,
    ) -> Self {
        Self {
            protocol_version: protocol_version.into(),
            request_id: request_id.into(),
            operation,
            ok: false,
            error,
        }
    }

    pub fn unparsed(error: ProtocolError) -> Self {
        Self::new(PROTOCOL_VERSION, UNKNOWN_REQUEST_ID, None, error)
    }

    pub fn validate(&self) -> Result<(), ProtocolValidationError> {
        if self.ok {
            return Err(ProtocolValidationError::InvalidOkFlag);
        }
        self.error
            .validate_details()
            .map_err(ProtocolValidationError::InvalidErrorDetail)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolValidationError {
    InvalidOkFlag,
    ResultOperationMismatch {
        operation: Operation,
        result_operation: Operation,
    },
    InvalidErrorDetail(InvalidErrorDetail),
}

impl fmt::Display for ProtocolValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOkFlag => formatter.write_str("response ok flag does not match variant"),
            Self::ResultOperationMismatch {
                operation,
                result_operation,
            } => write!(
                formatter,
                "success response operation {operation} does not match result {result_operation}"
            ),
            Self::InvalidErrorDetail(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ProtocolValidationError {}
