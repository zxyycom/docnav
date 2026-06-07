// SDK 边界常量集中在本模块：只放 adapter invoke/output 会复用的字段名、标签和诊断前缀。
// 协议稳定字段仍由 docnav-protocol 拥有，本模块不重新定义协议规则。
pub(crate) mod fields {
    pub const ARGUMENTS: &str = "arguments";
    pub const REQUEST: &str = "request";
}

// JSON 序列化标签用于 stderr 诊断，集中后避免输出层文案局部漂移。
pub(crate) mod json_labels {
    pub const MANIFEST: &str = "manifest";
    pub const PROBE_RESULT: &str = "probe result";
    pub const PROTOCOL_RESPONSE: &str = "protocol response";
}

// stderr 诊断前缀只服务人工排障，不改变 stdout 的机器协议字段。
pub(crate) mod diagnostics {
    pub const ADAPTER_ERROR_EXIT_CODE_CANNOT_BE: &str = "adapter error exit code cannot be";
    pub const FAILED_TO_READ_REQUEST: &str = "failed to read request";
    pub const FAILED_TO_SERIALIZE: &str = "failed to serialize";
    pub const FAILED_TO_WRITE_CLI_WARNING: &str = "failed to write CLI warning";
    pub const FAILED_TO_WRITE_JSON: &str = "failed to write JSON";
    pub const FAILED_TO_WRITE_PROTOCOL_RESPONSE: &str = "failed to write protocol response";
    pub const FAILED_TO_WRITE_TEXT_ERROR: &str = "failed to write text error";
    pub const FAILED_TO_WRITE_TEXT_OUTPUT: &str = "failed to write text output";
    pub const INVALID_REQUEST_JSON: &str = "invalid request JSON";
    pub const MANIFEST_ADAPTER_ID_MISMATCH: &str = "manifest adapter id mismatch";
    pub const MANIFEST_SCHEMA_VALIDATION_FAILED: &str = "manifest schema validation failed";
    pub const MANIFEST_SEMANTIC_VALIDATION_FAILED: &str = "manifest semantic validation failed";
    pub const PROBE_RESULT_ADAPTER_ID_MISMATCH: &str = "probe result adapter id mismatch";
    pub const PROBE_RESULT_SCHEMA_VALIDATION_FAILED: &str = "probe result schema validation failed";
    pub const PROBE_RESULT_SEMANTIC_VALIDATION_FAILED: &str =
        "probe result semantic validation failed";
    pub const PROTOCOL_RESPONSE_SCHEMA_VALIDATION_FAILED: &str =
        "protocol response schema validation failed";
    pub const PROTOCOL_RESPONSE_SEMANTIC_VALIDATION_FAILED: &str =
        "protocol response semantic validation failed";
    pub const REQUEST_DESERIALIZATION_FAILED: &str =
        "request deserialization failed after schema validation";
    pub const REQUEST_SCHEMA_VALIDATION_FAILED: &str = "request schema validation failed";
}
