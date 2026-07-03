# adapter-protocol Specification

## Purpose
定义 Docnav v0 原始协议共享类型、schema/example 校验材料、linked adapter operation handler 边界，以及 adapter descriptor/probe/manifest 元数据的所有权。该规范不定义 adapter direct CLI、外部 `invoke` 进程或 runtime adapter SDK 作为当前默认执行路径。
## Requirements
### Requirement: 共享协议类型完整覆盖 v0 原始协议
`docnav-protocol` MUST 定义 v0 request envelope、response envelope、operation、operation arguments、operation result、page、protocol error、manifest 和 probe 的共享类型，并 MUST 不包含格式专属解析字段。

#### Scenario: 构造 outline 成功响应
- **WHEN** 调用方使用共享协议类型构造 `outline` 成功响应
- **THEN** 响应包含 `protocol_version`、`request_id`、`operation: "outline"`、`ok: true` 和 outline result
- **THEN** result 只包含扁平 entries 和 page

#### Scenario: 拒绝格式专属字段进入共享协议
- **WHEN** 实现者需要表达 Markdown heading path
- **THEN** 该信息只能存在于 adapter 生成的 `ref` 或 `display`
- **THEN** `docnav-protocol` 不新增 Markdown 专属 result 字段

### Requirement: operation 必须绑定成功 result 类型
Protocol response schema 和共享校验 MUST 使用响应 `operation` 绑定成功 result 类型，且成功响应 operation MUST 与请求 operation 一致。

#### Scenario: read 响应绑定 ReadResult
- **WHEN** 请求 operation 为 `read`
- **THEN** 成功响应 operation 为 `read`
- **THEN** result 必须符合 ReadResult

### Requirement: Linked adapter handler 接收已准备的 operation input
Docnav core 和 navigation layer MUST 在 dispatch linked adapter handler 前完成 public input boundary 处理。Core MUST classify commands and pass config source descriptors/paths; `docnav-navigation` MUST load raw config sources and construct typed operation input from raw command, protocol request arguments, project/user config and built-in defaults, preserving declared adapter-owned native option source metadata. Linked adapter handlers MUST NOT read CLI argv、stdin、stdout、stderr、process cwd or process exit code to obtain operation input.

Invalid public input MUST fail before linked adapter business execution when it belongs to core CLI parsing、protocol envelope/request shape、config source loading、navigation input resolution mapping or operation applicability。Declared adapter-owned native options MAY be handed to the selected adapter through source-level static native option registry metadata；unsupported option、type mismatch or range invalid MUST be reported by selected adapter typed-field validation before format business handling continues.

#### Scenario: core CLI unknown argv 被拒绝在 adapter dispatch 前
- **WHEN** caller executes `docnav outline docs/guide.md --unknown --output readable-json`
- **THEN** core CLI returns an input diagnostic
- **THEN** navigation does not dispatch the linked adapter handler
- **THEN** failure output projects one primary `DiagnosticRecord`

#### Scenario: protocol request shape failure 停在 protocol owner
- **WHEN** a protocol request JSON value contains unknown envelope fields、missing required fields or malformed request shape
- **THEN** protocol input validation rejects the request at the protocol boundary
- **THEN** navigation input resolution does not receive the invalid envelope
- **THEN** failure output uses the protocol failure projection for the primary `DiagnosticRecord`

#### Scenario: known operation arguments 进入 navigation input resolution
- **WHEN** a protocol request envelope is valid but operation arguments contain wrong type、unmapped arguments or invalid values
- **THEN** navigation input resolution and typed-field processing produce validation diagnostics
- **THEN** linked adapter business handling does not execute
- **THEN** the owning surface projects the diagnostics as a failed document request

#### Scenario: declared native option handoff 保留 owner metadata
- **WHEN** CLI、config or protocol arguments provide `options.max_heading_level: 2`
- **AND** the source-level static native option registry declares the Markdown option source
- **THEN** navigation input resolution preserves source kind、owner、namespace、key and type variant metadata
- **THEN** the linked Markdown handler receives the merged native option value in prepared operation input

#### Scenario: selected adapter typed-field native option validation 返回结构化诊断
- **WHEN** adapter selection succeeds and prepared input contains an unsupported option、type mismatch or range invalid value for the selected adapter
- **THEN** selected adapter typed-field validation returns a structured diagnostic before handler execution
- **THEN** core/output projects that diagnostic through the selected raw or readable failure surface

### Requirement: Manifest/probe metadata 不提供 implementation source
Manifest、probe result 和 equivalent descriptor metadata MUST restrict field ownership to adapter identity, supported formats, extensions, content types and observable metadata. They MUST NOT provide runtime implementation sources, command paths, external executables, protocol version ranges, document operation sets or default/native option values.

