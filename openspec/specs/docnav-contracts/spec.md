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
`docnav` MUST 负责项目根解析、核心配置 source loading、core release static adapter registry、adapter inspection、协议字段校验、输出模式和错误映射。默认 document operation path MUST use adapter-layer workspace crates registered in the current core release as adapter implementations. Adapter layer ownership MUST remain a code and contract boundary rather than a separate default distribution boundary. Internal operation orchestration, adapter selection and navigation input resolution MUST be owned by `docnav-navigation`, while adapter layer interface definitions, static descriptors and shared contract types MUST be owned by `docnav-adapter-contracts`. `docnav-navigation` MUST prepare requests and dispatch operation handlers; it MUST NOT act as an adapter loader. Independent adapter packages、external adapter executables、command paths and historical adapter artifact records MUST NOT become default document operation implementation sources.

#### Scenario: 读取 Markdown outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md`
- **THEN** `docnav` 从当前 core release static adapter registry 选择 adapter implementation
- **THEN** `docnav` resolves `docs/guide.md` to an absolute path before navigation dispatch
- **THEN** `docnav` 将 page、limit 和 merged native options 等参数准备为 operation input
- **THEN** `docnav` 不从 adapter metadata、配置或隐式默认值生成格式专属 `options`
- **THEN** adapter 生成的 ref 和 display 被保留到阅读输出

#### Scenario: adapter contract owner remains smaller than operation orchestration
- **WHEN** adapter crate 接入默认 document operation path
- **THEN** adapter crate 依赖 `docnav-adapter-contracts` 暴露的 adapter layer interface definitions
- **THEN** `docnav-navigation` 负责组合 `outline/read/find/info` 流程
- **THEN** adapter crate 不需要依赖独立 runtime SDK、dynamic registration API 或 adapter direct CLI 才能参与默认 document operation

#### Scenario: adapter interface uses operation-handler granularity
- **WHEN** adapter crate 实现 adapter layer interface
- **THEN** adapter handle exposes static descriptor metadata, probe check, source-level native option registry entries and `outline/read/find/info` operation handlers through `docnav-adapter-contracts`
- **THEN** parser、ref、navigation、pagination 和 native option semantics remain adapter-owned inside those handlers
- **THEN** `docnav-navigation` dispatches the selected operation handler instead of composing adapter ref splitter、locator、format probe validation or parser/navigation primitives across the adapter/core boundary

#### Scenario: native option registry feeds adapter handoff
- **WHEN** the source-level native option registry includes the Markdown `options.max_heading_level` entry
- **AND** request or config sources provide `options.max_heading_level`
- **THEN** navigation input resolution merges the value with source and registry metadata
- **THEN** the linked Markdown handler receives the final option value and validates support, type and range semantics

#### Scenario: navigation layer is not an adapter loader
- **WHEN** `docnav-navigation` dispatches an operation
- **THEN** it receives a selected linked adapter handle from core registry/routing
- **THEN** it prepares the request and calls the operation handler
- **THEN** it does not load executables, resolve command paths, or mutate runtime adapter registration

#### Scenario: adapter diagnostic boundary excludes exit code API
- **WHEN** linked adapter handling fails
- **THEN** the adapter layer returns structured diagnostic facts
- **THEN** core/output owns protocol/readable projection and final process exit code
- **THEN** adapter contract does not expose exit-code return semantics

#### Scenario: core release static adapter registry inspection
- **WHEN** 调用方执行 `docnav adapter list`
- **THEN** `docnav` 输出当前 core release static adapter registry 中 adapter library 的身份、版本和支持格式

#### Scenario: dynamic adapter management commands are not default surface
- **WHEN** 调用方执行 `docnav adapter install <source>`
- **OR** 调用方执行 `docnav adapter register <source>`
- **OR** 调用方执行 `docnav adapter update <adapter-id>`
- **OR** 调用方执行 `docnav adapter remove <adapter-id>`
- **THEN** `docnav` 不把这些命令作为有效默认 adapter management surface
- **THEN** 这些命令不会改变当前 release 的 static adapter registry

