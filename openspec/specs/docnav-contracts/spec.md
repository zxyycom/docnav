# docnav-contracts Specification

## Purpose
定义 Docnav v0 长期文档契约，约束原始协议、阅读输出、core CLI 职责、navigation input resolution、linked adapter 边界、诊断投影、示例材料和自动化验证映射。默认文档操作路径是 core release 内的静态注册 linked adapter library，不是 runtime dynamic registration、external executable 或 adapter `invoke` 进程。

## Requirements
### Requirement: 原始协议与阅读输出分层
项目 MUST 将 `docnav --output protocol-json` 原始协议与 CLI 阅读输出定义为两个独立语义层。原始协议层 MUST 作为机器稳定接口；阅读输出层 MUST 优先服务 AI 和人类阅读，不作为长期机器解析接口。两层 MUST 复用业务语义，并以各自 schema 校验字段形状。

#### Scenario: 原始协议响应自描述 operation
- **WHEN** core renders a successful `protocol-json` response
- **THEN** response envelope contains the executed `operation`
- **THEN** protocol response schema uses `operation` to validate the corresponding result type

#### Scenario: Read 保留内容类型
- **WHEN** adapter read returns content
- **THEN** protocol result and readable JSON both include `content_type`
- **THEN** `content_type` does not participate in content budget truncation

### Requirement: `docnav` 是 core CLI router/manager
`docnav` MUST own command classification, project root discovery, cwd handling, config source descriptor/path discovery for handoff, document path resolution, output mode dispatch and stable error mapping. For navigation commands, core MUST pass the raw command, config source descriptors/paths and current static registry to `docnav-navigation`; `docnav-navigation` MUST load raw project/user config sources before source resolution. Core MUST resolve the project/cwd/document path context before navigation input resolution.

Core MUST NOT treat project/user historical adapter records, installed packages, external executables, command paths or manifest metadata as default document operation implementations.

#### Scenario: 读取 Markdown outline
- **WHEN** caller executes `docnav outline docs/guide.md`
- **THEN** `docnav` resolves `docs/guide.md` against the effective cwd/project context into an absolute document path
- **THEN** `docnav` passes raw command, config source descriptors/paths and static registry to `docnav-navigation`
- **THEN** navigation input resolution selects the adapter and prepares typed operation input before dispatch
- **THEN** adapter-generated refs and display facts are preserved into the selected output surface

#### Scenario: Missing adapter implementation
- **WHEN** caller explicitly requests an adapter id absent from the current static registry
- **THEN** `docnav` returns a stable adapter selection diagnostic
- **THEN** `docnav` does not search installed packages, command paths, external executables or historical adapter artifacts as fallback implementation sources

### Requirement: Static registry 是 adapter implementation source
The current core release static adapter registry MUST be the source of default document operation implementations. A registry entry MUST resolve to source-linked adapter code and a static descriptor containing adapter id, supported formats, content types, native option registry entries, and operation handler bindings. Descriptor metadata is inspection metadata and release invariant material. Runtime candidate order is the registry order, and format support is decided by adapter probe results.

#### Scenario: Adapter list inspects linked metadata
- **WHEN** caller executes `docnav adapter list`
- **THEN** output is derived from the current static registry and linked adapter descriptors
- **THEN** output does not imply runtime installation, dynamic registration or external executable discovery

#### Scenario: Descriptor declares operation handlers
- **WHEN** a linked adapter participates in document operations
- **THEN** it declares and provides `outline`, `read`, `find` and `info` handlers
- **THEN** missing handlers are treated as adapter layer invalid or a release/doctor check failure, not as a recoverable per-request candidate branch

### Requirement: Navigation layer resolves inputs, prepares requests and dispatches operations
The navigation layer MUST act as the in-process navigation input resolution, request preparation and operation dispatch boundary. It MUST parse routing-required input, select the adapter from the provided registry, read selected adapter typed-field parameter declarations, resolve sources, validate/extract typed operation arguments, construct request envelopes and call operation handlers. It MUST NOT be described or implemented as an adapter loader, executable launcher or runtime registry manager.

