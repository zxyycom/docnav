本 spec delta 定义 Docnav runtime invocation logging 的新增要求：可通过 CLI/config 显式启用、默认 metadata-only，并以 SHA-256 content hash 和可选 content capture directory 控制正文日志体积。

## ADDED Requirements

### Requirement: 调用日志必须显式启用并使用独立 sink

Docnav runtime invocation logging MUST be inactive unless an explicit CLI option, configuration field, or equivalent owner-documented CLI/config surface enables it. When inactive, invocation logging MUST NOT add stdout/stderr output, protocol fields, readable payload fields, linked adapter handler payload, or log file side effects. When active, invocation logging MUST write events only to an explicitly resolved log sink/path that is separate from document output.

#### Scenario: 未启用时没有可观察输出变化

- **WHEN** a caller runs a document operation without enabling invocation logging
- **THEN** stdout, stderr, exit code, `RequestEnvelope`, `ProtocolResponse`, readable payloads, and linked adapter handler payloads remain the same as the equivalent run without this feature
- **THEN** no invocation log event is created as a side effect of the document operation

#### Scenario: 启用时只写入配置 sink

- **WHEN** invocation logging is enabled with an owner-documented log sink
- **THEN** invocation log events are written only to that resolved sink
- **THEN** the event sink is not document output stdout and is not injected into protocol, readable, manifest, probe, or adapter handler payloads

### Requirement: 调用日志使用结构化 JSONL 事件

Docnav runtime invocation logging MUST write structured JSON Lines / NDJSON events when invocation logging is enabled. Each event MUST include a schema version, timestamp, event name, request id when available, operation, selected adapter id when available, success/failure status, duration, and bounded size/status metadata sufficient to correlate a core CLI document operation with the linked adapter dispatch outcome. Event shape MUST be covered by JSON Schema validation material.

#### Scenario: 成功调用写入可关联事件

- **WHEN** invocation logging is enabled and `docnav` completes an adapter-backed document operation successfully
- **THEN** the log contains at least one JSONL event with `schema_version`, timestamp, event name, `request_id`, operation, adapter id, success status, duration, output/error status metadata, and bounded response size metadata
- **THEN** the event can be parsed independently from other log lines without requiring the full log file to be valid JSON

#### Scenario: 失败调用写入失败边界

- **WHEN** invocation logging is enabled and adapter selection, navigation request construction, linked adapter handler dispatch, operation result validation, output projection, or operation execution fails
- **THEN** the log records the available request id or fallback correlation id
- **THEN** the log records the failure layer, stable error code when available, duration, and bounded diagnostic summary

### Requirement: 默认调用日志只记录元数据

When invocation logging is enabled, invocation records MUST default to metadata-only records. Metadata-only records MUST NOT include document body content, full protocol request/response payloads, full structured diagnostic/debug output, full environment variables, secrets, full document paths without an owner-documented display policy, or unbounded raw user input.

#### Scenario: Read 响应不默认写入正文

- **WHEN** invocation logging is enabled in metadata-only mode for a `read` operation
- **THEN** the log records request/response correlation and bounded size/status metadata
- **THEN** the log does not record the full `content` field from the read result

#### Scenario: Find 查询和 ref 使用安全摘要

- **WHEN** invocation logging is enabled in metadata-only mode for `find` or `read`
- **THEN** query and ref fields are either omitted, represented by presence/length metadata, or represented by an explicitly bounded and documented summary
- **THEN** the log does not treat adapter-owned ref grammar or document text as a stable cross-layer identity

### Requirement: 文档内容使用 hash 引用和可选 content capture directory

Invocation logs MUST NOT inline full document content in the primary invocation log. When document content needs to be represented in an operation result event, the primary event MUST use a compact content reference containing `hash_algorithm: "sha256"`, a lowercase 64-character hexadecimal SHA-256 `content_hash`, content type when available, size metadata, and optional bounded summary metadata. The SHA-256 input MUST be the same bytes written to the content capture file when capture is enabled; when capture is disabled, the hash MUST be computed from the same content bytes that would be captured without line-ending, encoding, or whitespace normalization. A separate content capture directory MAY store content files only when enabled through a separate owner-documented CLI/config option and root path. When content capture is enabled, the primary invocation log MUST record content capture events containing capture time, hash metadata, size metadata, and the content file `relative_path`. Content capture event shape MUST be covered by invocation log JSON Schema validation.

#### Scenario: 主调用日志只记录正文 hash

- **WHEN** invocation logging records a `read` result or other document content metadata
- **THEN** the primary invocation log records `hash_algorithm: "sha256"`, a lowercase 64-character hexadecimal SHA-256 `content_hash`, content type when available, and bounded size metadata
- **THEN** the primary invocation log does not inline the full document content

