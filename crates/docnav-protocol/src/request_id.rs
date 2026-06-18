use std::time::{SystemTime, UNIX_EPOCH};

pub const GENERATED_REQUEST_ID_PREFIX: &str = "docnav-";

pub fn generate_request_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{GENERATED_REQUEST_ID_PREFIX}{nanos}")
}