#### Scenario: Dispatch linked operation
- **WHEN** core passes a raw Markdown outline command, config sources and registry to `docnav-navigation`
- **THEN** the navigation layer selects the linked Markdown adapter and prepares typed outline input
- **THEN** the navigation layer dispatches to the descriptor-bound outline handler
- **THEN** the handler receives prepared values rather than CLI argv, process cwd, stdin or stdout handles

### Requirement: Adapter contract returns structured results or diagnostics
Adapter handlers MUST return structured operation results or structured diagnostics to the caller boundary. Adapter contracts MUST NOT expose process exit code as an adapter API. CLI process exit code, stdout/stderr placement, readable output and protocol error envelope are owned by core/output surfaces.

#### Scenario: Adapter reports a parse failure
- **WHEN** a linked adapter cannot parse or navigate a document
- **THEN** it returns structured diagnostic facts with adapter-owned context
- **THEN** core/output maps those facts to protocol/readable output and final process exit code

### Requirement: Adapter 选择按 static registry 和 probe 校验
`docnav-navigation` MUST choose adapter implementations only from the current static registry provided by core. Declared adapter ids come only from direct input or `defaults.adapter`; when present, `docnav-navigation` MUST look up only that registry entry, execute its probe and MUST NOT fallback to automatic discovery. Without a declared adapter id, `docnav-navigation` MUST traverse the static registry in release order and select the first adapter whose probe returns `supported: true`. Descriptor, manifest, path, extension and content type facts remain inspection or adapter-owned recognition inputs; runtime selection uses only declared adapter lookup or registry-order probe.

#### Scenario: 声明式 adapter 不 fallback
- **WHEN** caller provides `--adapter docnav-markdown`
- **THEN** `docnav-navigation` looks up only `docnav-markdown` in the current static registry
- **THEN** selection succeeds only if that linked adapter's probe accepts the document
- **THEN** unsupported, invalid or missing declared adapter failure returns an adapter selection diagnostic without trying later registry entries

#### Scenario: 未声明 adapter 只按 registry 顺序 probe
- **WHEN** no declared adapter id exists
- **THEN** `docnav` traverses static registry entries in release order
- **THEN** descriptor extension/content-type/format facts remain inspection metadata
- **THEN** registry order and probe results determine the selected adapter
- **THEN** the first adapter probe returning `supported: true` selects the adapter

#### Scenario: missing operation handler is release invariant failure
- **WHEN** a default adapter layer entry lacks `outline`, `read`, `find` or `info`
- **THEN** that missing handler is an adapter layer invariant, release validation or `doctor` failure
- **THEN** it is not modeled as a normal runtime candidate branch

### Requirement: 接入方式共享 `docnav` 契约
Core CLI, tools, skills and AGENTS.md/system prompts MUST share `docnav` path, ref, page, limit, output mode and error contract. Adapter refs remain opaque strings generated and interpreted by the selected adapter.

#### Scenario: Agent 通过项目规则读取文档
- **WHEN** project rules instruct an agent to read a large document
- **THEN** the agent uses `docnav outline/read/find/info`
- **THEN** the agent passes outline/find refs back to read unchanged

### Requirement: Markdown v0 通过 linked adapter 提供首期能力
Markdown v0 adapter MUST implement `outline`, `read`, `find` and `info` through linked adapter handlers. `outline -> ref -> read` MUST remain the primary vertical reading flow.

#### Scenario: Markdown operation handlers
- **WHEN** caller inspects linked Markdown adapter metadata
- **THEN** the linked adapter exposes `outline`, `read`, `find` and `info` handlers
- **THEN** each operation has core CLI behavior and linked handler verification

### Requirement: Outline 只使用扁平条目
Shared protocol MUST define outline as flat entries. Each entry MUST contain `ref` and `display`; hierarchy MAY be represented by adapter-generated refs, labels or display text without adding Markdown-specific protocol fields.

