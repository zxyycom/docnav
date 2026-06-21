use docnav_protocol::{
    decode_manifest_value, decode_probe_result_value, decode_protocol_response_value,
    DecodePipelineError, FailureResponse, Manifest, Operation, ProbeResult, ProtocolResponse,
    StableError, SuccessResponse, PROTOCOL_VERSION,
};
use serde_json::{json, Value};

use crate::adapter_process::{parse_single_json, AdapterProcessError, AdapterProcessOutput};

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
    let context = ProtocolResponseContext::new(adapter_id, request_id, operation, &output);
    let value = parse_protocol_response_json(&context, &output.stdout)?;
    let response = decode_protocol_response(&context, value)?;
    validate_protocol_response_semantics(&context, &response)?;

    Ok(response)
}

struct ProtocolResponseContext<'a> {
    adapter_id: &'a str,
    request_id: &'a str,
    operation: Operation,
    exit_code: Option<i32>,
    stderr: &'a str,
}

impl<'a> ProtocolResponseContext<'a> {
    fn new(
        adapter_id: &'a str,
        request_id: &'a str,
        operation: Operation,
        output: &'a AdapterProcessOutput,
    ) -> Self {
        Self {
            adapter_id,
            request_id,
            operation,
            exit_code: output.exit_code,
            stderr: &output.stderr,
        }
    }

    fn adapter_invoke_failed(&self, reason: impl Into<String>) -> StableError {
        adapter_invoke_failed(
            self.adapter_id,
            reason,
            self.exit_code,
            self.stderr.to_owned(),
        )
    }
}

fn parse_protocol_response_json(
    context: &ProtocolResponseContext<'_>,
    stdout: &str,
) -> Result<Value, StableError> {
    parse_single_json(stdout).map_err(|reason| context.adapter_invoke_failed(reason))
}

fn decode_protocol_response(
    context: &ProtocolResponseContext<'_>,
    value: Value,
) -> Result<ProtocolResponse, StableError> {
    decode_protocol_response_value(value)
        .map_err(|error| context.adapter_invoke_failed(protocol_decode_failure_reason(error)))
}

fn protocol_decode_failure_reason<E>(error: DecodePipelineError<ProtocolResponse, E>) -> String
where
    E: std::fmt::Display,
{
    match error {
        DecodePipelineError::Schema(error) => {
            format!("protocol response schema validation failed: {error}")
        }
        DecodePipelineError::Deserialize(error) => {
            format!("failed to decode protocol response: {error}")
        }
        DecodePipelineError::Semantic { error, .. } => {
            format!("protocol response semantic validation failed: {error}")
        }
    }
}

fn validate_protocol_response_semantics(
    context: &ProtocolResponseContext<'_>,
    response: &ProtocolResponse,
) -> Result<(), StableError> {
    match response {
        ProtocolResponse::Success(success) => validate_success_response(context, success),
        ProtocolResponse::Failure(failure) => validate_failure_response(context, failure),
    }
}

fn validate_success_response(
    context: &ProtocolResponseContext<'_>,
    success: &SuccessResponse,
) -> Result<(), StableError> {
    validate_common_response_fields(context, &success.protocol_version, &success.request_id)?;
    if success.operation != context.operation {
        return Err(
            context.adapter_invoke_failed("response operation does not match invoke request")
        );
    }
    if context.exit_code != Some(0) {
        return Err(context
            .adapter_invoke_failed("adapter returned success response with non-zero exit status"));
    }
    Ok(())
}

fn validate_failure_response(
    context: &ProtocolResponseContext<'_>,
    failure: &FailureResponse,
) -> Result<(), StableError> {
    validate_common_response_fields(context, &failure.protocol_version, &failure.request_id)?;
    if failure.operation != Some(context.operation) {
        return Err(context
            .adapter_invoke_failed("failure response operation does not match invoke request"));
    }
    failure.error.validate_required_details().map_err(|error| {
        context.adapter_invoke_failed(format!(
            "stable error is missing required detail {} for {:?}",
            error.field, error.code
        ))
    })
}

fn validate_common_response_fields(
    context: &ProtocolResponseContext<'_>,
    protocol_version: &str,
    request_id: &str,
) -> Result<(), StableError> {
    if protocol_version != PROTOCOL_VERSION {
        return Err(
            context.adapter_invoke_failed("protocol version does not match current contract")
        );
    }
    if request_id != context.request_id {
        return Err(
            context.adapter_invoke_failed("response request_id does not match invoke request")
        );
    }
    Ok(())
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
    // @case WB-CORE-ADAPTER-001
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
