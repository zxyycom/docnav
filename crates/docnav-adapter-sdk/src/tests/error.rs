use crate::{exit_code_for_diagnostic, AdapterError, AdapterExitCode};
use docnav_diagnostics::{
    typed_codes, CliArgvDetails, DiagnosticRecordDraft, DiagnosticSource, ProtocolDiagnosticCode,
};

// @case WB-SDK-ERROR-001
#[test]
fn internal_error_maps_to_internal_exit_code() {
    assert_eq!(AdapterExitCode::InternalError.code(), 1);
    assert_eq!(
        exit_code_for_diagnostic(ProtocolDiagnosticCode::InternalError),
        AdapterExitCode::InternalError
    );
    assert_eq!(
        AdapterError::internal("test").exit_code(),
        AdapterExitCode::InternalError
    );
}

#[test]
fn diagnostic_codes_map_to_adapter_exit_codes() {
    let cases = [
        (
            ProtocolDiagnosticCode::InvalidRequest,
            AdapterExitCode::ProtocolError,
        ),
        (
            ProtocolDiagnosticCode::DocumentNotFound,
            AdapterExitCode::HandlerError,
        ),
        (
            ProtocolDiagnosticCode::DocumentPathInvalid,
            AdapterExitCode::HandlerError,
        ),
        (
            ProtocolDiagnosticCode::DocumentEncodingUnsupported,
            AdapterExitCode::HandlerError,
        ),
        (
            ProtocolDiagnosticCode::FormatUnknown,
            AdapterExitCode::HandlerError,
        ),
        (
            ProtocolDiagnosticCode::FormatAmbiguous,
            AdapterExitCode::HandlerError,
        ),
        (
            ProtocolDiagnosticCode::CapabilityUnsupported,
            AdapterExitCode::ProtocolError,
        ),
        (
            ProtocolDiagnosticCode::RefNotFound,
            AdapterExitCode::HandlerError,
        ),
        (
            ProtocolDiagnosticCode::RefAmbiguous,
            AdapterExitCode::HandlerError,
        ),
        (
            ProtocolDiagnosticCode::RefInvalid,
            AdapterExitCode::HandlerError,
        ),
        (
            ProtocolDiagnosticCode::AdapterUnavailable,
            AdapterExitCode::IoError,
        ),
        (
            ProtocolDiagnosticCode::AdapterInvokeFailed,
            AdapterExitCode::IoError,
        ),
        (
            ProtocolDiagnosticCode::InternalError,
            AdapterExitCode::InternalError,
        ),
    ];

    for (code, expected) in cases {
        assert_eq!(exit_code_for_diagnostic(code), expected, "{code:?}");
    }
}

#[test]
fn adapter_error_rejects_success_exit_code() {
    let error = AdapterError::with_exit_code(
        AdapterError::ref_not_found("missing").diagnostic().clone(),
        AdapterExitCode::Success,
    )
    .expect_err("failure cannot use success exit code");

    assert_eq!(error.exit_code(), AdapterExitCode::Success);
}

#[test]
fn adapter_error_normalizes_non_protocol_diagnostic_for_protocol_projection() {
    let warning_draft = DiagnosticRecordDraft::new::<typed_codes::readable_warning::CliArgvIgnored>(
        "ignored adapter argv",
        CliArgvDetails::new(vec!["--unused".into()]),
        DiagnosticSource::with_stage("test", "adapter"),
    );

    let error = AdapterError::new(warning_draft);
    let protocol_error = error.protocol_error();

    assert_eq!(error.exit_code(), AdapterExitCode::InternalError);
    assert_eq!(protocol_error.code(), ProtocolDiagnosticCode::InternalError);
    assert_eq!(
        protocol_error.details()["error_id"].as_str(),
        Some("adapter-error-diagnostic-not-protocol")
    );
}
