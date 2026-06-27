use serde_json::{json, Value};

use crate::adapter_process::AdapterProcessError;
use crate::error::AppError;

pub fn adapter_invoke_failed(
    adapter_id: &str,
    reason: impl Into<String>,
    exit_code: Option<i32>,
    stderr: String,
) -> AppError {
    AppError::adapter_invoke_failed(adapter_id, reason, exit_code, stderr)
}

pub fn process_error_details(error: &AdapterProcessError) -> Value {
    json!({
        "reason": error.reason,
        "exit_code": error.exit_code,
        "stderr": error.stderr,
    })
}
