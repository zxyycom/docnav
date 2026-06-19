use docnav_protocol::{
    decode_manifest_value, decode_probe_result_value, decode_protocol_response_value,
    DecodePipelineError, Manifest, Operation, ProbeResult, ProtocolResponse, StableError,
    PROTOCOL_VERSION,
};
use serde_json::{json, Value};

use crate::process::{parse_single_json, AdapterProcessError, AdapterProcessOutput};

pub fn manifest_from_output(
    adapter_id: &str,
    output: AdapterProcessOutput,
) -> Result<Manifest, String> {
    let value = parse_single_json(&output.stdout)?;
    let manifest = decode_manifest_value(value).map_err(|error| error.to_string())?;
    if manifest.adapter.id != adapter_id {
        return Err(format!(
            "manifest adapter id {:?} does not match registry id {:?}",
            manifest.adapter.id, adapter_id
        ));
    }
    Ok(manifest)
}

pub fn ensure_capability(manifest: &Manifest, operation: Operation) -> Result<(), String> {
    if manifest.capabilities.contains(&operation) {
        Ok(())
    } else {
        Err(format!("adapter does not declare capability {operation}"))
    }
}

pub fn probe_from_output(
    adapter_id: &str,
    document_path: &str,
    output: AdapterProcessOutput,
) -> Result<ProbeResult, String> {
    let value = parse_single_json(&output.stdout)?;
    let probe = decode_probe_result_value(value).map_err(|error| error.to_string())?;
    if probe.adapter_id != adapter_id {
        return Err(format!(
            "probe adapter_id {:?} does not match registry id {:?}",
            probe.adapter_id, adapter_id
        ));
    }
    if probe.path != document_path {
        return Err(format!(
            "probe path {:?} does not match requested path {:?}",
            probe.path, document_path
        ));
    }
    Ok(probe)
}

pub fn protocol_response_from_output(
    adapter_id: &str,
    request_id: &str,
    operation: Operation,
    output: AdapterProcessOutput,
) -> Result<ProtocolResponse, StableError> {
    let value = parse_single_json(&output.stdout).map_err(|reason| {
        adapter_invoke_failed(adapter_id, reason, output.exit_code, output.stderr.clone())
    })?;
    let response = decode_protocol_response_value(value).map_err(|error| match error {
        DecodePipelineError::Schema(error) => adapter_invoke_failed(
            adapter_id,
            format!("protocol response schema validation failed: {error}"),
            output.exit_code,
            output.stderr.clone(),
        ),
        DecodePipelineError::Deserialize(error) => adapter_invoke_failed(
            adapter_id,
            format!("failed to decode protocol response: {error}"),
            output.exit_code,
            output.stderr.clone(),
        ),
        DecodePipelineError::Semantic { error, .. } => adapter_invoke_failed(
            adapter_id,
            format!("protocol response semantic validation failed: {error}"),
            output.exit_code,
            output.stderr.clone(),
        ),
    })?;

    match &response {
        ProtocolResponse::Success(success) => {
            if success.protocol_version != PROTOCOL_VERSION {
                return Err(adapter_invoke_failed(
                    adapter_id,
                    "protocol version does not match current contract",
                    output.exit_code,
                    output.stderr,
                ));
            }
            if success.request_id != request_id {
                return Err(adapter_invoke_failed(
                    adapter_id,
                    "response request_id does not match invoke request",
                    output.exit_code,
                    output.stderr,
                ));
            }
            if success.operation != operation {
                return Err(adapter_invoke_failed(
                    adapter_id,
                    "response operation does not match invoke request",
                    output.exit_code,
                    output.stderr,
                ));
            }
            if output.exit_code != Some(0) {
                return Err(adapter_invoke_failed(
                    adapter_id,
                    "adapter returned success response with non-zero exit status",
                    output.exit_code,
                    output.stderr,
                ));
            }
        }
        ProtocolResponse::Failure(failure) => {
            if failure.protocol_version != PROTOCOL_VERSION {
                return Err(adapter_invoke_failed(
                    adapter_id,
                    "protocol version does not match current contract",
                    output.exit_code,
                    output.stderr,
                ));
            }
            if failure.request_id != request_id {
                return Err(adapter_invoke_failed(
                    adapter_id,
                    "response request_id does not match invoke request",
                    output.exit_code,
                    output.stderr,
                ));
            }
            if failure.operation != Some(operation) {
                return Err(adapter_invoke_failed(
                    adapter_id,
                    "failure response operation does not match invoke request",
                    output.exit_code,
                    output.stderr,
                ));
            }
            failure.error.validate_required_details().map_err(|error| {
                adapter_invoke_failed(
                    adapter_id,
                    format!(
                        "stable error is missing required detail {} for {:?}",
                        error.field, error.code
                    ),
                    output.exit_code,
                    output.stderr.clone(),
                )
            })?;
        }
    }

    Ok(response)
}

