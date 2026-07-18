// @case WB-CORE-OUTPUT-001
use super::*;
use docnav_diagnostics::{
    typed_codes, BoundaryDetails, DiagnosticRecordDraft, DiagnosticSource, RefDetails,
};
use docnav_navigation::{
    NavigationCommandOutcome, NavigationInvocationTrace, NavigationOutputMode,
};
use docnav_protocol::{
    protocol_error_record_draft_with_summary, Cost, Measurement, OperationResult, OutlineResult,
    ProtocolResponse, ReadResult, UnstructuredOutlineReason,
};
use serde_json::{json, Value};

fn write_success(outcome: CommandOutcome) -> (Vec<u8>, Vec<u8>) {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = write_outcome(outcome, &mut stdout, &mut stderr);
    assert_eq!(exit, 0);
    (stdout, stderr)
}

#[test]
fn plain_text_outcome_writes_text_directly() {
    let outcome = CommandOutcome::plain_text("hello world");
    let (stdout, _) = write_success(outcome);
    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("hello world"));
    assert!(!output.trim().starts_with('{'));
}

#[test]
fn non_document_json_writes_value_directly() {
    let outcome = CommandOutcome::json(json!({"config": "ok"}));
    let (stdout, stderr) = write_success(outcome);
    assert!(stderr.is_empty());
    let value: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(value["config"], "ok");
}

#[test]
fn document_readable_view_uses_shared_output_facade() {
    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "request-1",
        OperationResult::Read(ReadResult {
            ref_id: "R1".into(),
            content: "body".into(),
            content_type: "text/plain".into(),
            cost: test_cost(),
            page: None,
        }),
    );
    let outcome = document_outcome(response, OutputMode::ReadableView);
    let (stdout, _) = write_success(outcome);
    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("\"$block\": \"/content\""));
    assert!(output.contains("[block /content bytes=4]"));
}

#[test]
fn document_protocol_json_writes_protocol_envelope_with_empty_stderr() {
    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "request-1",
        OperationResult::Outline(OutlineResult::structured(vec![], None)),
    );
    let outcome = document_outcome(response, OutputMode::ProtocolJson);
    let (stdout, stderr) = write_success(outcome);
    let stdout: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(stdout["protocol_version"], PROTOCOL_VERSION);
    let stderr = String::from_utf8(stderr).unwrap();
    assert!(stderr.is_empty());
}

#[test]
fn document_unstructured_outline_readable_view_uses_shared_output_facade() {
    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "request-1",
        OperationResult::Outline(OutlineResult::unstructured(
            UnstructuredOutlineReason::CostThreshold,
            "small note",
            "text/markdown",
            Cost {
                measurements: Vec::new(),
            },
        )),
    );
    let outcome = document_outcome(response, OutputMode::ReadableView);
    let (stdout, stderr) = write_success(outcome);

    assert!(stderr.is_empty());
    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("\"kind\": \"unstructured\""));
    assert!(output.contains("\"reason\": \"cost_threshold\""));
    assert!(output.contains("\"$block\": \"/content\""));
    assert!(output.contains("[block /content bytes=10]"));
    assert!(output.contains("small note"));
    assert!(!output.contains("\"entries\""));
    assert!(!output.contains("\"page\""));
}

#[test]
fn readable_error_uses_document_facade_and_exit_policy_stays_local() {
    let error = AppError::new(
        protocol_error_record_draft_with_summary::<typed_codes::protocol::RefNotFound>(
            "No content found for ref `L99`",
            RefDetails::new("L99"),
            DiagnosticSource::with_stage("test", "output"),
        )
        .with_guidance(["Check available entries."]),
    );
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = write_error(ErrorOutput {
        error: &error,
        output_mode: OutputMode::ReadableView,
        operation: Some(Operation::Read),
        stdout: &mut stdout,
        stderr: &mut stderr,
    });
    assert_eq!(exit, DocnavExitCode::DocumentError.code());
    assert!(stderr.is_empty());
    let output = String::from_utf8(stdout).unwrap();
    assert!(output.contains("[block /error bytes="));
    assert!(output.contains("\"code\": \"REF_NOT_FOUND\""));
    assert!(output.contains("Check available entries."));
}

#[test]
fn app_error_normalizes_non_protocol_diagnostic_before_document_output() {
    let error = AppError::new(DiagnosticRecordDraft::new::<
        typed_codes::boundary::FailedToWriteJson,
    >(
        "failed to write JSON output",
        BoundaryDetails::new("stdout closed"),
        DiagnosticSource::with_stage("test", "output"),
    ));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = write_error(ErrorOutput {
        error: &error,
        output_mode: OutputMode::ProtocolJson,
        operation: None,
        stdout: &mut stdout,
        stderr: &mut stderr,
    });

    assert_eq!(exit, DocnavExitCode::InternalError.code());
    assert!(stderr.is_empty());
    let output: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(output["error"]["code"], "INTERNAL_ERROR");
    assert_eq!(
        output["error"]["details"]["error_id"],
        "app-error-diagnostic-not-protocol"
    );
}

#[test]
fn readable_view_renderer_fatal_uses_bounded_stderr_and_internal_exit() {
    let mut stderr = Vec::new();
    let exit = write_document_output_error(
        DocumentOutputError::Render(docnav_output::RenderFailure::new("x".repeat(2_000))),
        &mut stderr,
    );

    assert_eq!(exit, DocnavExitCode::InternalError.code());
    let diagnostic = String::from_utf8(stderr).unwrap();
    assert!(diagnostic.contains("readable_view_render_failed"));
    assert!(
        diagnostic.trim_end().chars().count() <= MAX_FATAL_DIAGNOSTIC_CHARS,
        "{diagnostic}"
    );
}

#[test]
fn rendered_writer_failure_stays_an_io_failure() {
    struct FailingWriter;

    impl std::io::Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("stdout closed"))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "request-1",
        OperationResult::Outline(OutlineResult::structured(vec![], None)),
    );
    let outcome = document_outcome(response, OutputMode::ReadableView);
    let mut stdout = FailingWriter;
    let mut stderr = Vec::new();
    let exit = write_outcome(outcome, &mut stdout, &mut stderr);

    assert_eq!(exit, DocnavExitCode::InternalError.code());
    let diagnostic = String::from_utf8(stderr).unwrap();
    assert!(diagnostic.contains("stdout closed"));
    assert!(!diagnostic.contains("readable_view_render_failed"));
}

fn test_cost() -> Cost {
    Cost {
        measurements: vec![
            Measurement {
                unit: "lines".to_owned(),
                value: 1,
                scope: Some("selection".to_owned()),
            },
            Measurement {
                unit: "bytes".to_owned(),
                value: 4,
                scope: Some("selection".to_owned()),
            },
        ],
    }
}

fn document_outcome(response: ProtocolResponse, output: OutputMode) -> CommandOutcome {
    let operation = match &response {
        ProtocolResponse::Success(success) => success.operation,
        ProtocolResponse::Failure(failure) => failure.operation.unwrap_or(Operation::Outline),
    };
    let navigation_output = match output {
        OutputMode::ReadableView => NavigationOutputMode::ReadableView,
        OutputMode::ProtocolJson => NavigationOutputMode::ProtocolJson,
    };
    outcome_for_response(
        NavigationCommandOutcome {
            response,
            output: navigation_output,
            trace: NavigationInvocationTrace {
                operation,
                selected_adapter_id: Some("test-adapter".to_owned()),
                request_id: Some("request-1".to_owned()),
                failure_layer: None,
            },
        },
        output,
        None,
    )
    .unwrap()
}
