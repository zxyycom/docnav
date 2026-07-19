use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;
use docnav_protocol::{
    positive_result, validate_protocol_response_value, Cost, Entry, FindResult, InfoAdapter,
    InfoDocument, InfoResult, Location, Measurement, Operation, OperationResult, OutlineResult,
    ProtocolResponse, ReadResult, UnstructuredOutlineReason, PROTOCOL_VERSION,
};
use serde_json::{json, Value};

mod auto_read;

static FAILING_RENDERER_CALLS: AtomicUsize = AtomicUsize::new(0);

fn read_result() -> OperationResult {
    OperationResult::Read(ReadResult {
        ref_id: "R1".into(),
        content: "正文".into(),
        content_type: "text/plain".into(),
        cost: test_cost(),
        page: None,
    })
}

fn unstructured_outline_result() -> OperationResult {
    OperationResult::Outline(OutlineResult::unstructured(
        UnstructuredOutlineReason::PathRule,
        "full body\n",
        "text/markdown",
        Cost {
            measurements: Vec::new(),
        },
    ))
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
                value: 6,
                scope: Some("selection".to_owned()),
            },
            Measurement {
                unit: "tokens".to_owned(),
                value: 8,
                scope: Some("selection".to_owned()),
            },
        ],
    }
}

fn success_response() -> ProtocolResponse {
    ProtocolResponse::success(PROTOCOL_VERSION, "request-1", read_result())
}

fn failure_response() -> ProtocolResponse {
    ProtocolResponse::failure(
        PROTOCOL_VERSION,
        "request-1",
        Some(Operation::Read),
        docnav_protocol::ProtocolError::ref_not_found("R99"),
    )
}

fn render_custom_success(response: &ProtocolResponse) -> Result<String, RenderFailure> {
    let ProtocolResponse::Success(success) = response else {
        panic!("custom success renderer received a failure response");
    };
    assert_eq!(success.request_id, "request-1");
    assert!(matches!(success.result, OperationResult::Read(_)));
    Ok("custom success text".to_owned())
}

fn render_custom_failure(response: &ProtocolResponse) -> Result<String, RenderFailure> {
    let ProtocolResponse::Failure(failure) = response else {
        panic!("custom failure renderer received a success response");
    };
    assert_eq!(failure.request_id, "request-1");
    assert_eq!(failure.operation, Some(Operation::Read));
    assert_eq!(failure.error.code().protocol_code(), "REF_NOT_FOUND");
    Ok("custom failure text".to_owned())
}

fn render_failure(_: &ProtocolResponse) -> Result<String, RenderFailure> {
    FAILING_RENDERER_CALLS.fetch_add(1, Ordering::SeqCst);
    Err(RenderFailure::new("boom"))
}

fn render_for_writer_failure(_: &ProtocolResponse) -> Result<String, RenderFailure> {
    Ok("rendered before writer failure".to_owned())
}

#[derive(Default)]
struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("writer failed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn assert_single_trailing_lf(output: &str) -> &str {
    let without_trailing_lf = output
        .strip_suffix('\n')
        .expect("readable-view text must end with LF");
    assert!(
        !without_trailing_lf.ends_with('\n'),
        "readable-view text must have exactly one trailing LF"
    );
    without_trailing_lf
}

fn assert_readable_header(output: &str, expected_header: Value) {
    let header = assert_single_trailing_lf(output);
    assert!(
        !header.contains("\n\n"),
        "header-only readable-view text must not contain a block separator"
    );
    let actual_header: Value = serde_json::from_str(header).unwrap();
    assert_eq!(actual_header, expected_header);
}

fn assert_readable_block(output: &str, expected_header: Value, pointer: &str, payload: &str) {
    let output = assert_single_trailing_lf(output);
    let (header, block) = output
        .split_once("\n\n")
        .expect("block readable-view text must contain one LF separator");
    let actual_header: Value = serde_json::from_str(header).unwrap();
    assert_eq!(actual_header, expected_header);

    let utf8_byte_len = payload.len();
    let start_marker = format!("[block {pointer} bytes={utf8_byte_len}]\n");
    let block = block
        .strip_prefix(&start_marker)
        .expect("block start marker must include the UTF-8 byte length");
    let end_marker = format!("[endblock {pointer}]");
    let payload_with_framing = block
        .strip_suffix(&end_marker)
        .expect("block must end with the matching end marker");
    if payload.ends_with('\n') {
        assert_eq!(payload_with_framing, payload);
    } else {
        assert_eq!(payload_with_framing.strip_suffix('\n'), Some(payload));
    }
}

// @case WB-OUTPUT-DOCUMENT-001
#[test]
fn custom_renderer_receives_success_response_and_controls_exact_text() {
    let response = success_response();
    let mut stdout = Vec::new();

    write_document_response(
        &response,
        OutputPlan::Rendered(render_custom_success),
        &mut stdout,
    )
    .unwrap();

    assert_eq!(stdout, b"custom success text");
}

#[test]
fn custom_renderer_receives_failure_response() {
    let response = failure_response();
    let mut stdout = Vec::new();

    write_document_response(
        &response,
        OutputPlan::Rendered(render_custom_failure),
        &mut stdout,
    )
    .unwrap();

    assert_eq!(stdout, b"custom failure text");
}