#### Scenario: External adapter artifact is not a default implementation source
- **WHEN** 项目配置、用户配置或历史 adapter record 指向 external adapter executable
- **AND** 调用方执行 document operation
- **THEN** `docnav` 不把该 executable 当作 adapter implementation source
- **THEN** adapter implementation 只来自当前 core release static adapter registry

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

### Requirement: DiagnosticCode owns identity and canonical details
`docnav-diagnostics` MUST provide stable diagnostic identities、grouped code families、primary `DiagnosticRecord` construction/validation rules 和 canonical `details` list helper materials。It MUST NOT own operation outcome、surface output format、exit behavior、adapter selection、strict input routing、protocol envelope、readable wrapping、CLI surface 或 public projection policy。Public failure surfaces MUST consume one primary `DiagnosticRecord` for the failed request。`DiagnosticCode` 保持为 record `code` field 使用的 stable machine identity；public surface contract 是 owner-specific `DiagnosticRecord` projection。

#### Scenario: Diagnostic code 提供 primary record identity
- **WHEN** implementation 按 purpose、producer 或 projection family 组织 diagnostic codes
- **THEN** top-level diagnostic identity 映射到 primary `DiagnosticRecord.code`
- **THEN** `docnav-diagnostics` 外部 callers 使用 helper-provided constructors 或 mappings 创建 primary records

#### Scenario: Diagnostic details 从属于 primary record
- **WHEN** caller 为 specific diagnostic identity 创建 `DiagnosticRecord`
- **THEN** required fields、optional fields 和 allowed `details` list keys 遵循 diagnostic helper record rules
- **THEN** field issues、config issues、typed validation failures 和 candidate failures 从属于该 record

#### Scenario: Projection rules 来自 primary record
- **WHEN** implementation renders protocol、readable、CLI、adapter 或 stderr failure output
- **THEN** visible error code、message、details、guidance 和 exit behavior 来自 primary `DiagnosticRecord` 与 owning surface policy
- **THEN** schema、examples 和 fixtures 验证 projection；rule source 仍由 owner contract 提供

### Requirement: Boundary surfaces project diagnostic records
发现 public failure 的 modules MUST 返回或构造足够的 diagnostic record facts，使 owning boundary 能投影一个 primary `DiagnosticRecord`。Boundary surfaces MUST 按 owner contract 投影该 primary record。Internal events 由内部流程消费；operation/output owner 可以把需要公开表达的状态建模为 explicit business fields 或 status。

#### Scenario: Runtime module 报告 facts，boundary surface 拥有 final output
- **WHEN** core runtime、navigation input resolution、adapter selection 或 selected adapter dispatch 发现 strict failure
- **THEN** 该 module 报告自己拥有的 diagnostic identity、owner、location、received value、expected shape、guidance 和 subordinate details
- **THEN** final user-visible formatting 由 boundary surface owner 拥有

#### Scenario: Boundary surface 投影一个 primary record
- **WHEN** CLI、protocol surface 或 readable output 到达 failure output boundary
- **THEN** boundary 投影一个 primary `DiagnosticRecord`
- **THEN** stdout、stderr、process exit 和 envelope shape 遵循 `docs/cli.md`、`docs/protocol.md`、`docs/output.md` 或 `docs/adapter-contract.md`

#### Scenario: Surface docs keep helper and projection ownership separate
- **WHEN** protocol、readable、CLI、adapter、schema 或 example docs 描述 diagnostic output
- **THEN** 这些 docs 描述各自 surface 的 display、mapping、stdout/stderr placement、envelope shape 或 exit behavior
- **THEN** stable identity、canonical record field invariants 和 subordinate details helpers 由 `docnav-diagnostics` helper boundary 提供
- **THEN** public projection rules remain owned by protocol/output/CLI/adapter/schema/example surface docs

