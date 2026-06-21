use docnav_protocol::StableError;
use serde_json::{json, Value};

use crate::adapter_process::AdapterProcessError;

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
