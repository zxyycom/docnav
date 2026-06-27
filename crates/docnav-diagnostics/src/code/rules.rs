use crate::details::DiagnosticDetailsRule;

use super::details::{
    ADAPTER_CANDIDATE_FIELDS, ADAPTER_CONFIG_FIELDS, ADAPTER_REASON_FIELDS, BOUNDARY_FIELDS,
    CAPABILITY_ADAPTER_FIELDS, CLI_ARGV_FIELDS, FIELD_REASON_FIELDS, FORMAT_AMBIGUOUS_FIELDS,
    FORMAT_UNKNOWN_FIELDS, INTERNAL_FIELDS, PATH_ENCODING_FIELDS, PATH_FIELDS, PATH_REASON_FIELDS,
    REF_CANDIDATE_FIELDS, REF_FIELDS, REF_REASON_FIELDS,
};
use super::{
    BoundaryDiagnosticCode, DiagnosticCategory, DiagnosticEffect, DiagnosticSeverity,
    ProtocolDiagnosticCode, ReadableWarningDiagnosticCode,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ProtocolDiagnosticRule {
    pub(super) code: ProtocolDiagnosticCode,
    pub(super) protocol_code: &'static str,
    pub(super) category: DiagnosticCategory,
    pub(super) severity: DiagnosticSeverity,
    pub(super) effect: DiagnosticEffect,
    pub(super) details: DiagnosticDetailsRule,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ReadableWarningDiagnosticRule {
    pub(super) code: ReadableWarningDiagnosticCode,
    pub(super) warning_id: &'static str,
    pub(super) effect: DiagnosticEffect,
    pub(super) details: DiagnosticDetailsRule,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct BoundaryDiagnosticRule {
    pub(super) code: BoundaryDiagnosticCode,
    pub(super) id: &'static str,
    pub(super) category: DiagnosticCategory,
    pub(super) severity: DiagnosticSeverity,
    pub(super) effect: DiagnosticEffect,
    pub(super) details: DiagnosticDetailsRule,
}

macro_rules! protocol_rules {
    ($($code:ident => ($protocol_code:literal, $category:ident, $severity:ident, $effect:ident, $fields:ident)),+ $(,)?) => {
        [
            $(
                ProtocolDiagnosticRule {
                    code: ProtocolDiagnosticCode::$code,
                    protocol_code: $protocol_code,
                    category: DiagnosticCategory::$category,
                    severity: DiagnosticSeverity::$severity,
                    effect: DiagnosticEffect::$effect,
                    details: DiagnosticDetailsRule::exact($fields),
                },
            )+
        ]
    };
}

macro_rules! readable_warning_rules {
    ($($code:ident => ($warning_id:literal, $effect:ident, $fields:ident)),+ $(,)?) => {
        [
            $(
                ReadableWarningDiagnosticRule {
                    code: ReadableWarningDiagnosticCode::$code,
                    warning_id: $warning_id,
                    effect: DiagnosticEffect::$effect,
                    details: DiagnosticDetailsRule::exact($fields),
                },
            )+
        ]
    };
}

macro_rules! boundary_rules {
    ($($code:ident => ($id:literal, $category:ident, $severity:ident, $effect:ident)),+ $(,)?) => {
        [
            $(
                BoundaryDiagnosticRule {
                    code: BoundaryDiagnosticCode::$code,
                    id: $id,
                    category: DiagnosticCategory::$category,
                    severity: DiagnosticSeverity::$severity,
                    effect: DiagnosticEffect::$effect,
                    details: DiagnosticDetailsRule::exact(BOUNDARY_FIELDS),
                },
            )+
        ]
    };
}

pub(super) const PROTOCOL_RULES: [ProtocolDiagnosticRule; 13] = protocol_rules![
    InvalidRequest => ("INVALID_REQUEST", Request, Error, InputRejected, FIELD_REASON_FIELDS),
    DocumentNotFound => ("DOCUMENT_NOT_FOUND", Document, Error, DocumentFailed, PATH_FIELDS),
    DocumentPathInvalid => ("DOCUMENT_PATH_INVALID", Document, Error, DocumentFailed, PATH_REASON_FIELDS),
    DocumentEncodingUnsupported => ("DOCUMENT_ENCODING_UNSUPPORTED", Document, Error, DocumentFailed, PATH_ENCODING_FIELDS),
    FormatUnknown => ("FORMAT_UNKNOWN", Document, Error, DocumentFailed, FORMAT_UNKNOWN_FIELDS),
    FormatAmbiguous => ("FORMAT_AMBIGUOUS", Document, Error, DocumentFailed, FORMAT_AMBIGUOUS_FIELDS),
    CapabilityUnsupported => ("CAPABILITY_UNSUPPORTED", Request, Error, InputRejected, CAPABILITY_ADAPTER_FIELDS),
    RefNotFound => ("REF_NOT_FOUND", Document, Error, DocumentFailed, REF_FIELDS),
    RefAmbiguous => ("REF_AMBIGUOUS", Document, Error, DocumentFailed, REF_CANDIDATE_FIELDS),
    RefInvalid => ("REF_INVALID", Document, Error, DocumentFailed, REF_REASON_FIELDS),
    AdapterUnavailable => ("ADAPTER_UNAVAILABLE", AdapterBoundary, Error, AdapterBoundaryFailed, ADAPTER_REASON_FIELDS),
    AdapterInvokeFailed => ("ADAPTER_INVOKE_FAILED", AdapterBoundary, Error, AdapterBoundaryFailed, ADAPTER_REASON_FIELDS),
    InternalError => ("INTERNAL_ERROR", Internal, Fatal, InternalFailure, INTERNAL_FIELDS),
];

pub(super) const READABLE_WARNING_RULES: [ReadableWarningDiagnosticRule; 3] = readable_warning_rules![
    CliArgvIgnored => ("cli_argv_ignored", OperationContinued, CLI_ARGV_FIELDS),
    AdapterCandidateFailure => ("adapter_candidate_failure", CandidateSkipped, ADAPTER_CANDIDATE_FIELDS),
    AdapterConfigSourceSkipped => ("adapter_config_source_skipped", OperationContinued, ADAPTER_CONFIG_FIELDS),
];

pub(super) const BOUNDARY_RULES: [BoundaryDiagnosticRule; 19] = boundary_rules![
    AdapterErrorExitCodeCannotBe => ("adapter_error_exit_code_cannot_be", Internal, Error, InternalFailure),
    FailedToReadRequest => ("failed_to_read_request", AdapterBoundary, Error, AdapterBoundaryFailed),
    FailedToSerialize => ("failed_to_serialize", Internal, Fatal, InternalFailure),
    FailedToWriteCliWarning => ("failed_to_write_cli_warning", Internal, Fatal, InternalFailure),
    FailedToWriteJson => ("failed_to_write_json", Internal, Fatal, InternalFailure),
    FailedToWriteProtocolResponse => ("failed_to_write_protocol_response", Internal, Fatal, InternalFailure),
    FailedToWriteReadableView => ("failed_to_write_readable_view", Internal, Fatal, InternalFailure),
    InvalidRequestJson => ("invalid_request_json", Request, Error, InputRejected),
    ManifestAdapterIdMismatch => ("manifest_adapter_id_mismatch", AdapterBoundary, Error, AdapterBoundaryFailed),
    ManifestSchemaValidationFailed => ("manifest_schema_validation_failed", AdapterBoundary, Error, AdapterBoundaryFailed),
    ManifestSemanticValidationFailed => ("manifest_semantic_validation_failed", AdapterBoundary, Error, AdapterBoundaryFailed),
    ProbeResultAdapterIdMismatch => ("probe_result_adapter_id_mismatch", AdapterBoundary, Error, AdapterBoundaryFailed),
    ProbeResultSchemaValidationFailed => ("probe_result_schema_validation_failed", AdapterBoundary, Error, AdapterBoundaryFailed),
    ProbeResultSemanticValidationFailed => ("probe_result_semantic_validation_failed", AdapterBoundary, Error, AdapterBoundaryFailed),
    ProtocolResponseSchemaValidationFailed => ("protocol_response_schema_validation_failed", AdapterBoundary, Error, AdapterBoundaryFailed),
    ProtocolResponseSemanticValidationFailed => ("protocol_response_semantic_validation_failed", AdapterBoundary, Error, AdapterBoundaryFailed),
    ReadableViewRenderFailed => ("readable_view_render_failed", Internal, Fatal, InternalFailure),
    RequestDeserializationFailed => ("request_deserialization_failed", Request, Error, InputRejected),
    RequestSchemaValidationFailed => ("request_schema_validation_failed", Request, Error, InputRejected),
];

#[cfg(test)]
mod tests {
    use super::{BOUNDARY_RULES, PROTOCOL_RULES, READABLE_WARNING_RULES};
    use crate::code::{
        BoundaryDiagnosticCode, ProtocolDiagnosticCode, ReadableWarningDiagnosticCode,
    };

    #[test]
    fn diagnostic_rule_tables_follow_enum_order() {
        assert_eq!(
            PROTOCOL_RULES.len(),
            ProtocolDiagnosticCode::InternalError as usize + 1
        );
        for (index, rule) in PROTOCOL_RULES.iter().enumerate() {
            assert_eq!(rule.code as usize, index, "{:?}", rule.code);
        }

        assert_eq!(
            READABLE_WARNING_RULES.len(),
            ReadableWarningDiagnosticCode::AdapterConfigSourceSkipped as usize + 1
        );
        for (index, rule) in READABLE_WARNING_RULES.iter().enumerate() {
            assert_eq!(rule.code as usize, index, "{:?}", rule.code);
        }

        assert_eq!(
            BOUNDARY_RULES.len(),
            BoundaryDiagnosticCode::RequestSchemaValidationFailed as usize + 1
        );
        for (index, rule) in BOUNDARY_RULES.iter().enumerate() {
            assert_eq!(rule.code as usize, index, "{:?}", rule.code);
        }
    }
}