### Requirement: Legacy diagnostic sources are fully migrated
Existing public error fact sources 和 strict-input diagnostic families MUST migrate to helper-backed primary `DiagnosticRecord` construction/validation and surface-owned projection。Document success output 使用 owning success payload shape。Internal logging、tracing 或 owner-scoped status 可以记录 non-fatal events；document success output 不承载通用诊断通道。

#### Scenario: Stable error projection 使用 primary record
- **WHEN** a fatal or blocking diagnostic is rendered or serialized as a stable surface error
- **THEN** the target surface error fields derive from the primary `DiagnosticRecord`
- **THEN** no parallel public error object owns a second field shape for the same failed request

#### Scenario: 成功路径诊断场景改为失败或 owner fields
- **WHEN** public input boundaries 收到 unknown argv、skipped explicit config、explicit adapter fallback 或 undeclared native option cases
- **THEN** the owning boundary returns a failure diagnostic
- **THEN** valid success output follows the owning success payload shape

#### Scenario: Direct stderr text 补充 primary record
- **WHEN** a Rust entry point rejects command shape, fails metadata/schema validation or hits output write failure before normal document output
- **THEN** the entry point creates or maps a primary `DiagnosticRecord`
- **THEN** any stderr text is supplemental and follows the owning surface policy

### Requirement: Protocol error rules JSON is removed
`docs/protocol/error-rules.json` MUST be deleted as a rule source. Protocol error code and details validation MUST consume `DiagnosticCode` protocol projections from `docnav-diagnostics`, while protocol docs, schema, examples and tests remain validation and presentation materials.

#### Scenario: Protocol crate uses diagnostics code directly
- **WHEN** `docnav-protocol` needs to render, validate or categorize a protocol-visible diagnostic
- **THEN** it depends on `docnav-diagnostics` and uses `DiagnosticCode` or an explicit diagnostics-owned protocol projection
- **THEN** it does not maintain a separate protocol-local required-details rule source

### Requirement: Diagnostic channel changes update validation materials
Changes to diagnostic helper semantics or surface projection MUST update the relevant owner docs, JSON Schema, examples, fixtures and tests in the same implementation work. Invalid public input fixtures MUST assert strict failure and actionable diagnostic guidance.

#### Scenario: Protocol JSON projection 被验证
- **WHEN** a document operation is rendered as `protocol-json`
- **THEN** stdout follows the protocol response schema owned by the protocol docs
- **THEN** any protocol-visible diagnostic fields or errors are derived from the primary `DiagnosticRecord`

#### Scenario: Readable output projection 被验证
- **WHEN** a document operation is rendered as `readable-view` or `readable-json`
- **THEN** fatal or blocking diagnostics are rendered as readable error output according to the readable output owner
- **THEN** success output follows the readable success payload schema
- **THEN** readable output remains separate from the protocol response envelope

#### Scenario: Machine-readable projection 被验证
- **WHEN** core adapter inspection、manifest/probe-shaped metadata validation or protocol output writes machine-readable output
- **THEN** stdout follows the owning metadata or protocol response schema
- **THEN** failure output is derived from the primary `DiagnosticRecord` according to that surface policy

#### Scenario: Strict diagnostics 需要 validation material
- **WHEN** implementation defines or changes a strict input error
- **THEN** owner docs describe the failing condition, error code, details shape and guidance
- **THEN** schema, examples, fixtures and tests are updated in the same implementation work

### Requirement: Raw protocol exposes structured facts and readable output organizes them
Docnav raw protocol MUST 将机器可读事实表达为结构化 JSON 字段。Readable output MUST 把这些事实组织成面向阅读的 display text、summary 和布局，并且不成为 raw protocol fact source。

#### Scenario: 请求预算使用 canonical limit
- **WHEN** document operation request 携带分页预算
- **THEN** canonical protocol argument 是 `limit`
- **THEN** `limit` 是 positive integer，预算单位解释仍归 adapter 所有
- **THEN** schema、examples、typed arguments 和 operation handling 使用 `limit`

