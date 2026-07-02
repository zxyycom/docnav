use docnav_diagnostics::{DiagnosticDetailsPayload, ProtocolDiagnosticCode};
use serde_json::{json, Value};

use crate::ErrorDetails;

use super::InvalidErrorDetail;

pub(super) fn location_from_details(details: &ErrorDetails) -> Option<Value> {
    config_issue_location(details)
        .or_else(|| string_detail(details, "path").map(|path| json!({ "path": path })))
        .or_else(|| string_detail(details, "field").map(|field| json!({ "field": field })))
        .or_else(|| string_detail(details, "ref").map(|ref_id| json!({ "ref": ref_id })))
        .or_else(|| {
            string_detail(details, "adapter_id")
                .map(|adapter_id| json!({ "adapter_id": adapter_id }))
        })
}

fn config_issue_location(details: &ErrorDetails) -> Option<Value> {
    let issue = details.get("config_issues")?.as_array()?.first()?;
    let config_path = issue.get("path")?.as_str()?;
    let mut location = serde_json::Map::new();
    location.insert("config_path".to_owned(), json!(config_path));
    if let Some(field) = issue.get("field").and_then(Value::as_str) {
        location.insert("field".to_owned(), json!(field));
    }
    Some(Value::Object(location))
}

pub(super) fn received_from_details(details: &ErrorDetails) -> Option<Value> {
    details
        .get("received")
        .cloned()
        .or_else(|| option_issue_detail(details, "received"))
}

pub(super) fn expected_from_details(details: &ErrorDetails) -> Option<Value> {
    option_issue_detail(details, "expected").or_else(|| details.get("accepted").cloned())
}

pub(super) fn error_details_from_payload<T>(details: &T) -> ErrorDetails
where
    T: DiagnosticDetailsPayload,
{
    let Value::Object(object) =
        serde_json::to_value(details).expect("diagnostic details payloads serialize to objects")
    else {
        unreachable!("diagnostic details payloads serialize to objects");
    };
    object.into_iter().collect()
}

pub(super) fn payload_from_error_details<T>(
    code: ProtocolDiagnosticCode,
    details: &ErrorDetails,
) -> Result<T, InvalidErrorDetail>
where
    T: DiagnosticDetailsPayload,
{
    serde_json::from_value(Value::Object(details.clone().into_iter().collect())).map_err(|error| {
        InvalidErrorDetail {
            code,
            reason: error.to_string(),
        }
    })
}

fn string_detail(details: &ErrorDetails, key: &str) -> Option<String> {
    details.get(key)?.as_str().map(ToOwned::to_owned)
}

fn option_issue_detail(details: &ErrorDetails, key: &str) -> Option<Value> {
    details
        .get("option_issues")?
        .as_array()?
        .first()?
        .get(key)
        .cloned()
}
