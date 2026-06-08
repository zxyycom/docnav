use docnav_protocol::{
    validate_manifest_value, validate_probe_result_value, validate_protocol_response_value,
    Manifest, Operation, ProbeResult, ProtocolResponse, StableError, PROTOCOL_VERSION,
};
use serde_json::{json, Value};

use crate::process::{parse_single_json, AdapterProcessError, AdapterProcessOutput};

pub fn manifest_from_output(
    adapter_id: &str,
    output: AdapterProcessOutput,
) -> Result<Manifest, String> {
    let value = parse_single_json(&output.stdout)?;
    validate_manifest_value(&value).map_err(|error| error.to_string())?;
    let manifest = serde_json::from_value::<Manifest>(value).map_err(|error| error.to_string())?;
    manifest
        .validate_semantics()
        .map_err(|error| error.to_string())?;
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
    validate_probe_result_value(&value).map_err(|error| error.to_string())?;
    let probe = serde_json::from_value::<ProbeResult>(value).map_err(|error| error.to_string())?;
    probe
        .validate_semantics()
        .map_err(|error| error.to_string())?;
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
    validate_protocol_response_value(&value).map_err(|error| {
        adapter_invoke_failed(
            adapter_id,
            format!("protocol response schema validation failed: {error}"),
            output.exit_code,
            output.stderr.clone(),
        )
    })?;
    let response = serde_json::from_value::<ProtocolResponse>(value).map_err(|error| {
        adapter_invoke_failed(
            adapter_id,
            format!("failed to decode protocol response: {error}"),
            output.exit_code,
            output.stderr.clone(),
        )
    })?;
    response.validate().map_err(|error| {
        adapter_invoke_failed(
            adapter_id,
            format!("protocol response semantic validation failed: {error}"),
            output.exit_code,
            output.stderr.clone(),
        )
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

pub fn adapter_unavailable(adapter_id: &str, reason: impl Into<String>) -> StableError {
    StableError::adapter_unavailable(adapter_id, reason)
}

pub fn adapter_unavailable_from_process(
    adapter_id: &str,
    error: AdapterProcessError,
) -> StableError {
    let mut stable = StableError::adapter_unavailable(adapter_id, error.reason);
    if let Some(exit_code) = error.exit_code {
        stable
            .details
            .insert("exit_code".to_owned(), Value::from(exit_code));
    }
    if !error.stderr.trim().is_empty() {
        stable
            .details
            .insert("stderr".to_owned(), Value::from(error.stderr));
    }
    stable
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
