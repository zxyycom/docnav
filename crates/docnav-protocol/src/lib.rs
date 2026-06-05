use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use std::cmp::{max, min};
use std::collections::BTreeMap;
use std::fmt;
use std::num::NonZeroU32;
use std::str::FromStr;

pub const PROTOCOL_VERSION: &str = "0.1";
pub const MANIFEST_VERSION: &str = "0.1";
pub const PROBE_VERSION: &str = "0.1";
pub const UNKNOWN_REQUEST_ID: &str = "unknown";

pub type PositiveInteger = NonZeroU32;
pub type Options = Map<String, Value>;
pub type ErrorDetails = BTreeMap<String, Value>;

const PROTOCOL_REQUEST_SCHEMA: &str =
    include_str!("../../../docs/schemas/protocol-request.schema.json");
const PROTOCOL_RESPONSE_SCHEMA: &str =
    include_str!("../../../docs/schemas/protocol-response.schema.json");
const MANIFEST_SCHEMA: &str = include_str!("../../../docs/schemas/manifest.schema.json");
const PROBE_RESULT_SCHEMA: &str = include_str!("../../../docs/schemas/probe-result.schema.json");

pub fn validate_protocol_request_value(value: &Value) -> Result<(), SchemaValidationError> {
    validate_value_with_schema(
        "protocol-request.schema.json",
        PROTOCOL_REQUEST_SCHEMA,
        value,
    )
}

pub fn validate_protocol_response_value(value: &Value) -> Result<(), SchemaValidationError> {
    validate_value_with_schema(
        "protocol-response.schema.json",
        PROTOCOL_RESPONSE_SCHEMA,
        value,
    )
}

pub fn validate_manifest_value(value: &Value) -> Result<(), SchemaValidationError> {
    validate_value_with_schema("manifest.schema.json", MANIFEST_SCHEMA, value)
}

pub fn validate_probe_result_value(value: &Value) -> Result<(), SchemaValidationError> {
    validate_value_with_schema("probe-result.schema.json", PROBE_RESULT_SCHEMA, value)
}

fn validate_value_with_schema(
    schema_name: &'static str,
    schema_source: &str,
    value: &Value,
) -> Result<(), SchemaValidationError> {
    let schema = serde_json::from_str::<Value>(schema_source).map_err(|error| {
        SchemaValidationError::compile(schema_name, format!("schema JSON parse failed: {error}"))
    })?;
    let validator = jsonschema::draft202012::options()
        .build(&schema)
        .map_err(|error| {
            SchemaValidationError::compile(schema_name, format!("schema compile failed: {error}"))
        })?;

    let errors = validator
        .iter_errors(value)
        .map(|error| {
            let path = error.instance_path().as_str();
            if path.is_empty() {
                error.to_string()
            } else {
                format!("{path}: {error}")
            }
        })
        .collect::<Vec<_>>();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(SchemaValidationError {
            schema: schema_name,
            errors,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaValidationError {
    pub schema: &'static str,
    pub errors: Vec<String>,
}

impl SchemaValidationError {
    fn compile(schema: &'static str, message: String) -> Self {
        Self {
            schema,
            errors: vec![message],
        }
    }
}

impl fmt::Display for SchemaValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} failed validation", self.schema)?;
        if !self.errors.is_empty() {
            write!(formatter, ": {}", self.errors.join("; "))?;
        }
        Ok(())
    }
}

impl std::error::Error for SchemaValidationError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ProtocolVersion {
    major: u32,
    minor: u32,
}

impl ProtocolVersion {
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    pub const fn major(self) -> u32 {
        self.major
    }

