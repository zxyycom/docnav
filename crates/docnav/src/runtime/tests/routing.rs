use std::fs;

use docnav_protocol::Operation;
use serde_json::json;

use super::super::{AdapterRuntime, DocnavRuntime, DocumentRequest};
use super::support::*;
use crate::cli::OutputMode;

#[test]
fn automatic_unknown_suffix_fails_before_target_document_io() {
    let workspace = temp_workspace("unknown-suffix-before-io");
    let project_root = workspace.path().join("project");
    fs::create_dir_all(&project_root).unwrap();
    let missing_path = project_root
        .join("missing.unknown")
        .to_string_lossy()
        .into_owned();
    let context = default_context(project_root);
    let mut command = outline_command(None, None);
    command.path = missing_path.clone();
    let request = DocumentRequest::from_config_context(command, context);

    let (exit_code, output) = write_document_result(
        AdapterRuntime.execute_document(request),
        Operation::Outline,
        OutputMode::ProtocolJson,
    );

    assert_eq!(exit_code, 3);
    assert_eq!(
        output["error"]["code"], "FORMAT_UNKNOWN",
        "automatic routing must reject an unknown suffix before target-document I/O: {output}"
    );
    assert_eq!(
        output["error"]["details"],
        json!({
            "path": missing_path,
            "reason": "FORMAT_NOT_RECOGNIZED",
            "candidates": [],
        })
    );

    let serialized = serde_json::to_string(&output).unwrap();
    for private_key in [
        "\"format\":",
        "\"format_identity\":",
        "\"matched_hint\":",
        "\"matched_format\":",
        "\"routing_pathname\":",
    ] {
        assert!(
            !serialized.contains(private_key),
            "protocol output leaked routing-private field {private_key}: {serialized}"
        );
    }
}

#[test]
fn automatic_known_suffix_reaches_post_selection_path_failure() {
    let workspace = temp_workspace("known-suffix-after-selection");
    let project_root = workspace.path().join("project");
    fs::create_dir_all(&project_root).unwrap();
    let missing_path = project_root
        .join("missing.md")
        .to_string_lossy()
        .into_owned();
    let context = default_context(project_root);
    let mut command = outline_command(None, None);
    command.path = missing_path.clone();
    let request = DocumentRequest::from_config_context(command, context);

    let (exit_code, output) = write_document_result(
        AdapterRuntime.execute_document(request),
        Operation::Outline,
        OutputMode::ProtocolJson,
    );

    assert_eq!(exit_code, 3);
    assert_eq!(output["error"]["code"], "DOCUMENT_NOT_FOUND");
    assert_eq!(output["error"]["details"], json!({ "path": missing_path }));

    let serialized = serde_json::to_string(&output).unwrap();
    for private_key in [
        "\"format\":",
        "\"format_identity\":",
        "\"matched_hint\":",
        "\"matched_format\":",
        "\"routing_pathname\":",
    ] {
        assert!(
            !serialized.contains(private_key),
            "protocol output leaked routing-private field {private_key}: {serialized}"
        );
    }
}
