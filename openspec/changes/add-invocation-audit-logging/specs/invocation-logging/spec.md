本 spec delta 只在 `openspec/changes/add-invocation-audit-logging/` 下形成未审核临时文档，目标是为 Docnav 核心调用链引入默认元数据级调用日志和可选协议追踪，为简单调用记录与后续审计改进提供基础。

## ADDED Requirements

### Requirement: 调用日志使用结构化 JSONL 事件

Docnav runtime invocation logging MUST write structured JSON Lines / NDJSON events when invocation logging is enabled. Each event MUST include a schema version, timestamp, event name, request id when available, operation, selected adapter id when available, success/failure status, duration, and bounded size/status metadata sufficient to correlate a core CLI document operation with the adapter invoke result.

#### Scenario: 成功调用写入可关联事件

- **WHEN** invocation logging is enabled and `docnav` completes an adapter-backed document operation successfully
- **THEN** the log contains at least one JSONL event with `schema_version`, timestamp, event name, `request_id`, operation, adapter id, success status, duration, exit code, and bounded response size metadata
- **THEN** the event can be parsed independently from other log lines without requiring the full log file to be valid JSON

#### Scenario: 失败调用写入失败边界

- **WHEN** invocation logging is enabled and adapter selection, adapter process startup, stdin write, stdout parse, protocol response validation, or operation execution fails
- **THEN** the log records the available request id or fallback correlation id
- **THEN** the log records the failure layer, stable error code when available, exit code when available, duration, and bounded diagnostic summary

### Requirement: 默认调用日志只记录元数据

Invocation logging MUST default to metadata-only records. Metadata-only records MUST NOT include document body content, full protocol request/response payloads, full adapter stderr, full environment variables, or unbounded raw user input.

#### Scenario: Read 响应不默认写入正文

- **WHEN** invocation logging is enabled in metadata-only mode for a `read` operation
- **THEN** the log records request/response correlation and bounded size/status metadata
- **THEN** the log does not record the full `content` field from the read result

#### Scenario: Find 查询和 ref 使用安全摘要

- **WHEN** invocation logging is enabled in metadata-only mode for `find` or `read`
- **THEN** query and ref fields are either omitted, represented by presence/length metadata, or represented by an explicitly bounded and documented summary
- **THEN** the log does not treat adapter-owned ref grammar or document text as a stable cross-layer identity

### Requirement: Raw protocol trace 必须显式开启并受限

Full protocol request/response tracing MUST be explicit opt-in. Trace records MUST apply field redaction, byte or character limits, and truncation markers before writing raw or near-raw protocol payloads.

#### Scenario: 未开启 trace 时不写完整 envelope

- **WHEN** invocation logging is enabled but raw protocol trace is not enabled
- **THEN** the log does not include the complete `RequestEnvelope`
- **THEN** the log does not include the complete `ProtocolResponse`

#### Scenario: 开启 trace 时执行截断和脱敏

- **WHEN** raw protocol trace is explicitly enabled
- **THEN** request and response payload records are written only after applying the configured size caps and redaction rules
- **THEN** truncated fields include a marker or metadata that makes truncation auditable

### Requirement: 调用日志不得污染 stdout 或协议输出

Invocation logging MUST be isolated from document output stdout and adapter protocol stdout. Logging MUST NOT add fields to `RequestEnvelope`, `ProtocolResponse`, readable output payloads, manifest output, or probe output.

#### Scenario: protocol-json stdout 保持纯净

- **WHEN** invocation logging is enabled and a caller runs a document operation with `--output protocol-json`
- **THEN** stdout contains only the protocol-shaped response for that operation
- **THEN** log events are written only to the configured log sink

#### Scenario: adapter invoke stdout 保持协议边界

- **WHEN** core CLI invokes an adapter subprocess
- **THEN** adapter stdout is still parsed only as adapter protocol output
- **THEN** runtime invocation logs are not written to adapter stdin or adapter stdout

### Requirement: 日志写入失败不改变文档操作语义

Invocation logging failures MUST NOT change the success or failure semantics of the document operation being logged. The original CLI exit behavior, output mode, and stable error mapping MUST remain determined by the document operation and adapter result, not by the logging subsystem.

#### Scenario: 日志文件不可写时主调用继续

- **WHEN** invocation logging is enabled but the configured log file cannot be created or appended
- **THEN** a document operation that would otherwise succeed still succeeds
- **THEN** stdout remains valid for the selected output mode
- **THEN** any logging failure diagnostic is bounded and emitted only through a channel that does not corrupt machine-readable stdout

### Requirement: 日志库引入必须通过依赖与输出通道审计

The first implementation of invocation logging MAY use an internal JSONL writer. Introducing a logging framework such as `tracing` or `tracing-subscriber` MUST be preceded by an implementation audit that records dependency impact, initialization behavior, output sink isolation, feature selection, and test coverage for stdout purity.

#### Scenario: 首期使用内部 writer

- **WHEN** the implementation uses a repository-owned JSONL writer
- **THEN** the writer owns event serialization, append behavior, bounded diagnostics, and failure downgrade behavior
- **THEN** no global logging subscriber is required for the metadata-only invocation record

#### Scenario: 后续引入 tracing

- **WHEN** a later implementation introduces `tracing` or another logging framework
- **THEN** the change records why the framework is needed beyond the internal writer
- **THEN** validation proves that document stdout and adapter protocol stdout remain uncontaminated
