use crate::{exit_code_for_error, AdapterError, AdapterExitCode};
use docnav_protocol::{StableError, StableErrorCode};

#[test]
fn internal_error_maps_to_internal_exit_code() {
    assert_eq!(AdapterExitCode::InternalError.code(), 1);
    assert_eq!(
        exit_code_for_error(StableErrorCode::InternalError),
        AdapterExitCode::InternalError
    );
    assert_eq!(
        AdapterError::new(StableError::internal_error("test")).exit_code(),
        AdapterExitCode::InternalError
    );
}

#[test]
fn stable_error_codes_map_to_adapter_exit_codes() {
    let cases = [
        (
            StableErrorCode::InvalidRequest,
            AdapterExitCode::ProtocolError,
        ),
        (
            StableErrorCode::DocumentNotFound,
            AdapterExitCode::HandlerError,
        ),
        (
            StableErrorCode::DocumentPathInvalid,
            AdapterExitCode::HandlerError,
        ),
        (
            StableErrorCode::DocumentEncodingUnsupported,
            AdapterExitCode::HandlerError,
        ),
        (
            StableErrorCode::FormatUnknown,
            AdapterExitCode::HandlerError,
        ),
        (
            StableErrorCode::FormatAmbiguous,
            AdapterExitCode::HandlerError,
        ),
        (
            StableErrorCode::CapabilityUnsupported,
            AdapterExitCode::ProtocolError,
        ),
        (StableErrorCode::RefNotFound, AdapterExitCode::HandlerError),
        (StableErrorCode::RefAmbiguous, AdapterExitCode::HandlerError),
        (StableErrorCode::RefInvalid, AdapterExitCode::HandlerError),
        (
            StableErrorCode::AdapterUnavailable,
            AdapterExitCode::IoError,
        ),
        (
            StableErrorCode::AdapterInvokeFailed,
            AdapterExitCode::IoError,
        ),
        (
            StableErrorCode::InternalError,
            AdapterExitCode::InternalError,
        ),
    ];

    for (code, expected) in cases {
        assert_eq!(exit_code_for_error(code), expected, "{code:?}");
    }
}

#[test]
fn adapter_error_rejects_success_exit_code() {
    let error = AdapterError::with_exit_code(
        StableError::ref_not_found("missing"),
        AdapterExitCode::Success,
    )
    .expect_err("failure cannot use success exit code");

    assert_eq!(error.exit_code(), AdapterExitCode::Success);
}