#[test]
fn render_failure_happens_before_stdout_and_strategy_runs_once() {
    FAILING_RENDERER_CALLS.store(0, Ordering::SeqCst);
    let mut stdout = Vec::new();

    let error = write_document_response(
        &success_response(),
        OutputPlan::Rendered(render_failure),
        &mut stdout,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        DocumentOutputError::Render(ref failure) if failure.to_string() == "boom"
    ));
    assert_eq!(error.to_string(), "render failed: boom");
    assert!(stdout.is_empty());
    assert_eq!(FAILING_RENDERER_CALLS.load(Ordering::SeqCst), 1);
}

#[test]
fn writer_failure_after_rendering_stays_a_writer_error() {
    let mut stdout = FailingWriter;

    let error = write_document_response(
        &success_response(),
        OutputPlan::Rendered(render_for_writer_failure),
        &mut stdout,
    )
    .unwrap_err();

    assert!(matches!(error, DocumentOutputError::StdoutWrite(_)));
}

#[test]
fn protocol_json_serializes_success_and_failure_responses_without_rendering() {
    for response in [success_response(), failure_response()] {
        let mut stdout = Vec::new();

        write_document_response(&response, OutputPlan::ProtocolJson, &mut stdout).unwrap();

        let actual: Value = serde_json::from_slice(&stdout).unwrap();
        validate_protocol_response_value(&actual).unwrap();
        assert_eq!(actual, serde_json::to_value(&response).unwrap());
    }
}

// @case WB-READABLE-VIEW-001
#[test]
fn built_in_renderer_maps_structured_outline_response() {
    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "request-1",
        OperationResult::Outline(OutlineResult::structured(
            vec![Entry {
                ref_id: "H:L1:H2".into(),
                label: "Intro".into(),
                kind: Some("heading".into()),
                location: None,
                summary: None,
                excerpt: None,
                rank: None,
                cost: Some(test_cost()),
                metadata: Some(serde_json::Map::from_iter([(
                    "heading_level".into(),
                    json!(2),
                )])),
            }],
            None,
        )),
    );
    let output = render_readable_response(&response).unwrap();

    assert_readable_header(
        &output,
        json!({
            "kind": "structured",
            "entries": [{
                "ref": "H:L1:H2",
                "display": "H2 Intro | 1 line | 0.0 KB | 8 tokens",
            }],
            "page": null,
        }),
    );
}

#[test]
fn built_in_renderer_maps_find_response() {
    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "request-1",
        OperationResult::Find(FindResult::new(
            vec![Entry {
                ref_id: "M1".into(),
                label: "needle".into(),
                kind: Some("match".into()),
                location: Some(Location {
                    line_start: positive_result(7).unwrap(),
                    line_end: None,
                }),
                summary: None,
                excerpt: None,
                rank: None,
                cost: None,
                metadata: None,
            }],
            None,
        )),
    );
    let output = render_readable_response(&response).unwrap();

    assert_readable_header(
        &output,
        json!({
            "matches": [{
                "ref": "M1",
                "display": "L7: needle",
            }],
            "page": null,
        }),
    );
}

#[test]
fn built_in_renderer_maps_info_response() {
    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "request-1",
        OperationResult::Info(InfoResult {
            document: Some(InfoDocument {
                content_type: Some("text/markdown".into()),
                encoding: Some("utf-8".into()),
                size: Some(Measurement {
                    unit: "bytes".into(),
                    value: 1536,
                    scope: None,
                }),
            }),
            adapter: Some(InfoAdapter {
                id: Some("docnav-markdown".into()),
                format: Some("markdown".into()),
            }),
            metadata: Some(serde_json::Map::from_iter([(
                "heading_count".into(),
                json!(2),
            )])),
        }),
    );
    let output = render_readable_response(&response).unwrap();

    assert_readable_header(
        &output,
        json!({
            "display": "Markdown | text/markdown | 2 headings | 1.5 KB",
        }),
    );
}

#[test]
fn built_in_renderer_maps_read_response_and_preserves_block_framing() {
    let content = "正文";
    let output = render_readable_response(&success_response()).unwrap();

    assert_readable_block(
        &output,
        json!({
            "content": {
                "$block": "/content",
                "bytes": content.len(),
            },
            "content_type": "text/plain",
            "cost": "1 line | 0.0 KB | 8 tokens",
            "page": null,
            "ref": "R1",
        }),
        "/content",
        content,
    );
}

#[test]
fn built_in_renderer_maps_failure_response_and_preserves_block_framing() {
    let error_text = "Ref was not found.";
    let output = render_readable_response(&failure_response()).unwrap();

    assert_readable_block(
        &output,
        json!({
            "code": "REF_NOT_FOUND",
            "details": {
                "ref": "R99",
            },
            "error": {
                "$block": "/error",
                "bytes": error_text.len(),
            },
            "guidance": [
                "Run outline again and use a returned ref.",
            ],
            "location": {
                "ref": "R99",
            },
            "owner": "adapter",
        }),
        "/error",
        error_text,
    );
}

#[test]
fn built_in_renderer_maps_unstructured_outline_response_and_preserves_block_framing() {
    let content = "full body\n";
    let response =
        ProtocolResponse::success(PROTOCOL_VERSION, "request-1", unstructured_outline_result());
    let output = render_readable_response(&response).unwrap();

    assert_readable_block(
        &output,
        json!({
            "content": {
                "$block": "/content",
                "bytes": content.len(),
            },
            "content_type": "text/markdown",
            "cost": {
                "measurements": [],
            },
            "kind": "unstructured",
            "reason": "path_rule",
        }),
        "/content",
        content,
    );
}
