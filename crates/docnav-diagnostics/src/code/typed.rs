use crate::details::{
    AdapterCandidateDetails, AdapterConfigSourceDetails, AdapterReasonDetails, BoundaryDetails,
    CapabilityAdapterDetails, CliArgvDetails, DiagnosticDetailsPayload, FieldReasonDetails,
    FormatAmbiguousDetails, FormatUnknownDetails, InternalDetails, PathDetails,
    PathEncodingDetails, PathReasonDetails, RefCandidateCountDetails, RefDetails, RefReasonDetails,
};

use super::{
    BoundaryDiagnosticCode, DiagnosticCode, ProtocolDiagnosticCode, ReadableWarningDiagnosticCode,
};

mod sealed {
    pub trait Sealed {}
}

pub trait DiagnosticCodeMarker: sealed::Sealed {
    type Details: DiagnosticDetailsPayload;

    const CODE: DiagnosticCode;
}

pub trait ProtocolDiagnosticMarker: DiagnosticCodeMarker {
    const PROTOCOL_CODE: ProtocolDiagnosticCode;
}

pub trait ReadableWarningDiagnosticMarker: DiagnosticCodeMarker {
    const WARNING_CODE: ReadableWarningDiagnosticCode;
}

pub trait BoundaryDiagnosticMarker: DiagnosticCodeMarker {
    const BOUNDARY_CODE: BoundaryDiagnosticCode;
}

pub mod typed_codes {
    pub mod protocol {
        pub struct InvalidRequest;
        pub struct DocumentNotFound;
        pub struct DocumentPathInvalid;
        pub struct DocumentEncodingUnsupported;
        pub struct FormatUnknown;
        pub struct FormatAmbiguous;
        pub struct CapabilityUnsupported;
        pub struct RefNotFound;
        pub struct RefAmbiguous;
        pub struct RefInvalid;
        pub struct AdapterUnavailable;
        pub struct AdapterInvokeFailed;
        pub struct InternalError;
    }

    pub mod readable_warning {
        pub struct CliArgvIgnored;
        pub struct AdapterCandidateFailure;
        pub struct AdapterConfigSourceSkipped;
    }

    pub mod boundary {
        pub struct AdapterErrorExitCodeCannotBe;
        pub struct FailedToReadRequest;
        pub struct FailedToSerialize;
        pub struct FailedToWriteCliWarning;
        pub struct FailedToWriteJson;
        pub struct FailedToWriteProtocolResponse;
        pub struct FailedToWriteReadableView;
        pub struct InvalidRequestJson;
        pub struct ManifestAdapterIdMismatch;
        pub struct ManifestSchemaValidationFailed;
        pub struct ManifestSemanticValidationFailed;
        pub struct ProbeResultAdapterIdMismatch;
        pub struct ProbeResultSchemaValidationFailed;
        pub struct ProbeResultSemanticValidationFailed;
        pub struct ProtocolResponseSchemaValidationFailed;
        pub struct ProtocolResponseSemanticValidationFailed;
        pub struct ReadableViewRenderFailed;
        pub struct RequestDeserializationFailed;
        pub struct RequestSchemaValidationFailed;
    }
}

macro_rules! protocol_marker {
    ($marker:path, $code:ident, $details:ty) => {
        impl sealed::Sealed for $marker {}

        impl DiagnosticCodeMarker for $marker {
            type Details = $details;

            const CODE: DiagnosticCode = DiagnosticCode::Protocol(ProtocolDiagnosticCode::$code);
        }

        impl ProtocolDiagnosticMarker for $marker {
            const PROTOCOL_CODE: ProtocolDiagnosticCode = ProtocolDiagnosticCode::$code;
        }
    };
}

macro_rules! readable_warning_marker {
    ($marker:path, $code:ident, $details:ty) => {
        impl sealed::Sealed for $marker {}

        impl DiagnosticCodeMarker for $marker {
            type Details = $details;

            const CODE: DiagnosticCode =
                DiagnosticCode::ReadableWarning(ReadableWarningDiagnosticCode::$code);
        }

        impl ReadableWarningDiagnosticMarker for $marker {
            const WARNING_CODE: ReadableWarningDiagnosticCode =
                ReadableWarningDiagnosticCode::$code;
        }
    };
}

macro_rules! boundary_marker {
    ($marker:path, $code:ident) => {
        impl sealed::Sealed for $marker {}

        impl DiagnosticCodeMarker for $marker {
            type Details = BoundaryDetails;

            const CODE: DiagnosticCode = DiagnosticCode::Boundary(BoundaryDiagnosticCode::$code);
        }

        impl BoundaryDiagnosticMarker for $marker {
            const BOUNDARY_CODE: BoundaryDiagnosticCode = BoundaryDiagnosticCode::$code;
        }
    };
}

