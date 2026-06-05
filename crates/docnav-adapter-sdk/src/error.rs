use docnav_protocol::{StableError, StableErrorCode};
use std::fmt;

use crate::constants::diagnostics;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterExitCode {
    Success = 0,
    InternalError = 1,
    ProtocolError = 2,
    HandlerError = 3,
    IoError = 4,
}

impl AdapterExitCode {
    pub const fn code(self) -> i32 {
        self as i32
    }
}

pub fn exit_code_for_error(code: StableErrorCode) -> AdapterExitCode {
    match code {
        StableErrorCode::InvalidRequest
        | StableErrorCode::ProtocolIncompatible
        | StableErrorCode::CapabilityUnsupported => AdapterExitCode::ProtocolError,
        StableErrorCode::InternalError => AdapterExitCode::InternalError,
        StableErrorCode::AdapterUnavailable | StableErrorCode::AdapterInvokeFailed => {
            AdapterExitCode::IoError
        }
        StableErrorCode::DocumentNotFound
        | StableErrorCode::DocumentPathInvalid
        | StableErrorCode::DocumentEncodingUnsupported
        | StableErrorCode::FormatUnknown
        | StableErrorCode::FormatAmbiguous
        | StableErrorCode::RefNotFound
        | StableErrorCode::RefAmbiguous => AdapterExitCode::HandlerError,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterError {
    error: StableError,
    exit_code: AdapterExitCode,
}

impl AdapterError {
    pub fn new(error: StableError) -> Self {
        let exit_code = exit_code_for_error(error.code);
        Self { error, exit_code }
    }

    pub fn with_exit_code(
        error: StableError,
        exit_code: AdapterExitCode,
    ) -> Result<Self, AdapterExitCodeError> {
        if exit_code == AdapterExitCode::Success {
            Err(AdapterExitCodeError { exit_code })
        } else {
            Ok(Self { error, exit_code })
        }
    }

    pub const fn error(&self) -> &StableError {
        &self.error
    }

    pub fn into_error(self) -> StableError {
        self.error
    }

    pub const fn exit_code(&self) -> AdapterExitCode {
        self.exit_code
    }
}

impl From<StableError> for AdapterError {
    fn from(error: StableError) -> Self {
        Self::new(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdapterExitCodeError {
    exit_code: AdapterExitCode,
}

impl AdapterExitCodeError {
    pub const fn exit_code(self) -> AdapterExitCode {
        self.exit_code
    }
}

impl fmt::Display for AdapterExitCodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} {:?}",
            diagnostics::ADAPTER_ERROR_EXIT_CODE_CANNOT_BE,
            self.exit_code
        )
    }
}

impl std::error::Error for AdapterExitCodeError {}