#### Scenario: 读取 manifest metadata
- **WHEN** adapter metadata is rendered as manifest-shaped output
- **THEN** fields express adapter identity, supported formats, extensions and content types
- **THEN** metadata does not contain command path, executable path, protocol range or `recommended_parameters`

#### Scenario: 旧 implementation 字段被拒绝
- **WHEN** manifest-like metadata contains `protocol.min`, `protocol.max`, command path or `recommended_parameters`
- **THEN** current validation rejects that metadata as a current contract artifact
- **THEN** the default core-linked document operation path does not use it as an implementation source

### Requirement: Native options 保持 adapter-owned registry source
Protocol request argument types MUST keep optional `options` as an opaque adapter-owned object. Navigation input resolution MAY accept and merge native option values only when the selected adapter typed-field declarations and source-level static native option registry declare the corresponding public source. Registry entries MUST preserve owner、namespace、key and type variant metadata, and the same option key MAY have multiple owner or type variants. Protocol schema MUST NOT derive `options` from manifest metadata or examples.

#### Scenario: Declared native option is handed to adapter
- **WHEN** the source-level static registry contains Markdown `options.max_heading_level`
- **AND** CLI/config/protocol arguments provide `options.max_heading_level: 2`
- **THEN** navigation input resolution merges source values and preserves registry owner/namespace/type variant metadata
- **THEN** the Markdown operation handler receives the final option value as part of prepared operation input

#### Scenario: Navigation validates unsupported or invalid selected option
- **WHEN** request arguments contain an adapter-owned option value
- **AND** adapter selection succeeds
- **THEN** selected adapter typed-field declarations validate whether the option is supported for that adapter and operation
- **THEN** type mismatch or range invalid returns a navigation input resolution diagnostic with selected adapter/source metadata

#### Scenario: Manifest 不提供 options 来源
- **WHEN** adapter metadata passes current schema validation
- **THEN** metadata does not contain `recommended_parameters`
- **THEN** core does not synthesize `arguments.options` from manifest metadata

### Requirement: 自动化验证必须覆盖 schema 与示例
Docnav protocol materials MUST provide automated validation for protocol request/response, manifest, probe, readable schema, and key JSON examples. Validation MUST prove schema shape and documented semantic bindings, including operation/result matching.

#### Scenario: 校验协议响应 fixture
- **WHEN** 验证脚本读取 protocol response 示例
- **THEN** 示例通过 protocol response schema
- **THEN** 响应 operation 与 result 类型匹配

### Requirement: 协议边界必须按当前契约硬校验
Docnav protocol validation MUST use current protocol、manifest、probe and readable schema plus semantic checks to determine whether a surface artifact conforms to the current contract. `protocol_version`、`manifest_version` and `probe_version` MUST remain fixed schema identification fields, but MUST NOT participate in adapter selection, installation, update or external invoke version negotiation.

#### Scenario: 当前契约校验通过
- **WHEN** protocol response, manifest-shaped metadata or probe-shaped metadata conforms to current schema
- **AND** required fields, field types, operation/result shape and semantic checks all pass
- **THEN** the protocol layer treats the artifact as current-contract valid

#### Scenario: 当前契约校验失败
- **WHEN** output misses a current required field or has a wrong field type
- **THEN** validation failure includes field or schema path evidence
- **THEN** the owning boundary returns the stable protocol/readable/diagnostic projection for that failure

#### Scenario: 请求版本字段不匹配当前 schema
- **WHEN** a protocol request contains a `protocol_version` other than the current fixed schema value
- **THEN** request schema or typed-field validation fails
- **THEN** the failure uses `INVALID_REQUEST` or the diagnostics-owned current equivalent
- **THEN** it does not use version-range negotiation

### Requirement: Text cost calculator outputs protocol-compatible measurements
Shared text cost calculator helpers MUST return cost measurements that can be represented through the current `cost.measurements[]` protocol shape without adding format-specific protocol fields or readable-only fields. For each helper function call, the returned measurement MUST include the function-defined `unit` and a helper-computed non-negative integer `value`.

#### Scenario: Plain text cost maps to protocol cost
- **WHEN** a Docnav component already has selected plain text and calls a shared text cost helper function
- **THEN** the helper returns a measurement with protocol-compatible `unit` and non-negative integer `value`
- **THEN** callers can embed one or more such measurements in `cost.measurements[]` without changing response envelope or operation result shape
- **THEN** readable cost summaries remain derived by the output layer from protocol measurements

#### Scenario: Scope remains caller-owned protocol context
- **WHEN** a shared text cost helper function returns a measurement
- **THEN** the helper result has no helper-selected scope
- **WHEN** a caller embeds that measurement in a protocol result that has scoped cost semantics
- **THEN** the caller attaches the operation-appropriate scope without changing helper input or helper function selection