#### Scenario: cost 在协议中结构化，在 readable 输出中摘要化
- **WHEN** operation result 报告 cost
- **THEN** raw protocol 携带结构化 `cost.measurements[]`
- **THEN** 每个 measurement 包含机器可读的 `unit` 和 `value`
- **THEN** readable output 可以从这些 measurements 派生成紧凑成本摘要

#### Scenario: 导航条目分离 ref、事实字段和 display
- **WHEN** outline entries 或 find matches 返回
- **THEN** 每个 raw protocol item 保留 `ref` 作为 adapter-owned opaque string
- **THEN** label、location、summary、excerpt、rank、cost 和 metadata 等 item facts 在可用时使用结构化字段
- **THEN** readable output 拥有最终 display row

#### Scenario: info result 将 metadata 与摘要分离
- **WHEN** info 返回 document 或 adapter facts
- **THEN** raw protocol 使用结构化 document 和 adapter metadata 字段
- **THEN** readable output 可以把这些字段呈现为紧凑摘要

#### Scenario: failure projection 保留 structured details
- **WHEN** protocol output 返回 `ok: false`
- **THEN** `error.code`、`error.details`、`error.message` 和 `error.guidance` derive from the primary `DiagnosticRecord`
- **THEN** readable failure output 通过 readable error fields 投影同一个 primary diagnostic

#### Scenario: continuation owner 保持稳定
- **WHEN** 引入 structured fields
- **THEN** `page` 仍是 protocol-owned next-page integer or null

### Requirement: Public input boundaries must be strict by default
Docnav public input boundaries MUST 默认拒绝 invalid caller intent。Public input boundaries 包括 core CLI argv、protocol request fields and arguments、explicit adapter selection、explicit config source declarations、present config files、explicit document paths、explicit refs、operation arguments 和 declared native options。

AI-friendly behavior MUST 通过 precise diagnostics、stable error identities 和 actionable repair guidance 提供。Internal discovery MAY 在 caller 未显式声明失败候选时继续；successful output MUST 描述 successful document operation。

#### Scenario: Invalid caller input 快速失败
- **WHEN** caller 提供 unknown argv、extra positional input、unknown protocol fields、unknown config fields 或 undeclared native options
- **THEN** owning boundary 返回 input 或 config diagnostic
- **THEN** owning boundary 在 document operation execution 前返回
- **THEN** diagnostic 在可用时标出 input location，并提供 expected shape 或 repair guidance

#### Scenario: Internal discovery 可继续且 success payload 稳定
- **WHEN** Docnav 执行 automatic adapter discovery，且 caller 未声明 adapter id
- **AND** internal candidate 在 manifest、schema 或 probe validation 失败
- **THEN** Docnav may continue evaluating later candidates
- **THEN** 后续 candidate 成功时，successful output 只描述 successful document operation
- **THEN** 全部 candidates 失败时，failure output 包含 candidate failure list

#### Scenario: Explicit selection failure 返回 owning diagnostic
- **WHEN** caller 显式声明 adapter id、config path、operation argument、ref 或 path
- **AND** declared input 的 validation 或 resolution 失败
- **THEN** Docnav 返回 owning diagnostic
- **THEN** 该 diagnostic 是 declared input 的 final outcome

### Requirement: Public failures must use a single primary DiagnosticRecord
Docnav public failure surfaces MUST 由 failed request 的一个 primary `DiagnosticRecord` 驱动。`docnav-diagnostics` MUST remain a diagnostic/error model helper boundary for stable identities、record construction/validation 和 details helper materials。Protocol、output、CLI、adapter、schema 和 example surface owners MUST define their own projection、framing、channel 和 exit behavior from that primary record。Canonical primary record structure includes：

- `code`: 必需的 stable machine-readable code。
- `message`: 必需的简短 human-readable summary。
- `owner`: 必需的 owning boundary 或 stage。
- `location`: owner 可定位时提供的 input location object。
- `received`: useful 且 safe to expose 时提供的 received value 或 token。
- `expected`: 可用时提供的 expected shape 或 accepted values。
- `guidance`: actionable repair steps。
- `details`: optional object，只包含本次 failure 需要的 subordinate structured lists，例如 `field_issues`、`config_issues`、`typed_validation_failures` 或 `candidate_failures`。