    pub const fn minor(self) -> u32 {
        self.minor
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for ProtocolVersion {
    type Err = VersionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (major, minor) = value
            .split_once('.')
            .ok_or_else(|| VersionParseError(value.to_owned()))?;

        if major.is_empty() || minor.is_empty() || minor.contains('.') {
            return Err(VersionParseError(value.to_owned()));
        }

        let major = major
            .parse::<u32>()
            .map_err(|_| VersionParseError(value.to_owned()))?;
        let minor = minor
            .parse::<u32>()
            .map_err(|_| VersionParseError(value.to_owned()))?;

        Ok(Self::new(major, minor))
    }
}

impl Serialize for ProtocolVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ProtocolVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionParseError(String);

impl fmt::Display for VersionParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid protocol version: {}", self.0)
    }
}

impl std::error::Error for VersionParseError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProtocolRange {
    pub min: ProtocolVersion,
    pub max: ProtocolVersion,
}

impl ProtocolRange {
    pub fn new(min: ProtocolVersion, max: ProtocolVersion) -> Result<Self, ProtocolRangeError> {
        if min > max {
            return Err(ProtocolRangeError { min, max });
        }
        Ok(Self { min, max })
    }

    pub const fn v0_1() -> Self {
        Self {
            min: ProtocolVersion::new(0, 1),
            max: ProtocolVersion::new(0, 1),
        }
    }

    pub fn contains(&self, version: ProtocolVersion) -> bool {
        self.min <= version && version <= self.max
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolRangeError {
    pub min: ProtocolVersion,
    pub max: ProtocolVersion,
}

impl fmt::Display for ProtocolRangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "protocol range min {} is greater than max {}",
            self.min, self.max
        )
    }
}

impl std::error::Error for ProtocolRangeError {}

pub fn select_highest_compatible(
    docnav: &ProtocolRange,
    adapter: &ProtocolRange,
) -> Result<ProtocolVersion, StableError> {
    let lower = max(docnav.min, adapter.min);
    let upper = min(docnav.max, adapter.max);

    if lower <= upper {
        Ok(upper)
    } else {
        Err(StableError::protocol_incompatible(
            format!("{}..{}", docnav.min, docnav.max),
            adapter.min.to_string(),
            adapter.max.to_string(),
        ))
    }
}

pub fn ensure_supported_protocol(
    requested: &str,
    supported: &ProtocolRange,
) -> Result<ProtocolVersion, StableError> {
    let requested_version = requested.parse::<ProtocolVersion>().map_err(|_| {
        StableError::invalid_request(
            "protocol_version",
            format!("invalid protocol version {requested:?}"),
        )
    })?;

    if supported.contains(requested_version) {
        Ok(requested_version)
    } else {
        Err(StableError::protocol_incompatible(
            requested,
            supported.min.to_string(),
            supported.max.to_string(),
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Outline,
    Read,
    Find,
    Info,
}

impl Operation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Outline => "outline",
            Self::Read => "read",
            Self::Find => "find",
            Self::Info => "info",
        }
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Operation {
    type Err = OperationParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "outline" => Ok(Self::Outline),
            "read" => Ok(Self::Read),
            "find" => Ok(Self::Find),
            "info" => Ok(Self::Info),
            _ => Err(OperationParseError(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationParseError(String);

impl fmt::Display for OperationParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid operation: {}", self.0)
    }
}

impl std::error::Error for OperationParseError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PagedOperation {
    Outline,
    Read,
    Find,
}

impl PagedOperation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Outline => "outline",
            Self::Read => "read",
            Self::Find => "find",
        }
    }
}

