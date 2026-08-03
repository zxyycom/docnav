use super::{
    AdapterUnavailableDetails, BoundaryDetails, DiagnosticDetails, DocumentContentInvalidDetails,
    FieldReasonDetails, FormatUnknownDetails, InternalDetails, PathDetails, PathEncodingDetails,
    PathReasonDetails, RefCandidateCountDetails, RefDetails, RefReasonDetails,
};

impl From<FieldReasonDetails> for DiagnosticDetails {
    fn from(details: FieldReasonDetails) -> Self {
        Self::FieldReason {
            field: details.field,
            reason: details.reason,
            path: details.path,
            received: details.received,
            accepted: details.accepted,
            field_issues: details.field_issues,
            config_issues: details.config_issues,
            typed_validation_failures: details.typed_validation_failures,
            option_issues: details.option_issues,
        }
    }
}

impl From<PathDetails> for DiagnosticDetails {
    fn from(details: PathDetails) -> Self {
        Self::Path { path: details.path }
    }
}

impl From<PathReasonDetails> for DiagnosticDetails {
    fn from(details: PathReasonDetails) -> Self {
        Self::PathReason {
            path: details.path,
            reason: details.reason,
        }
    }
}

impl From<PathEncodingDetails> for DiagnosticDetails {
    fn from(details: PathEncodingDetails) -> Self {
        Self::PathEncoding {
            path: details.path,
            encoding: details.encoding,
        }
    }
}

impl From<DocumentContentInvalidDetails> for DiagnosticDetails {
    fn from(details: DocumentContentInvalidDetails) -> Self {
        Self::DocumentContentInvalid {
            path: details.path,
            reason: details.reason,
        }
    }
}

impl From<FormatUnknownDetails> for DiagnosticDetails {
    fn from(details: FormatUnknownDetails) -> Self {
        Self::FormatUnknown {
            path: details.path,
            reason: details.reason,
            candidates: details.candidates,
        }
    }
}

impl From<RefDetails> for DiagnosticDetails {
    fn from(details: RefDetails) -> Self {
        Self::Ref {
            ref_id: details.ref_id,
        }
    }
}

impl From<RefCandidateCountDetails> for DiagnosticDetails {
    fn from(details: RefCandidateCountDetails) -> Self {
        Self::RefCandidateCount {
            ref_id: details.ref_id,
            candidate_count: details.candidate_count,
        }
    }
}

impl From<RefReasonDetails> for DiagnosticDetails {
    fn from(details: RefReasonDetails) -> Self {
        Self::RefReason {
            ref_id: details.ref_id,
            reason: details.reason,
        }
    }
}

impl From<AdapterUnavailableDetails> for DiagnosticDetails {
    fn from(details: AdapterUnavailableDetails) -> Self {
        let (adapter_id, reason, selection_source, stage) = details.into_parts();
        Self::AdapterUnavailable {
            adapter_id,
            reason,
            selection_source,
            stage,
        }
    }
}

impl From<InternalDetails> for DiagnosticDetails {
    fn from(details: InternalDetails) -> Self {
        Self::Internal {
            error_id: details.error_id,
        }
    }
}

impl From<BoundaryDetails> for DiagnosticDetails {
    fn from(details: BoundaryDetails) -> Self {
        Self::Boundary {
            reason: details.reason,
            label: details.label,
        }
    }
}
