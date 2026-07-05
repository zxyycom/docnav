use docnav_diagnostics::{
    DiagnosticCategory, DiagnosticCode, DiagnosticSource, ProtocolDiagnosticCode,
};

use crate::constants::protocol_error_messages;

use super::ProtocolErrorCategory;

pub(super) fn default_owner_for_code(code: ProtocolDiagnosticCode) -> &'static str {
    match code {
        ProtocolDiagnosticCode::InvalidRequest => "protocol_input",
        ProtocolDiagnosticCode::FormatUnknown | ProtocolDiagnosticCode::FormatAmbiguous => {
            "docnav_navigation_routing"
        }
        ProtocolDiagnosticCode::AdapterUnavailable | ProtocolDiagnosticCode::InternalError => {
            "adapter"
        }
        ProtocolDiagnosticCode::DocumentNotFound
        | ProtocolDiagnosticCode::DocumentPathInvalid
        | ProtocolDiagnosticCode::DocumentEncodingUnsupported
        | ProtocolDiagnosticCode::RefNotFound
        | ProtocolDiagnosticCode::RefAmbiguous
        | ProtocolDiagnosticCode::RefInvalid => "adapter",
    }
}

pub(super) fn owner_from_source(source: &DiagnosticSource) -> String {
    match (source.component.as_str(), source.stage.as_deref()) {
        ("docnav", Some("core")) => "core_cli".to_owned(),
        ("docnav-parameter-resolution", Some("parameter-resolution")) => {
            "parameter_resolution".to_owned()
        }
        ("docnav", Some("runtime")) => "runtime".to_owned(),
        ("docnav", Some("adapter-output")) => "adapter_boundary".to_owned(),
        ("docnav-navigation", Some("routing")) => "docnav_navigation_routing".to_owned(),
        ("docnav-adapter-contracts", Some("adapter" | "adapter-error")) => "adapter".to_owned(),
        (component, Some(stage)) => {
            format!("{}_{}", owner_segment(component), owner_segment(stage))
        }
        (component, None) => owner_segment(component),
    }
}

pub const fn protocol_error_default_guidance(code: ProtocolDiagnosticCode) -> &'static str {
    match code {
        ProtocolDiagnosticCode::InvalidRequest => "Correct the request input and retry.",
        ProtocolDiagnosticCode::DocumentNotFound => "Check the document path and retry.",
        ProtocolDiagnosticCode::DocumentPathInvalid => "Use a valid document path.",
        ProtocolDiagnosticCode::DocumentEncodingUnsupported => "Use a UTF-8 encoded document.",
        ProtocolDiagnosticCode::FormatUnknown => {
            "Use a supported document format or select a built-in adapter from this release."
        }
        ProtocolDiagnosticCode::FormatAmbiguous => "Select an adapter explicitly.",
        ProtocolDiagnosticCode::RefNotFound => "Run outline again and use a returned ref.",
        ProtocolDiagnosticCode::RefAmbiguous => "Use a more specific ref from outline.",
        ProtocolDiagnosticCode::RefInvalid => "Use a valid ref returned by outline.",
        ProtocolDiagnosticCode::AdapterUnavailable => {
            "Select an adapter from the current core release static registry."
        }
        ProtocolDiagnosticCode::InternalError => {
            "Retry the command or report the internal error id."
        }
    }
}

pub const fn protocol_error_default_message(code: ProtocolDiagnosticCode) -> &'static str {
    match code {
        ProtocolDiagnosticCode::InvalidRequest => protocol_error_messages::INVALID_PROTOCOL_REQUEST,
        ProtocolDiagnosticCode::DocumentNotFound => protocol_error_messages::DOCUMENT_NOT_FOUND,
        ProtocolDiagnosticCode::DocumentPathInvalid => {
            protocol_error_messages::DOCUMENT_PATH_INVALID
        }
        ProtocolDiagnosticCode::DocumentEncodingUnsupported => {
            protocol_error_messages::DOCUMENT_ENCODING_UNSUPPORTED
        }
        ProtocolDiagnosticCode::FormatUnknown => protocol_error_messages::DOCUMENT_FORMAT_UNKNOWN,
        ProtocolDiagnosticCode::FormatAmbiguous => {
            protocol_error_messages::DOCUMENT_FORMAT_AMBIGUOUS
        }
        ProtocolDiagnosticCode::RefNotFound => protocol_error_messages::REF_NOT_FOUND,
        ProtocolDiagnosticCode::RefAmbiguous => protocol_error_messages::REF_AMBIGUOUS,
        ProtocolDiagnosticCode::RefInvalid => protocol_error_messages::REF_INVALID,
        ProtocolDiagnosticCode::AdapterUnavailable => protocol_error_messages::ADAPTER_UNAVAILABLE,
        ProtocolDiagnosticCode::InternalError => protocol_error_messages::INTERNAL_ERROR,
    }
}

pub const fn protocol_error_category(code: ProtocolDiagnosticCode) -> ProtocolErrorCategory {
    match DiagnosticCode::Protocol(code).category() {
        DiagnosticCategory::Request => ProtocolErrorCategory::Request,
        DiagnosticCategory::Document => ProtocolErrorCategory::Document,
        DiagnosticCategory::AdapterBoundary => ProtocolErrorCategory::AdapterBoundary,
        DiagnosticCategory::Internal => ProtocolErrorCategory::Internal,
    }
}

fn owner_segment(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'A'..='Z' => character.to_ascii_lowercase(),
            'a'..='z' | '0'..='9' => character,
            _ => '_',
        })
        .collect::<String>()
        .split('_')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}
