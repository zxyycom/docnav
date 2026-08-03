本 delta spec 定义 `replace-probe-traversal-with-inferred-routing` 对 `invocation-logging` 尚未应用的 Target：删除已废止 probe payload/output 的正向日志隔离契约，同时保持日志 sink、stdout、protocol 与 linked adapter payload 的既有隔离边界；它不表示 Current 主规范或实现已经迁移。

## MODIFIED Requirements

### Requirement: 调用日志必须显式启用并使用独立 sink

Docnav runtime invocation logging MUST be inactive unless an explicit CLI option, configuration field, or equivalent owner-documented CLI/config surface enables it. When inactive, invocation logging MUST NOT add stdout/stderr output, protocol fields, readable payload fields, linked adapter handler payload, or log file side effects. When active, invocation logging MUST write events only to an explicitly resolved log sink/path that is separate from document output.

#### Scenario: 未启用时没有可观察输出变化

- **WHEN** a caller runs a document operation without enabling invocation logging
- **THEN** stdout, stderr, exit code, `RequestEnvelope`, `ProtocolResponse`, readable payloads, and linked adapter handler payloads remain the same as the equivalent run without this feature
- **THEN** no invocation log event is created as a side effect of the document operation

#### Scenario: 启用时只写入配置 sink

- **WHEN** invocation logging is enabled with an owner-documented log sink
- **THEN** invocation log events are written only to that resolved sink
- **THEN** the event sink is not document output stdout and is not injected into protocol, readable, manifest, or adapter handler payloads
- **THEN** no removed probe payload is retained as a logging destination

### Requirement: 调用日志不得污染 stdout 或协议输出

Invocation logging MUST be isolated from document output stdout and linked adapter handler payloads. Logging MUST NOT add fields to `RequestEnvelope`, `ProtocolResponse`, readable output payloads, or manifest output. The removed probe output surface MUST NOT be reconstructed as a logging record or destination.

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
- **THEN** logging emits no removed probe result or record