#### Scenario: content capture 单独开启

- **WHEN** invocation logging is enabled but content capture is not enabled
- **THEN** no content file is written as a side effect of the document operation
- **WHEN** content capture is separately enabled with an owner-documented root path
- **THEN** content files are written only under that resolved content capture directory
- **THEN** each content file path uses an owner-documented relative path with a date directory and hash filename formatted as `<YYYY-MM-DD>/sha256-<content_hash>.content`
- **THEN** the primary invocation log records a `content_captured` event with `captured_at`, hash metadata, size metadata, and `relative_path`
- **THEN** the content file is not written to document stdout, protocol output, readable output, or inline primary invocation log fields

#### Scenario: content capture event 只记录相对路径

- **WHEN** content capture writes a content file
- **THEN** the invocation log event records the content file location with `relative_path`
- **THEN** the event does not introduce additional content file location fields

### Requirement: 调用日志不得污染 stdout 或协议输出

Invocation logging MUST be isolated from document output stdout and linked adapter handler payloads. Logging MUST NOT add fields to `RequestEnvelope`, `ProtocolResponse`, readable output payloads, manifest output, or probe output.

#### Scenario: protocol-json stdout 保持纯净

- **WHEN** invocation logging is enabled and a caller runs a document operation with `--output protocol-json`
- **THEN** stdout contains only the protocol-shaped response for that operation
- **THEN** log events are written only to the configured log sink

#### Scenario: readable-json stdout 保持单一 JSON 值

- **WHEN** invocation logging is enabled and a caller runs a document operation with `--output readable-json`
- **THEN** stdout contains only the readable-json value for that operation
- **THEN** log events are written only to the configured log sink

#### Scenario: linked adapter dispatch 保持 payload 边界

- **WHEN** core CLI dispatches a linked adapter handler
- **THEN** the handler still returns only structured result or diagnostic payloads to the caller boundary
- **THEN** runtime invocation logs are written only to the configured log sink and are not injected into handler input, handler output, or document stdout

### Requirement: 日志写入失败不改变文档操作语义

Invocation logging failures MUST NOT change the success or failure semantics of the document operation being logged. The original CLI exit behavior, output mode, and stable error mapping MUST remain determined by the document operation and adapter result, not by the logging subsystem.

#### Scenario: 日志文件不可写时主调用继续

- **WHEN** invocation logging is enabled but the configured log file cannot be created or appended
- **THEN** a document operation that would otherwise succeed still succeeds
- **THEN** stdout remains valid for the selected output mode
- **THEN** any logging failure diagnostic is bounded and emitted only through a channel that does not corrupt machine-readable stdout

### Requirement: 外部日志框架引入必须通过依赖与输出通道审计

Invocation logging SHOULD use an internal JSONL writer unless a separate implementation audit proves that an external logging framework is needed. Introducing an external logging framework MUST be preceded by an implementation audit that records dependency impact, initialization behavior, output sink isolation, feature selection, and test coverage for stdout purity.

#### Scenario: 使用内部 writer

- **WHEN** the implementation uses a repository-owned JSONL writer
- **THEN** the writer owns event serialization, append behavior, bounded diagnostics, and failure downgrade behavior
- **THEN** no global logging subscriber is required for the metadata-only invocation record

#### Scenario: 引入外部日志框架

- **WHEN** an implementation introduces an external logging framework
- **THEN** the change records why the framework is needed beyond the internal writer
- **THEN** validation proves that document stdout and linked adapter handler payloads remain uncontaminated

### Requirement: 调用日志必须有 schema 和示例验证材料

Invocation logging MUST add JSON Schema and example validation materials for primary invocation log event variants. If content capture is implemented, invocation log schema and examples MUST cover content capture events. Schemas MUST remain validation materials and MUST NOT take ownership of CLI/config semantics, protocol envelope semantics, adapter behavior, content file bytes, or stdout/stderr placement.

#### Scenario: 主调用日志 event 通过 schema 验证

- **WHEN** an implementation emits a primary invocation log JSONL event
- **THEN** the event matches the documented invocation log JSON Schema
- **THEN** examples include at least one successful metadata-only event and one bounded failure event

#### Scenario: content capture event 通过 schema 验证

- **WHEN** content capture is implemented and emits a content capture event
- **THEN** the event matches the documented invocation log JSON Schema
- **THEN** examples prove that operation result events refer to content through hash metadata rather than inline full content
- **THEN** examples prove that content capture events locate captured content through `relative_path`
