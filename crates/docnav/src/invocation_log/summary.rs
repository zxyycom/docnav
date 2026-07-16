use std::path::Path;

use cli_config_resolution::CandidateInput;
use docnav_navigation::NavigationFailureLayer;
use docnav_protocol::ProtocolDiagnosticCode;
use serde_json::{json, Value};

use crate::cli::DocumentCommand;
use crate::error::AppError;
use crate::project_context::ProjectContext;
use crate::project_paths::path_to_slash;

use super::hash::{sha256_hex, HASH_ALGORITHM};
use super::paths::resolve_project_path;

const MAX_SUMMARY_CHARS: usize = 512;

pub(super) fn document_summary(
    project: &ProjectContext,
    raw_path: &str,
    absolute_path: Option<&Path>,
) -> Value {
    let path = absolute_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| resolve_project_path(project, raw_path));
    let (display, kind) = match path.strip_prefix(&project.project_root) {
        Ok(relative) if !relative.as_os_str().is_empty() => {
            (path_to_slash(relative), "project_relative")
        }
        _ => (path_to_slash(&path), "absolute"),
    };
    json!({
        "path_display": bounded_text(&display, MAX_SUMMARY_CHARS),
        "path_kind": kind,
        "path_hash": sha256_hex(path_to_slash(&path).as_bytes()),
    })
}

pub(super) fn arguments_summary(command: &DocumentCommand) -> Value {
    let mut value = serde_json::Map::new();
    for (identity, label) in [
        ("docnav.document.page", "page"),
        ("docnav.defaults.pagination.limit", "limit"),
    ] {
        let Some(candidate) = command.cli_candidate(identity) else {
            continue;
        };
        if let CandidateInput::Value(raw) = candidate.input() {
            value.insert(label.to_owned(), raw.clone());
        }
    }
    if let Some(ref_id) = &command.ref_id {
        value.insert("ref".to_owned(), bounded_input_summary(ref_id));
    }
    if let Some(query) = &command.query {
        value.insert("query".to_owned(), bounded_input_summary(query));
    }
    Value::Object(value)
}

pub(super) fn app_error_summary(
    layer: NavigationFailureLayer,
    diagnostic: &docnav_diagnostics::DiagnosticRecordDraft,
) -> Value {
    failure_summary(
        layer.as_str(),
        Some(format!("{:?}", diagnostic.code())),
        diagnostic.summary(),
    )
}

pub(super) fn core_error_summary(layer: &str, error: &AppError) -> Value {
    let diagnostic = error.diagnostic();
    failure_summary(
        layer,
        Some(format!("{:?}", diagnostic.code())),
        diagnostic.summary(),
    )
}

pub(super) fn failure_summary(
    layer: &str,
    code: Option<String>,
    summary: impl AsRef<str>,
) -> Value {
    let mut value = json!({
        "layer": layer,
        "summary": bounded_text(summary.as_ref(), MAX_SUMMARY_CHARS),
    });
    if let Some(code) = code.filter(|code| !code.is_empty()) {
        value["code"] = json!(code);
    }
    value
}

pub(super) fn protocol_code_text(code: ProtocolDiagnosticCode) -> String {
    serde_json::to_value(code)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("{code:?}"))
}

fn bounded_input_summary(value: &str) -> Value {
    json!({
        "kind": "length_hash",
        "length": value.chars().count(),
        "hash_algorithm": HASH_ALGORITHM,
        "value_hash": sha256_hex(value.as_bytes()),
    })
}

fn bounded_text(value: &str, max_chars: usize) -> String {
    let mut text = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        text.truncate(text.trim_end().len());
    }
    text
}