#### Scenario: 嵌套 Markdown heading
- **WHEN** Markdown document contains nested headings
- **THEN** outline returns flat entries in document order
- **THEN** hierarchy is expressed by adapter-owned ref/display facts

### Requirement: 默认结果有限且可继续
Each navigation operation MUST have a clear budget default. Paginated operations MUST use positive integer `page` and canonical `limit`, and MUST return the next page number or null. The budget unit interpretation remains owned by the adapter, while protocol shape, field name and continuation semantics remain shared.

Page MUST be call-position state rather than configuration default. When an entrypoint omits page, it MUST start from `1`.

#### Scenario: Markdown outline reaches default budget
- **WHEN** outline exceeds the effective budget
- **THEN** adapter returns only entries for the current page
- **THEN** result includes the next page number

#### Scenario: Ref remains完整
- **WHEN** a single outline or find record's full ref is larger than the effective entry budget
- **THEN** ref remains complete
- **THEN** the single record may exceed budget but pagination still progresses

### Requirement: Navigation input resolution uses source-level native option registry
Navigation input resolution MUST prepare typed operation arguments from explicit input, project/user configuration and built-in defaults according to the owning docs. Adapter-owned native options MAY participate through the source-level static native option registry and selected adapter typed-field declarations. Registry entries MUST preserve owner, namespace, key and type variant metadata; the same option key MAY have multiple variants without being collapsed into one core type.

#### Scenario: Markdown max heading level reaches adapter
- **WHEN** `<project-root>/.docnav/docnav.json`、core user config or request arguments provide `options.max_heading_level`
- **AND** the source-level static registry contains the Markdown option entry
- **THEN** navigation input resolution merges the value with source info and registry metadata
- **THEN** the Markdown handler receives the final max-heading-level option value

#### Scenario: Selected adapter typed-field declarations own option validation
- **WHEN** request/config input contains a native option value
- **AND** adapter selection succeeds
- **THEN** navigation input resolution validates selected adapter support, type mismatch and range invalid before handler dispatch
- **THEN** unsupported option diagnostics preserve selected adapter/source metadata

### Requirement: Ref 文档必须描述 adapter 拥有的 ref 边界
Ref docs and examples MUST describe ref as an adapter-generated and adapter-interpreted non-empty opaque string. Shared protocol, `docnav` and access layers MUST validate only shared field shape and pass ref unchanged to the selected adapter.

Adapter-specific grammar, lookup semantics, operation applicability, read granularity, uniqueness, stability, disambiguation and invalid-ref behavior MUST be defined by the corresponding adapter's main specification.

#### Scenario: 共享层原样传递 ref
- **WHEN** caller submits a non-empty ref to read
- **THEN** `docnav` selects an adapter by path/context and passes ref unchanged
- **THEN** shared protocol and schema do not parse or infer internal ref structure

#### Scenario: 查找 adapter 私有 ref 语义
- **WHEN** reader needs Markdown ref behavior
- **THEN** shared docs point to `docs/adapters/markdown.md`
- **THEN** Markdown docs define grammar, supported operations, guarantees and error boundary

### Requirement: 稳定契约可直接校验
Project MUST provide independent JSON Schema and complete parseable examples for raw protocol, manifest/probe-shaped metadata and readable outputs. Protocol schema MUST serve the machine-stable surface; readable schema MUST serve examples and implementation self-tests.

#### Scenario: 校验同一业务结果
- **WHEN** auditor validates protocol and readable examples
- **THEN** protocol JSON passes protocol response schema and operation/result binding checks
- **THEN** readable JSON passes readable schema
- **THEN** both surfaces preserve the same business facts with separate wrappers and compatibility goals

### Requirement: 文档以中文为主要审计语言
Audience-facing explanations, design reasons and requirements MUST primarily use Chinese, while commands, fields, enum values and error codes keep English machine identifiers.

#### Scenario: 审计协议
- **WHEN** auditor reads protocol docs
- **THEN** rule descriptions use Chinese and machine identifiers remain English