pub fn adapter_invoke_failed(
    adapter_id: &str,
    reason: impl Into<String>,
    exit_code: Option<i32>,
    stderr: String,
) -> StableError {
    let mut stable = StableError::adapter_invoke_failed(adapter_id, reason);
    if let Some(exit_code) = exit_code {
        stable
            .details
            .insert("exit_code".to_owned(), Value::from(exit_code));
    }
    if !stderr.trim().is_empty() {
        stable
            .details
            .insert("stderr".to_owned(), Value::from(stderr));
    }
    stable
}

pub fn process_error_details(error: &AdapterProcessError) -> Value {
    json!({
        "reason": error.reason,
        "exit_code": error.exit_code,
        "stderr": error.stderr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use docnav_protocol::StableErrorCode;

    fn adapter_output(stdout: impl Into<String>) -> AdapterProcessOutput {
        AdapterProcessOutput {
            stdout: stdout.into(),
            stderr: "adapter stderr".to_owned(),
            exit_code: Some(0),
        }
    }

    fn error_reason(error: &StableError) -> &str {
        error
            .details
            .get("reason")
            .and_then(Value::as_str)
            .expect("adapter invoke failure reason")
    }

    #[test]
    // @case WB-CORE-ADAPTER-001
    fn protocol_response_schema_invalid_maps_to_adapter_invoke_failed() {
        let output = adapter_output(
            r#"{
              "protocol_version": "0.1",
              "request_id": "req-1",
              "operation": "outline",
              "ok": true,
              "result": { "page": null }
            }"#,
        );

        let error = protocol_response_from_output("stub", "req-1", Operation::Outline, output)
            .expect_err("schema-invalid response should fail");

        assert_eq!(error.code, StableErrorCode::AdapterInvokeFailed);
        assert!(error_reason(&error).contains("protocol response schema validation failed"));
    }

    #[test]
    fn protocol_response_semantic_invalid_maps_to_adapter_invoke_failed() {
        let output = adapter_output(
            r#"{
              "protocol_version": "0.1",
              "request_id": "other-req",
              "operation": "outline",
              "ok": true,
              "result": { "entries": [], "page": null }
            }"#,
        );

        let error = protocol_response_from_output("stub", "req-1", Operation::Outline, output)
            .expect_err("request id drift should fail after schema and typed decode");

        assert_eq!(error.code, StableErrorCode::AdapterInvokeFailed);
        assert!(error_reason(&error).contains("response request_id does not match invoke request"));
    }

    #[test]
    fn manifest_parse_schema_and_semantic_invalid_paths_are_distinct() {
        let malformed = manifest_from_output("stub", adapter_output("{not-json}"))
            .expect_err("malformed manifest stdout should fail");
        assert!(malformed.contains("stdout is not JSON"));

        let schema_invalid = manifest_from_output(
            "stub",
            adapter_output(r#"{ "manifest_version": "0.1", "adapter": {} }"#),
        )
        .expect_err("schema-invalid manifest should fail");
        assert!(schema_invalid.contains("manifest.schema.json"));

        let semantic_invalid = manifest_from_output(
            "stub",
            adapter_output(
                r#"{
                  "manifest_version": "0.1",
                  "adapter": {
                    "id": "other",
                    "name": "Other",
                    "version": "0.1.0"
                  },
                  "formats": [
                    {
                      "id": "stub",
                      "extensions": [".stub"],
                      "content_types": ["text/stub"]
                    }
                  ],
                  "capabilities": ["outline"]
                }"#,
            ),
        )
        .expect_err("manifest adapter id drift should fail");
        assert!(semantic_invalid.contains("does not match registry id"));
    }

    #[test]
    fn probe_parse_schema_and_semantic_invalid_paths_are_distinct() {
        let malformed = probe_from_output("stub", "doc.stub", adapter_output("{not-json}"))
            .expect_err("malformed probe stdout should fail");
        assert!(malformed.contains("stdout is not JSON"));

        let schema_invalid = probe_from_output(
            "stub",
            "doc.stub",
            adapter_output(
                r#"{
                  "probe_version": "0.1",
                  "adapter_id": "stub",
                  "path": "doc.stub",
                  "supported": true,
                  "format": "stub",
                  "confidence": 2.0,
                  "reasons": []
                }"#,
            ),
        )
        .expect_err("schema-invalid probe should fail");
        assert!(schema_invalid.contains("probe-result.schema.json"));

        let semantic_invalid = probe_from_output(
            "stub",
            "doc.stub",
            adapter_output(
                r#"{
                  "probe_version": "0.1",
                  "adapter_id": "stub",
                  "path": "other.stub",
                  "supported": true,
                  "format": "stub",
                  "confidence": 1.0,
                  "reasons": [
                    { "code": "EXTENSION_MATCH", "detail": "extension matched" }
                  ]
                }"#,
            ),
        )
        .expect_err("probe path drift should fail after schema and typed decode");
        assert!(semantic_invalid.contains("does not match requested path"));
    }
}
