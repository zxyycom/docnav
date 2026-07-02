use super::{
    AdapterReasonDetails, BoundaryDetails, CapabilityAdapterDetails, DiagnosticDetails,
    FieldReasonDetails, FormatAmbiguousDetails, FormatUnknownDetails, InternalDetails, PathDetails,
    PathEncodingDetails, PathReasonDetails, RefCandidateCountDetails, RefDetails, RefReasonDetails,
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

impl From<FormatUnknownDetails> for DiagnosticDetails {
    fn from(details: FormatUnknownDetails) -> Self {
        Self::FormatUnknown {
            path: details.path,
            reason: details.reason,
            candidates: details.candidates,
        }
    }
}

impl From<FormatAmbiguousDetails> for DiagnosticDetails {
    fn from(details: FormatAmbiguousDetails) -> Self {
        Self::FormatAmbiguous {
            path: details.path,
            candidates: details.candidates,
        }
    }
}

impl From<CapabilityAdapterDetails> for DiagnosticDetails {
    fn from(details: CapabilityAdapterDetails) -> Self {
        Self::CapabilityAdapter {
            capability: details.capability,
            adapter_id: details.adapter_id,
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

impl From<AdapterReasonDetails> for DiagnosticDetails {
    fn from(details: AdapterReasonDetails) -> Self {
        Self::AdapterReason {
            adapter_id: details.adapter_id,
            reason: details.reason,
            exit_code: details.exit_code,
            stderr: details.stderr,
            selection_source: details.selection_source,
            stage: details.stage,
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