### Requirement: 文档契约可映射到自动化验证
Testing strategy MUST map protocol stability, readable output density, navigation input resolution ownership, adapter descriptor/native option registry and selected adapter typed-field validation, ref/page/limit behavior and core CLI end-to-end linked adapter behavior to automated tests or documented smoke checks.

#### Scenario: 提出实现变更
- **WHEN** implementer plans a behavior change
- **THEN** docs identify protocol, readable, navigation input resolution, adapter-boundary and `docnav` route tests
- **THEN** package smoke proves linked adapter behavior through the packaged `docnav` binary

### Requirement: 文档阅读路径清晰
README MUST remain a traditional project README and MUST NOT duplicate full specifications. `docs/navigation.md` MUST be the daily documentation navigation entry and MUST contain role-based reading paths, document layers and rule-owner index. Schema and examples MUST remain validation materials; OpenSpec changes MUST remain change proposals, acceptance records and audit history rather than the daily implementation entry.

#### Scenario: 新实现者选择阅读路径
- **WHEN** a new implementer opens README
- **THEN** they can find `docs/navigation.md`
- **THEN** `docs/navigation.md` directs them to the relevant owner docs and validation materials

### Requirement: 共享 Rust crate 所有权必须保持 Docnav 契约分层
Docnav shared Rust crates MUST keep raw protocol, document output orchestration, JSON IO, readable renderer, diagnostics, CLI argv classification, navigation dispatch, adapter contract definitions and format adapter semantics separated by owner. Shared crates MUST lift only stable contracts and mechanical flow; routing, ref interpretation, format parsing and user-visible surface policy remain with their documented owners unless a later spec changes that boundary.

#### Scenario: 共享 crate 不接管 adapter-owned 语义
- **WHEN** adapter generates, parses or rejects a ref
- **THEN** shared protocol, output, diagnostics and CLI argv crates treat that ref as opaque
- **THEN** shared crates do not infer heading structure, uniqueness, region boundary or format-specific navigation behavior

#### Scenario: Navigation and contracts split remain internal support
- **WHEN** implementation uses `docnav-navigation` and `docnav-adapter-contracts`
- **THEN** `docnav-navigation` prepares requests and dispatches operations
- **THEN** `docnav-adapter-contracts` defines in-process adapter descriptors, handler traits and shared adapter-layer types
- **THEN** neither crate creates an external runtime adapter SDK contract by itself

### Requirement: Runtime problems flow through a request-local diagnostic stack
Docnav runtime and public surface code MUST record runtime problems in a request-local diagnostic stack before the owning boundary decides whether to continue, fail, exit or write surface-specific output.

#### Scenario: Fatal problem records context before failure surface
- **WHEN** an operation encounters a fatal request, document, adapter boundary or internal failure
- **THEN** the diagnostic stack records the fatal context before the fatal outcome is returned or propagated
- **THEN** the record carries a diagnostic code that can be projected to surface error code, message, details, guidance and exit-code category

#### Scenario: Diagnostic stack stores facts only
- **WHEN** a diagnostic record is pushed
- **THEN** the stack stores the record without deciding whether the operation succeeds or fails
- **THEN** the caller or surface owner decides continuation, failure, output format, output channel and exit behavior

### Requirement: DiagnosticCode owns identity and canonical details
`docnav-diagnostics` MUST own `DiagnosticCode`, grouped code families, each code's canonical details object and projection metadata. Other Docnav crates MUST use those diagnostics-owned identities and MUST NOT redefine protocol, readable, adapter or navigation input resolution diagnostic code identities.

#### Scenario: Diagnostic code owns surface error identity
- **WHEN** a diagnostic record is rendered as a fatal error, protocol error code, stderr line, readable error field or other surface field
- **THEN** mechanical identity is derived from the record's `DiagnosticCode`
- **THEN** the surface field does not become the source of identity for the internal channel

#### Scenario: Diagnostic code owns canonical details
- **WHEN** a caller creates a diagnostic record for a specific `DiagnosticCode`
- **THEN** record details conform to the canonical details object structure for that code
- **THEN** surface projection maps from that canonical details object

