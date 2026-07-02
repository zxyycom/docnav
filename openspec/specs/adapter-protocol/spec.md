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
Current document operations MUST dispatch to linked adapter operation handlers through the in-process adapter contract. Core and the navigation layer MUST prepare typed operation input from CLI/config/protocol arguments before dispatch, including document path resolution, pagination, output mode, ref/query fields and declared native options.

Adapter handlers MUST NOT read CLI argv、stdin、stdout、stderr、process cwd or process exit code to obtain operation input. The adapter contract MAY return success payloads or structured adapter diagnostics; final protocol/readable projection and process exit code remain owned by core/output surfaces.

#### Scenario: Core prepares request before linked dispatch
- **WHEN** `docnav outline docs/guide.md --limit 120` selects the linked Markdown adapter
- **THEN** core resolves the document path to an absolute path before calling the navigation layer
- **THEN** standard parameter resolution prepares typed operation input with `limit: 120`
- **THEN** the linked Markdown handler receives the prepared input without reading process cwd or CLI argv

#### Scenario: Adapter error is structured
- **WHEN** a linked adapter cannot satisfy an operation
- **THEN** it returns a structured diagnostic or adapter error to the caller boundary
- **THEN** it does not expose an adapter exit-code API
- **THEN** core/output maps the diagnostic to protocol/readable output and the final process exit code

### Requirement: Manifest/probe metadata 不提供 implementation source
Manifest、probe result 和 equivalent descriptor metadata MUST restrict field ownership to adapter identity, supported formats, extensions, content types, capabilities and observable metadata. They MUST NOT provide runtime implementation sources, command paths, external executables, protocol version ranges or default/native option values.

#### Scenario: 读取 manifest metadata
- **WHEN** adapter metadata is rendered as manifest-shaped output
- **THEN** fields express adapter identity, supported formats, extensions, content types and capabilities
- **THEN** metadata does not contain command path, executable path, protocol range or `recommended_parameters`

#### Scenario: 旧 implementation 字段被拒绝
- **WHEN** manifest-like metadata contains `protocol.min`, `protocol.max`, command path or `recommended_parameters`
- **THEN** current validation rejects that metadata as a current contract artifact
- **THEN** the default core-linked document operation path does not use it as an implementation source

### Requirement: Native options 保持 adapter-owned registry source
Protocol request argument types MUST keep optional `options` as an opaque adapter-owned object. Core standard parameter resolution MAY accept and merge native option values only when a source-level static native option registry declares the corresponding public source. Registry entries MUST preserve owner、namespace、key and type variant metadata, and the same option key MAY have multiple owner or type variants. Protocol schema MUST NOT derive `options` from manifest metadata or examples.

#### Scenario: Declared native option is handed to adapter
- **WHEN** the source-level static registry contains Markdown `options.max_heading_level`
- **AND** CLI/config/protocol arguments provide `options.max_heading_level: 2`
- **THEN** standard parameter resolution merges source values and preserves registry owner/namespace/type variant metadata
- **THEN** the Markdown operation handler receives the final option value as part of prepared operation input

#### Scenario: Adapter validates unsupported or invalid option
- **WHEN** request arguments contain an adapter-owned option value
- **AND** adapter selection succeeds
- **THEN** the selected adapter validates whether the option is supported for that adapter and operation
- **THEN** type mismatch or range invalid returns adapter-owned structured diagnostic

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
Docnav protocol validation MUST use current protocol、manifest、probe and readable schema plus semantic checks to determine whether a surface artifact conforms to the current contract. `protocol_version`、`manifest_version` and `probe_version` MUST remain fixed schema identification fields, but MUST NOT participate in adapter routing, installation, update or external invoke version negotiation.

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

### Requirement: Legacy invoke error code is compatibility-only
`ADAPTER_INVOKE_FAILED` MAY remain in protocol schema and examples only as a legacy/deprecated compatibility projection for historical external invoke surfaces. Current core-linked adapter operations MUST NOT use an external adapter executable, stdout/stderr response parsing, or adapter process exit code as the default implementation source.

#### Scenario: Current linked operation fails
- **WHEN** a linked adapter operation fails
- **THEN** the adapter layer returns structured diagnostic facts
- **THEN** core/output chooses the protocol/readable error projection and process exit code
- **THEN** `ADAPTER_INVOKE_FAILED` is not required to model the implementation mechanism

#### Scenario: Historical projection remains parseable
- **WHEN** a legacy compatibility artifact contains `ADAPTER_INVOKE_FAILED`
- **THEN** schema MAY allow legacy external invoke details such as `exit_code` or `stderr`
- **THEN** docs identify those details as historical compatibility, not the current recommended path