Invalid caller input diagnostics MUST 在 owner 可定位 input location 时包含 `location`，在 owner 可描述 expected shape 或 values 时包含 `expected`，并包含至少一个 `guidance` item。Diagnostic details MAY 包含 field issue lists、config issue lists、typed validation failure lists 或 candidate failure lists。这些 related details remain subordinate to the primary `DiagnosticRecord`。Candidate discovery all-failed details MUST 表达为 candidate failure list。

#### Scenario: Strict input problem 返回单个诊断
- **WHEN** operation 在 public boundary 遇到 invalid caller input
- **THEN** owning boundary 返回 failed request 的一个 primary `DiagnosticRecord`
- **THEN** diagnostic 在可用时标出 invalid input location、received value、expected shape 和 guidance
- **THEN** request 返回 diagnostic outcome

#### Scenario: 全部 candidate 失败时报告 discovery failure list
- **WHEN** automatic adapter discovery 评估多个 candidates 且全部失败
- **THEN** Docnav 返回一个 adapter-selection 或 `FORMAT_UNKNOWN` diagnostic
- **THEN** failure output 包含 candidate failure list
- **THEN** 每个 candidate failure item 使用 bounded summary shape
- **THEN** candidate failure list 从属于 primary `DiagnosticRecord`

### Requirement: Shared text cost calculator exposes simple text-to-cost functions
Docnav shared helpers MUST provide text cost calculator functions with a uniform interface: each function accepts plain text input and returns a protocol-compatible cost measurement for the function-defined cost type. The helper MUST NOT require adapter identity, format identity, path, ref, operation, parser state, output mode, unit parameters, scope, tokenizer policy, encoding, model preset, or strategy objects to calculate cost.

#### Scenario: Any component can calculate cost from selected text
- **WHEN** a Docnav component has already selected or produced plain text
- **AND** the component chooses which text cost helper function to call
- **THEN** it can call that function with the text as the only required input
- **THEN** the helper returns a deterministic cost measurement with the function-defined `unit` and calculated `value`
- **THEN** the caller remains responsible for whether and where that measurement is exposed in protocol, readable output, tests, diagnostics, or internal tooling

#### Scenario: First helper functions and units are fixed
- **WHEN** a caller uses the initial shared text cost calculator API
- **THEN** `line_cost(text: &str) -> Measurement` returns a measurement with unit `lines`
- **THEN** `byte_cost(text: &str) -> Measurement` returns a measurement with unit `bytes`
- **THEN** `token_cost(text: &str) -> Measurement` returns a measurement with unit `tokens`
- **THEN** each function accepts the selected plain text as its only required input
- **THEN** the helper does not attach scope to the returned measurement

#### Scenario: Adapter chooses functions and calls helper directly
- **WHEN** an adapter needs cost measurements for output it owns
- **THEN** the adapter chooses which helper functions to call and calls those functions directly
- **THEN** core, output formatting, and the helper do not choose adapter cost measurements on the adapter's behalf

#### Scenario: Token cost uses the same text-only interface
- **WHEN** a caller needs token cost for already selected plain text
- **THEN** it calls the token cost helper function with that text as the only required input
- **THEN** the token helper uses `tiktoken-rs` `o200k_base` ordinary plain-text tokenization to return a deterministic cost measurement with a token unit and calculated token value
- **THEN** text that looks like a special token is counted as plain output text rather than as a tokenizer control token

#### Scenario: Helper does not own text selection
- **WHEN** an adapter, output tool, or validation script needs cost for a document fragment
- **THEN** that caller chooses or produces the fragment text before invoking the helper
- **THEN** the helper does not parse the source document, resolve refs, apply native options, truncate content, or decide pagination budget semantics