### Requirement: Boundary surfaces project diagnostic records
Modules that discover problems MUST push diagnostic records into the channel. Boundary surfaces MUST read those records and project them according to their owner contract.

#### Scenario: Runtime module writes but does not format final output
- **WHEN** core runtime, navigation input resolution, adapter selection or selected adapter dispatch discovers a problem
- **THEN** that module records what happened, its impact, canonical details and source in the diagnostic stack
- **THEN** that module does not own final user-visible formatting unless it is also the boundary surface owner

#### Scenario: Boundary surface projects records to its own contract
- **WHEN** CLI, protocol surface or readable output reaches an output boundary
- **THEN** the boundary reads relevant diagnostic stack records
- **THEN** the boundary projects records according to `docs/cli.md`, `docs/protocol.md`, `docs/output.md` or `docs/adapter-contract.md`

### Requirement: Legacy diagnostic sources are fully migrated
Existing error fact sources MUST fully migrate to diagnostic channel records and diagnostics-owned projections. The completed implementation MUST NOT retain a legacy parallel diagnostic fact source.

#### Scenario: Stable error projection uses diagnostic code
- **WHEN** a fatal diagnostic is rendered or serialized as a stable surface error
- **THEN** the target surface error code is derived from `DiagnosticCode`
- **THEN** no legacy stable error object remains as an owning fact model after migration completes

### Requirement: Protocol error rules JSON is removed
`docs/protocol/error-rules.json` MUST be deleted as a rule source. Protocol error code and details validation MUST consume `DiagnosticCode` protocol projections from `docnav-diagnostics`, while protocol docs, schema, examples and tests remain validation and presentation materials.

#### Scenario: Protocol crate uses diagnostics code directly
- **WHEN** `docnav-protocol` needs to render, validate or categorize a protocol-visible diagnostic
- **THEN** it depends on `docnav-diagnostics` and uses `DiagnosticCode` or an explicit diagnostics-owned protocol projection
- **THEN** it does not maintain a separate protocol-local required-details rule source

### Requirement: Diagnostic channel changes update validation materials
Changes to diagnostic channel semantics or surface projection MUST update relevant owner docs, JSON Schema, examples, fixtures and tests in the same implementation work.

#### Scenario: Protocol JSON projection is validated
- **WHEN** a document operation is rendered as `protocol-json`
- **THEN** stdout follows the protocol response schema owned by the protocol docs
- **THEN** protocol-visible diagnostic fields are derived from diagnostic channel records

#### Scenario: Readable output projection is validated
- **WHEN** a document operation is rendered as `readable-view` or `readable-json`
- **THEN** primary failure diagnostics are rendered from diagnostic channel records when the operation fails
- **THEN** readable output remains separate from the protocol response envelope

### Requirement: Raw protocol exposes structured facts and readable output organizes them
Docnav raw protocol MUST express machine-readable facts as structured JSON fields. Readable output MUST organize those facts into display text, summary and layout, and MUST NOT become the raw protocol fact source.

#### Scenario: 请求预算使用 canonical limit
- **WHEN** document operation request carries pagination budget
- **THEN** canonical protocol argument is `limit`
- **THEN** `limit` is a positive integer and budget-unit interpretation remains adapter-owned
- **THEN** schema, examples, typed arguments and operation handling use `limit`

#### Scenario: 导航条目分离 ref、事实字段和 display
- **WHEN** outline entries or find matches return
- **THEN** each raw protocol item keeps `ref` as an adapter-owned opaque string
- **THEN** available label, location, summary, excerpt, rank, cost and metadata facts use structured fields
- **THEN** readable output owns final display rows

#### Scenario: error 投影保留结构化 details
- **WHEN** protocol output returns `ok: false`
- **THEN** `error.code` selects a documented structured `error.details` shape
- **THEN** `error.message` and `error.guidance` remain display fields
- **AND WHEN** readable output returns failure
- **THEN** readable error projection preserves primary diagnostic structured details
