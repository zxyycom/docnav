use docnav_protocol::{AutoReadResult, StructuredOutlineResult};
use serde_json::json;

use super::*;

#[test]
fn protocol_json_and_readable_view_share_outline_auto_read_facts() {
    let content = "## Only\n\nNested 正文.\n";
    let page = positive_result(2).unwrap();
    let response = ProtocolResponse::success(
        PROTOCOL_VERSION,
        "request-1",
        OperationResult::Outline(OutlineResult::Structured(StructuredOutlineResult {
            entries: vec![Entry {
                ref_id: "H:L3:H2".into(),
                label: "Only".into(),
                kind: None,
                location: None,
                summary: None,
                excerpt: None,
                rank: None,
                cost: None,
                metadata: None,
            }],
            page: Some(page),
            auto_read: Some(AutoReadResult::unique_ref(ReadResult {
                ref_id: "H:L3:H2".into(),
                content: content.into(),
                content_type: "text/markdown".into(),
                cost: test_cost(),
                page: Some(page),
            })),
        })),
    );

    let mut protocol_stdout = Vec::new();
    write_document_response(&response, OutputPlan::ProtocolJson, &mut protocol_stdout).unwrap();
    let protocol: Value = serde_json::from_slice(&protocol_stdout).unwrap();
    validate_protocol_response_value(&protocol).unwrap();
    assert_eq!(protocol, serde_json::to_value(&response).unwrap());
    assert_eq!(
        protocol.pointer("/result/auto_read/read/content"),
        Some(&json!(content))
    );

    let mut readable_stdout = Vec::new();
    write_document_response(
        &response,
        OutputPlan::Rendered(render_readable_response),
        &mut readable_stdout,
    )
    .unwrap();
    let readable = String::from_utf8(readable_stdout).unwrap();
    assert_readable_block(
        &readable,
        json!({
            "kind": "structured",
            "entries": [{
                "ref": "H:L3:H2",
                "display": "Only",
            }],
            "page": 2,
            "auto_read": {
                "reason": "unique_ref",
                "read": {
                    "ref": "H:L3:H2",
                    "content": {
                        "$block": "/auto_read/read/content",
                        "bytes": content.len(),
                    },
                    "content_type": "text/markdown",
                    "cost": "1 line | 0.0 KB | 8 tokens",
                    "page": 2,
                },
            },
        }),
        "/auto_read/read/content",
        content,
    );
}

#[test]
fn built_in_renderer_maps_find_auto_read_response() {
    let content = "matched content";
    let mut result = FindResult::new(
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
    );
    result.auto_read = Some(AutoReadResult::unique_ref(ReadResult {
        ref_id: "M1".into(),
        content: content.into(),
        content_type: "text/plain".into(),
        cost: test_cost(),
        page: None,
    }));
    let response =
        ProtocolResponse::success(PROTOCOL_VERSION, "request-1", OperationResult::Find(result));
    let output = render_readable_response(&response).unwrap();

    assert_readable_block(
        &output,
        json!({
            "matches": [{
                "ref": "M1",
                "display": "L7: needle",
            }],
            "page": null,
            "auto_read": {
                "reason": "unique_ref",
                "read": {
                    "ref": "M1",
                    "content": {
                        "$block": "/auto_read/read/content",
                        "bytes": content.len(),
                    },
                    "content_type": "text/plain",
                    "cost": "1 line | 0.0 KB | 8 tokens",
                    "page": null,
                },
            },
        }),
        "/auto_read/read/content",
        content,
    );
}
