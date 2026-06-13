use docnav_protocol::{StableError, StableErrorCode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocnavExitCode {
    Success = 0,
    InternalError = 1,
    InputError = 2,
    DocumentError = 3,
    AdapterOrProtocolError = 4,
}

impl DocnavExitCode {
    pub const fn code(self) -> i32 {
        self as i32
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppError {
    error: StableError,
    exit_code: DocnavExitCode,
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn new(error: StableError) -> Self {
        let exit_code = exit_code_for_error(error.code);
        Self { error, exit_code }
    }

    pub fn invalid_request(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(StableError::invalid_request(field, reason))
    }

    pub fn internal(error_id: impl Into<String>) -> Self {
        Self::new(StableError::internal_error(error_id))
    }

    pub const fn error(&self) -> &StableError {
        &self.error
    }

    pub const fn exit_code(&self) -> DocnavExitCode {
        self.exit_code
    }
}

impl From<StableError> for AppError {
    fn from(error: StableError) -> Self {
        Self::new(error)
    }
}

pub fn exit_code_for_error(code: StableErrorCode) -> DocnavExitCode {
    match code {
        StableErrorCode::InvalidRequest | StableErrorCode::CapabilityUnsupported => {
            DocnavExitCode::InputError
        }
        StableErrorCode::InternalError => DocnavExitCode::InternalError,
        StableErrorCode::AdapterUnavailable | StableErrorCode::AdapterInvokeFailed => {
            DocnavExitCode::AdapterOrProtocolError
        }
        StableErrorCode::DocumentNotFound
        | StableErrorCode::DocumentPathInvalid
        | StableErrorCode::DocumentEncodingUnsupported
        | StableErrorCode::FormatUnknown
        | StableErrorCode::FormatAmbiguous
        | StableErrorCode::RefNotFound
        | StableErrorCode::RefAmbiguous
        | StableErrorCode::RefInvalid => DocnavExitCode::DocumentError,
    }
}
