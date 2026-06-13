// 协议层常量集中在本模块：这里只放跨模块共享的稳定字段名、版本号和固定文案。
// 这些值来自主规范和 schema，代码只能引用它们，不能把本文件变成新的规则来源。
pub const PROTOCOL_VERSION: &str = "0.1";
pub const MANIFEST_VERSION: &str = "0.1";
pub const PROBE_VERSION: &str = "0.1";
pub const UNKNOWN_REQUEST_ID: &str = "unknown";

// 原始协议 envelope 的稳定字段名，供边界解析和错误定位复用。
pub(crate) mod fields {
    pub const PROTOCOL_VERSION: &str = "protocol_version";
    pub const REQUEST_ID: &str = "request_id";
    pub const OPERATION: &str = "operation";
    pub const ARGUMENTS: &str = "arguments";
}

// 稳定错误 details 的字段名，必须与 docs/protocol.md 和示例保持一致。
pub(crate) mod error_detail_fields {
    pub const ADAPTER_ID: &str = "adapter_id";
    pub const CANDIDATE_COUNT: &str = "candidate_count";
    pub const CANDIDATES: &str = "candidates";
    pub const CAPABILITY: &str = "capability";
    pub const ENCODING: &str = "encoding";
    pub const ERROR_ID: &str = "error_id";
    pub const FIELD: &str = "field";
    pub const PATH: &str = "path";
    pub const REASON: &str = "reason";
    pub const REF: &str = "ref";
}

// operation 的 wire 字符串，集中后避免 Display、FromStr 和脚本示例漂移。
pub(crate) mod operation_names {
    pub const FIND: &str = "find";
    pub const INFO: &str = "info";
    pub const OUTLINE: &str = "outline";
    pub const READ: &str = "read";
}

// schema 文件名仅用于错误报告；include_str! 的路径仍留在 schema.rs 作为编译期资源边界。
pub(crate) mod schema_names {
    pub const MANIFEST: &str = "manifest.schema.json";
    pub const PROBE_RESULT: &str = "probe-result.schema.json";
    pub const PROTOCOL_REQUEST: &str = "protocol-request.schema.json";
    pub const PROTOCOL_RESPONSE: &str = "protocol-response.schema.json";
}

// 稳定错误 message 的默认文案集中在这里；调用方只解析 code 和 details。
pub(crate) mod stable_error_messages {
    pub const ADAPTER_INVOKE_FAILED: &str = "Adapter invoke failed.";
    pub const ADAPTER_UNAVAILABLE: &str = "Adapter is unavailable.";
    pub const CAPABILITY_UNSUPPORTED: &str = "Adapter does not support the requested capability.";
    pub const DOCUMENT_ENCODING_UNSUPPORTED: &str = "Document encoding is unsupported.";
    pub const DOCUMENT_FORMAT_AMBIGUOUS: &str = "Document format is ambiguous.";
    pub const DOCUMENT_FORMAT_UNKNOWN: &str = "Document format is unknown.";
    pub const DOCUMENT_NOT_FOUND: &str = "Document was not found.";
    pub const DOCUMENT_PATH_INVALID: &str = "Document path is invalid.";
    pub const INTERNAL_ERROR: &str = "Internal error.";
    pub const INVALID_PROTOCOL_REQUEST: &str = "Invalid protocol request.";
    pub const REF_AMBIGUOUS: &str = "Ref is ambiguous.";
    pub const REF_INVALID: &str = "Ref grammar is invalid.";
    pub const REF_NOT_FOUND: &str = "Ref was not found.";
}