impl fmt::Display for PagedOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<PagedOperation> for Operation {
    fn from(operation: PagedOperation) -> Self {
        match operation {
            PagedOperation::Outline => Self::Outline,
            PagedOperation::Read => Self::Read,
            PagedOperation::Find => Self::Find,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestEnvelope {
    pub protocol_version: String,
    pub request_id: String,
    pub operation: Operation,
    pub document: Document,
    pub arguments: OperationArguments,
}

impl RequestEnvelope {
    pub fn operation_arguments(&self) -> Result<&OperationArguments, StableError> {
        if self.arguments.operation() == self.operation {
            Ok(&self.arguments)
        } else {
            Err(StableError::invalid_request(
                "arguments",
                format!("arguments do not match operation {}", self.operation),
            ))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OperationArguments {
    Outline(OutlineArguments),
    Read(ReadArguments),
    Find(FindArguments),
    Info(InfoArguments),
}

impl OperationArguments {
    pub const fn operation(&self) -> Operation {
        match self {
            Self::Outline(_) => Operation::Outline,
            Self::Read(_) => Operation::Read,
            Self::Find(_) => Operation::Find,
            Self::Info(_) => Operation::Info,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutlineArguments {
    pub limit_chars: PositiveInteger,
    pub page: PositiveInteger,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Options>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadArguments {
    #[serde(rename = "ref")]
    pub ref_id: String,
    pub limit_chars: PositiveInteger,
    pub page: PositiveInteger,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Options>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindArguments {
    pub query: String,
    pub limit_chars: PositiveInteger,
    pub page: PositiveInteger,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Options>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoArguments {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Options>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProtocolResponse {
    Success(SuccessResponse),
    Failure(FailureResponse),
}

impl ProtocolResponse {
    pub fn success(
        protocol_version: impl Into<String>,
        request_id: impl Into<String>,
        result: OperationResult,
    ) -> Self {
        Self::Success(SuccessResponse::new(protocol_version, request_id, result))
    }

    pub fn failure(
        protocol_version: impl Into<String>,
        request_id: impl Into<String>,
        operation: Option<Operation>,
        error: StableError,
    ) -> Self {
        Self::Failure(FailureResponse::new(
            protocol_version,
            request_id,
            operation,
            error,
        ))
    }

    pub fn failure_for_request(request: &RequestEnvelope, error: StableError) -> Self {
        Self::failure(
            request.protocol_version.clone(),
            request.request_id.clone(),
            Some(request.operation),
            error,
        )
    }

    pub fn validate(&self) -> Result<(), ProtocolValidationError> {
        match self {
            Self::Success(response) => response.validate(),
            Self::Failure(response) => response.validate(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessResponse {
    pub protocol_version: String,
    pub request_id: String,
    pub operation: Operation,
    pub ok: bool,
    pub result: OperationResult,
}

impl SuccessResponse {
    pub fn new(
        protocol_version: impl Into<String>,
        request_id: impl Into<String>,
        result: OperationResult,
    ) -> Self {
        Self {
            protocol_version: protocol_version.into(),
            request_id: request_id.into(),
            operation: result.operation(),
            ok: true,
            result,
        }
    }

    pub fn validate(&self) -> Result<(), ProtocolValidationError> {
        if !self.ok {
            return Err(ProtocolValidationError::InvalidOkFlag);
        }
        if self.operation != self.result.operation() {
            return Err(ProtocolValidationError::ResultOperationMismatch {
                operation: self.operation,
                result_operation: self.result.operation(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureResponse {
    pub protocol_version: String,
    pub request_id: String,
    pub operation: Option<Operation>,
    pub ok: bool,
    pub error: StableError,
}

impl FailureResponse {
    pub fn new(
        protocol_version: impl Into<String>,
        request_id: impl Into<String>,
        operation: Option<Operation>,
        error: StableError,
    ) -> Self {
        Self {
            protocol_version: protocol_version.into(),
            request_id: request_id.into(),
            operation,
            ok: false,
            error,
        }
    }

    pub fn unparsed(error: StableError, supported: &ProtocolRange) -> Self {
        Self::new(supported.max.to_string(), UNKNOWN_REQUEST_ID, None, error)
    }

    pub fn validate(&self) -> Result<(), ProtocolValidationError> {
        if self.ok {
            return Err(ProtocolValidationError::InvalidOkFlag);
        }
        self.error
            .validate_required_details()
            .map_err(ProtocolValidationError::MissingErrorDetail)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolValidationError {
    InvalidOkFlag,
    ResultOperationMismatch {
        operation: Operation,
        result_operation: Operation,
    },
    MissingErrorDetail(MissingErrorDetail),
}

impl fmt::Display for ProtocolValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOkFlag => formatter.write_str("response ok flag does not match variant"),
            Self::ResultOperationMismatch {
                operation,
                result_operation,
            } => write!(
                formatter,
                "success response operation {operation} does not match result {result_operation}"
            ),
            Self::MissingErrorDetail(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ProtocolValidationError {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OperationResult {
    Outline(OutlineResult),
    Read(ReadResult),
    Find(FindResult),
    Info(InfoResult),
}

impl OperationResult {
    pub const fn operation(&self) -> Operation {
        match self {
            Self::Outline(_) => Operation::Outline,
            Self::Read(_) => Operation::Read,
            Self::Find(_) => Operation::Find,
            Self::Info(_) => Operation::Info,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(rename = "ref")]
    pub ref_id: String,
    pub display: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutlineResult {
    pub entries: Vec<Entry>,
    pub page: Option<PositiveInteger>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadResult {
    #[serde(rename = "ref")]
    pub ref_id: String,
    pub content: String,
    pub content_type: String,
    pub cost: String,
    pub page: Option<PositiveInteger>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindResult {
    pub matches: Vec<Entry>,
    pub page: Option<PositiveInteger>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfoResult {
    pub display: String,
    pub capabilities: Vec<Operation>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StableError {
    pub code: StableErrorCode,
    pub message: String,
    pub details: ErrorDetails,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guidance: Option<Vec<String>>,
}

impl StableError {
    pub fn new(code: StableErrorCode, message: impl Into<String>, details: ErrorDetails) -> Self {
        Self {
            code,
            message: message.into(),
            details,
            guidance: None,
        }
    }

    pub fn with_guidance(mut self, guidance: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.guidance = Some(guidance.into_iter().map(Into::into).collect());
        self
    }

    pub fn invalid_request(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::InvalidRequest,
            "Invalid protocol request.",
            details([("field", field.into()), ("reason", reason.into())]),
        )
    }

    pub fn protocol_incompatible(
        requested: impl Into<String>,
        supported_min: impl Into<String>,
        supported_max: impl Into<String>,
    ) -> Self {
        Self::new(
            StableErrorCode::ProtocolIncompatible,
            "Protocol versions are incompatible.",
            details([
                ("requested", requested.into()),
                ("supported_min", supported_min.into()),
                ("supported_max", supported_max.into()),
            ]),
        )
    }

    pub fn document_not_found(path: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::DocumentNotFound,
            "Document was not found.",
            details([("path", path.into())]),
        )
    }

    pub fn document_path_invalid(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::DocumentPathInvalid,
            "Document path is invalid.",
            details([("path", path.into()), ("reason", reason.into())]),
        )
    }

    pub fn document_encoding_unsupported(
        path: impl Into<String>,
        encoding: impl Into<String>,
    ) -> Self {
        Self::new(
            StableErrorCode::DocumentEncodingUnsupported,
            "Document encoding is unsupported.",
            details([("path", path.into()), ("encoding", encoding.into())]),
        )
    }

    pub fn format_unknown(
        path: impl Into<String>,
        reason: impl Into<String>,
        candidates: Value,
    ) -> Self {
        let mut details = details([("path", path.into()), ("reason", reason.into())]);
        details.insert("candidates".to_owned(), candidates);
        Self::new(
            StableErrorCode::FormatUnknown,
            "Document format is unknown.",
            details,
        )
    }

    pub fn format_ambiguous(path: impl Into<String>, candidates: Value) -> Self {
        let mut details = details([("path", path.into())]);
        details.insert("candidates".to_owned(), candidates);
        Self::new(
            StableErrorCode::FormatAmbiguous,
            "Document format is ambiguous.",
            details,
        )
    }

    pub fn capability_unsupported(capability: Operation, adapter_id: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::CapabilityUnsupported,
            "Adapter does not support the requested capability.",
            details([
                ("capability", capability.to_string()),
                ("adapter_id", adapter_id.into()),
            ]),
        )
    }

    pub fn ref_not_found(ref_id: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::RefNotFound,
            "Ref was not found.",
            details([("ref", ref_id.into())]),
        )
    }

    pub fn ref_ambiguous(ref_id: impl Into<String>, candidate_count: u32) -> Self {
        let mut details = details([("ref", ref_id.into())]);
        details.insert("candidate_count".to_owned(), Value::from(candidate_count));
        Self::new(StableErrorCode::RefAmbiguous, "Ref is ambiguous.", details)
    }

    pub fn adapter_unavailable(adapter_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::AdapterUnavailable,
            "Adapter is unavailable.",
            details([("adapter_id", adapter_id.into()), ("reason", reason.into())]),
        )
    }

    pub fn adapter_invoke_failed(adapter_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::AdapterInvokeFailed,
            "Adapter invoke failed.",
            details([("adapter_id", adapter_id.into()), ("reason", reason.into())]),
        )
    }

    pub fn internal_error(error_id: impl Into<String>) -> Self {
        Self::new(
            StableErrorCode::InternalError,
            "Internal error.",
            details([("error_id", error_id.into())]),
        )
    }

    pub fn validate_required_details(&self) -> Result<(), MissingErrorDetail> {
        for &field in self.code.required_details() {
            if !self.details.contains_key(field) {
                return Err(MissingErrorDetail {
                    code: self.code,
                    field,
                });
            }
        }
        Ok(())
    }
}

fn details(fields: impl IntoIterator<Item = (&'static str, String)>) -> ErrorDetails {
    fields
        .into_iter()
        .map(|(key, value)| (key.to_owned(), Value::String(value)))
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StableErrorCode {
    InvalidRequest,
    ProtocolIncompatible,
    DocumentNotFound,
    DocumentPathInvalid,
    DocumentEncodingUnsupported,
    FormatUnknown,
    FormatAmbiguous,
    CapabilityUnsupported,
    RefNotFound,
    RefAmbiguous,
    AdapterUnavailable,
    AdapterInvokeFailed,
    InternalError,
}

impl StableErrorCode {
    pub const fn required_details(self) -> &'static [&'static str] {
        match self {
            Self::InvalidRequest => &["field", "reason"],
            Self::ProtocolIncompatible => &["requested", "supported_min", "supported_max"],
            Self::DocumentNotFound => &["path"],
            Self::DocumentPathInvalid => &["path", "reason"],
            Self::DocumentEncodingUnsupported => &["path", "encoding"],
            Self::FormatUnknown => &["path", "reason", "candidates"],
            Self::FormatAmbiguous => &["path", "candidates"],
            Self::CapabilityUnsupported => &["capability", "adapter_id"],
            Self::RefNotFound => &["ref"],
            Self::RefAmbiguous => &["ref", "candidate_count"],
            Self::AdapterUnavailable => &["adapter_id", "reason"],
            Self::AdapterInvokeFailed => &["adapter_id", "reason"],
            Self::InternalError => &["error_id"],
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissingErrorDetail {
    pub code: StableErrorCode,
    pub field: &'static str,
}

impl fmt::Display for MissingErrorDetail {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "error {:?} is missing required details.{}",
            self.code, self.field
        )
    }
}

impl std::error::Error for MissingErrorDetail {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub manifest_version: String,
    pub adapter: AdapterIdentity,
    pub protocol: ProtocolRange,
    pub formats: Vec<FormatDescriptor>,
    pub capabilities: Vec<Operation>,
    pub recommended_parameters: BTreeMap<PagedOperation, RecommendedParameters>,
}

impl Manifest {
    pub fn validate_semantics(&self) -> Result<(), ManifestValidationError> {
        if self.protocol.min > self.protocol.max {
            return Err(ManifestValidationError::InvalidProtocolRange {
                min: self.protocol.min,
                max: self.protocol.max,
            });
        }

        for operation in self.recommended_parameters.keys() {
            let capability = Operation::from(*operation);
            if !self.capabilities.contains(&capability) {
                return Err(
                    ManifestValidationError::RecommendedParameterWithoutCapability {
                        operation: *operation,
                    },
                );
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManifestValidationError {
    InvalidProtocolRange {
        min: ProtocolVersion,
        max: ProtocolVersion,
    },
    RecommendedParameterWithoutCapability {
        operation: PagedOperation,
    },
}

impl fmt::Display for ManifestValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProtocolRange { min, max } => write!(
                formatter,
                "manifest protocol range min {min} is greater than max {max}"
            ),
            Self::RecommendedParameterWithoutCapability { operation } => write!(
                formatter,
                "manifest recommended_parameters.{operation} is declared without matching capability"
            ),
        }
    }
}

impl std::error::Error for ManifestValidationError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterIdentity {
    pub id: String,
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormatDescriptor {
    pub id: String,
    pub extensions: Vec<String>,
    pub content_types: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecommendedParameters {
    pub limit_chars: PositiveInteger,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Options>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeResult {
    pub probe_version: String,
    pub adapter_id: String,
    pub path: String,
    pub supported: bool,
    pub format: Option<String>,
    pub confidence: f64,
    pub reasons: Vec<ProbeReason>,
}

impl ProbeResult {
    pub fn validate_semantics(&self) -> Result<(), ProbeValidationError> {
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(ProbeValidationError::ConfidenceOutOfRange(self.confidence));
        }
        if self.reasons.is_empty() {
            return Err(ProbeValidationError::MissingReasons);
        }
        if self.supported && self.format.is_none() {
            return Err(ProbeValidationError::SupportedWithoutFormat);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProbeValidationError {
    ConfidenceOutOfRange(f64),
    MissingReasons,
    SupportedWithoutFormat,
}

impl fmt::Display for ProbeValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfidenceOutOfRange(confidence) => {
                write!(formatter, "probe confidence {confidence} is outside 0..1")
            }
            Self::MissingReasons => formatter.write_str("probe reasons must not be empty"),
            Self::SupportedWithoutFormat => {
                formatter.write_str("probe supported=true requires a format")
            }
        }
    }
}

impl std::error::Error for ProbeValidationError {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeReason {
    pub code: ProbeReasonCode,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProbeReasonCode {
    ExtensionMatch,
    ContentMatch,
    ContentConflict,
    ReadError,
}

pub fn positive(value: u32) -> PositiveInteger {
    NonZeroU32::new(value).expect("positive integer must be non-zero")
}

pub fn extract_request_context(input: &str) -> PartialRequestContext {
    let Ok(value) = serde_json::from_str::<Value>(input) else {
        return PartialRequestContext::default();
    };

    extract_request_context_from_value(&value)
}

pub fn extract_request_context_from_value(value: &Value) -> PartialRequestContext {
    PartialRequestContext {
        protocol_version: value
            .get("protocol_version")
            .and_then(Value::as_str)
            .map(str::to_owned),
        request_id: value
            .get("request_id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_owned),
        operation: value
            .get("operation")
            .and_then(Value::as_str)
            .and_then(|value| value.parse::<Operation>().ok()),
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PartialRequestContext {
    pub protocol_version: Option<String>,
    pub request_id: Option<String>,
    pub operation: Option<Operation>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("examples")
            .join("json")
            .join(name)
    }

    fn read_fixture(name: &str) -> String {
        std::fs::read_to_string(fixture_path(name)).expect("fixture should be readable")
    }

    fn read_json_fixture(name: &str) -> Value {
        serde_json::from_str(&read_fixture(name)).expect("fixture is JSON")
    }

    #[test]
    fn constructs_outline_success_response() {
        let response = ProtocolResponse::success(
            PROTOCOL_VERSION,
            "req-outline-001",
            OperationResult::Outline(OutlineResult {
                entries: vec![Entry {
                    ref_id: "L1:Guide".to_owned(),
                    display: "9 lines | 0.1 KB".to_owned(),
                }],
                page: Some(positive(2)),
            }),
        );

        let value = serde_json::to_value(response).expect("response serializes");
        assert_eq!(value["protocol_version"], PROTOCOL_VERSION);
        assert_eq!(value["request_id"], "req-outline-001");
        assert_eq!(value["operation"], "outline");
        assert_eq!(value["ok"], true);
        assert_eq!(value["result"]["entries"][0]["ref"], "L1:Guide");
        assert_eq!(value["result"]["page"], 2);
        assert!(value["result"].get("markdown_heading_path").is_none());
    }

    #[test]
    fn selects_highest_compatible_protocol_version() {
        let docnav = ProtocolRange::new(ProtocolVersion::new(0, 1), ProtocolVersion::new(0, 2))
            .expect("valid range");
        let adapter = ProtocolRange::v0_1();

        let selected = select_highest_compatible(&docnav, &adapter).expect("compatible");

        assert_eq!(selected, ProtocolVersion::new(0, 1));
    }

    #[test]
    fn incompatible_protocol_range_builds_stable_error() {
        let docnav = ProtocolRange::new(ProtocolVersion::new(1, 0), ProtocolVersion::new(1, 1))
            .expect("valid range");
        let adapter = ProtocolRange::v0_1();

        let error = select_highest_compatible(&docnav, &adapter).expect_err("incompatible");

        assert_eq!(error.code, StableErrorCode::ProtocolIncompatible);
        assert_eq!(error.details["requested"], "1.0..1.1");
        assert_eq!(error.details["supported_min"], "0.1");
        assert_eq!(error.details["supported_max"], "0.1");
        error.validate_required_details().expect("stable details");
    }

    #[test]
    fn failure_response_rules_preserve_or_null_operation() {
        let request: RequestEnvelope =
            serde_json::from_str(&read_fixture("protocol-read-request.json"))
                .expect("request parses");
        let request_failure =
            ProtocolResponse::failure_for_request(&request, StableError::ref_not_found("missing"));

        match request_failure {
            ProtocolResponse::Failure(response) => {
                assert_eq!(response.operation, Some(Operation::Read));
                response.validate().expect("failure validates");
            }
            ProtocolResponse::Success(_) => panic!("expected failure"),
        }

        let unparsed = FailureResponse::unparsed(
            StableError::invalid_request("request", "not json"),
            &ProtocolRange::v0_1(),
        );
        assert_eq!(unparsed.operation, None);
        unparsed.validate().expect("unparsed failure validates");
    }

    #[test]
    fn parses_protocol_fixtures_into_shared_types() {
        for operation in ["outline", "read", "find", "info"] {
            let request_value = read_json_fixture(&format!("protocol-{operation}-request.json"));
            validate_protocol_request_value(&request_value).expect("request fixture schema");
            let request: RequestEnvelope =
                serde_json::from_value(request_value).expect("request fixture parses");
            request
                .operation_arguments()
                .expect("arguments match operation");

            let response_value = read_json_fixture(&format!("protocol-{operation}-response.json"));
            validate_protocol_response_value(&response_value).expect("response fixture schema");
            let response: ProtocolResponse =
                serde_json::from_value(response_value).expect("response fixture parses");
            response.validate().expect("response validates");
        }

        let manifest_value = read_json_fixture("manifest.json");
        validate_manifest_value(&manifest_value).expect("manifest fixture schema");
        let manifest: Manifest =
            serde_json::from_value(manifest_value).expect("manifest fixture parses");
        assert_eq!(manifest.protocol, ProtocolRange::v0_1());
        manifest
            .validate_semantics()
            .expect("manifest fixture semantics");

        let probe_value = read_json_fixture("probe-result.json");
        validate_probe_result_value(&probe_value).expect("probe fixture schema");
        let probe: ProbeResult = serde_json::from_value(probe_value).expect("probe fixture parses");
        assert_eq!(probe.probe_version, PROBE_VERSION);
        probe.validate_semantics().expect("probe fixture semantics");
    }

    #[test]
    fn protocol_request_schema_rejects_empty_required_strings() {
        let cases = [
            serde_json::json!({
                "protocol_version": "0.1",
                "request_id": "",
                "operation": "outline",
                "document": { "path": "doc.md" },
                "arguments": { "limit_chars": 80, "page": 1 }
            }),
            serde_json::json!({
                "protocol_version": "0.1",
                "request_id": "req-1",
                "operation": "outline",
                "document": { "path": "" },
                "arguments": { "limit_chars": 80, "page": 1 }
            }),
            serde_json::json!({
                "protocol_version": "0.1",
                "request_id": "req-1",
                "operation": "read",
                "document": { "path": "doc.md" },
                "arguments": { "ref": "", "limit_chars": 80, "page": 1 }
            }),
            serde_json::json!({
                "protocol_version": "0.1",
                "request_id": "req-1",
                "operation": "find",
                "document": { "path": "doc.md" },
                "arguments": { "query": "", "limit_chars": 80, "page": 1 }
            }),
        ];

        for value in cases {
            assert!(validate_protocol_request_value(&value).is_err());
        }
    }

    #[test]
    fn manifest_schema_rejects_info_recommended_parameters() {
        let value = serde_json::json!({
            "manifest_version": "0.1",
            "adapter": {
                "id": "stub",
                "name": "Stub",
                "version": "0.1.0"
            },
            "protocol": {
                "min": "0.1",
                "max": "0.1"
            },
            "formats": [
                {
                    "id": "stub",
                    "extensions": [".stub"],
                    "content_types": ["text/stub"]
                }
            ],
            "capabilities": ["outline", "read", "find", "info"],
            "recommended_parameters": {
                "info": {
                    "limit_chars": 80
                }
            }
        });

        assert!(validate_manifest_value(&value).is_err());
        assert!(serde_json::from_value::<Manifest>(value).is_err());
    }

    #[test]
    fn probe_schema_rejects_missing_reasons_and_bad_confidence() {
        let missing_reasons = serde_json::json!({
            "probe_version": "0.1",
            "adapter_id": "stub",
            "path": "doc.stub",
            "supported": true,
            "format": "stub",
            "confidence": 1.0,
            "reasons": []
        });
        let bad_confidence = serde_json::json!({
            "probe_version": "0.1",
            "adapter_id": "stub",
            "path": "doc.stub",
            "supported": true,
            "format": "stub",
            "confidence": 1.5,
            "reasons": [
                { "code": "EXTENSION_MATCH", "detail": "extension matched" }
            ]
        });

        assert!(validate_probe_result_value(&missing_reasons).is_err());
        assert!(validate_probe_result_value(&bad_confidence).is_err());

        let probe: ProbeResult = serde_json::from_value(missing_reasons).expect("shape parses");
        assert_eq!(
            probe.validate_semantics(),
            Err(ProbeValidationError::MissingReasons)
        );
        let probe: ProbeResult = serde_json::from_value(bad_confidence).expect("shape parses");
        assert_eq!(
            probe.validate_semantics(),
            Err(ProbeValidationError::ConfidenceOutOfRange(1.5))
        );
    }

    #[test]
    fn manifest_semantics_reject_invalid_protocol_range() {
        let manifest = Manifest {
            manifest_version: MANIFEST_VERSION.to_owned(),
            adapter: AdapterIdentity {
                id: "stub".to_owned(),
                name: "Stub".to_owned(),
                version: "0.1.0".to_owned(),
            },
            protocol: ProtocolRange {
                min: ProtocolVersion::new(0, 2),
                max: ProtocolVersion::new(0, 1),
            },
            formats: vec![FormatDescriptor {
                id: "stub".to_owned(),
                extensions: vec![".stub".to_owned()],
                content_types: vec!["text/stub".to_owned()],
            }],
            capabilities: vec![Operation::Outline],
            recommended_parameters: BTreeMap::new(),
        };

        assert_eq!(
            manifest.validate_semantics(),
            Err(ManifestValidationError::InvalidProtocolRange {
                min: ProtocolVersion::new(0, 2),
                max: ProtocolVersion::new(0, 1),
            })
        );
    }
}