protocol_marker!(
    typed_codes::protocol::InvalidRequest,
    InvalidRequest,
    FieldReasonDetails
);
protocol_marker!(
    typed_codes::protocol::DocumentNotFound,
    DocumentNotFound,
    PathDetails
);
protocol_marker!(
    typed_codes::protocol::DocumentPathInvalid,
    DocumentPathInvalid,
    PathReasonDetails
);
protocol_marker!(
    typed_codes::protocol::DocumentEncodingUnsupported,
    DocumentEncodingUnsupported,
    PathEncodingDetails
);
protocol_marker!(
    typed_codes::protocol::FormatUnknown,
    FormatUnknown,
    FormatUnknownDetails
);
protocol_marker!(
    typed_codes::protocol::FormatAmbiguous,
    FormatAmbiguous,
    FormatAmbiguousDetails
);
protocol_marker!(
    typed_codes::protocol::CapabilityUnsupported,
    CapabilityUnsupported,
    CapabilityAdapterDetails
);
protocol_marker!(typed_codes::protocol::RefNotFound, RefNotFound, RefDetails);
protocol_marker!(
    typed_codes::protocol::RefAmbiguous,
    RefAmbiguous,
    RefCandidateCountDetails
);
protocol_marker!(
    typed_codes::protocol::RefInvalid,
    RefInvalid,
    RefReasonDetails
);
protocol_marker!(
    typed_codes::protocol::AdapterUnavailable,
    AdapterUnavailable,
    AdapterReasonDetails
);
protocol_marker!(
    typed_codes::protocol::AdapterInvokeFailed,
    AdapterInvokeFailed,
    AdapterReasonDetails
);
protocol_marker!(
    typed_codes::protocol::InternalError,
    InternalError,
    InternalDetails
);

readable_warning_marker!(
    typed_codes::readable_warning::CliArgvIgnored,
    CliArgvIgnored,
    CliArgvDetails
);
readable_warning_marker!(
    typed_codes::readable_warning::AdapterCandidateFailure,
    AdapterCandidateFailure,
    AdapterCandidateDetails
);
readable_warning_marker!(
    typed_codes::readable_warning::AdapterConfigSourceSkipped,
    AdapterConfigSourceSkipped,
    AdapterConfigSourceDetails
);

boundary_marker!(
    typed_codes::boundary::AdapterErrorExitCodeCannotBe,
    AdapterErrorExitCodeCannotBe
);
boundary_marker!(
    typed_codes::boundary::FailedToReadRequest,
    FailedToReadRequest
);
boundary_marker!(typed_codes::boundary::FailedToSerialize, FailedToSerialize);
boundary_marker!(
    typed_codes::boundary::FailedToWriteCliWarning,
    FailedToWriteCliWarning
);
boundary_marker!(typed_codes::boundary::FailedToWriteJson, FailedToWriteJson);
boundary_marker!(
    typed_codes::boundary::FailedToWriteProtocolResponse,
    FailedToWriteProtocolResponse
);
boundary_marker!(
    typed_codes::boundary::FailedToWriteReadableView,
    FailedToWriteReadableView
);
boundary_marker!(
    typed_codes::boundary::InvalidRequestJson,
    InvalidRequestJson
);
boundary_marker!(
    typed_codes::boundary::ManifestAdapterIdMismatch,
    ManifestAdapterIdMismatch
);
boundary_marker!(
    typed_codes::boundary::ManifestSchemaValidationFailed,
    ManifestSchemaValidationFailed
);
boundary_marker!(
    typed_codes::boundary::ManifestSemanticValidationFailed,
    ManifestSemanticValidationFailed
);
boundary_marker!(
    typed_codes::boundary::ProbeResultAdapterIdMismatch,
    ProbeResultAdapterIdMismatch
);
boundary_marker!(
    typed_codes::boundary::ProbeResultSchemaValidationFailed,
    ProbeResultSchemaValidationFailed
);
boundary_marker!(
    typed_codes::boundary::ProbeResultSemanticValidationFailed,
    ProbeResultSemanticValidationFailed
);
boundary_marker!(
    typed_codes::boundary::ProtocolResponseSchemaValidationFailed,
    ProtocolResponseSchemaValidationFailed
);
boundary_marker!(
    typed_codes::boundary::ProtocolResponseSemanticValidationFailed,
    ProtocolResponseSemanticValidationFailed
);
boundary_marker!(
    typed_codes::boundary::ReadableViewRenderFailed,
    ReadableViewRenderFailed
);
boundary_marker!(
    typed_codes::boundary::RequestDeserializationFailed,
    RequestDeserializationFailed
);
boundary_marker!(
    typed_codes::boundary::RequestSchemaValidationFailed,
    RequestSchemaValidationFailed
);
