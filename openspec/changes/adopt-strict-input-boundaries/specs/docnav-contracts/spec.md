本 spec delta 定义 `adopt-strict-input-boundaries` 对 `docnav-contracts` 的目标变更：把 Docnav 的长期公共契约改为 strict-by-default，并把 AI 友好性放到诊断与修复建议上。

## ADDED Requirements

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
Docnav public failure surfaces MUST 由 failed request 的一个 primary `DiagnosticRecord` 驱动。Diagnostics owner MUST 使用稳定字段名定义 canonical public structure：

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

## MODIFIED Requirements

### Requirement: DiagnosticCode owns identity and canonical details
`docnav-diagnostics` MUST own stable diagnostic identities、grouped code families、primary `DiagnosticRecord` field rules、canonical `details` list shapes 和 projection metadata。Public failure surfaces MUST consume one primary `DiagnosticRecord` for the failed request。`DiagnosticCode` 保持为 record `code` field 使用的 stable machine identity；public surface contract 是 `DiagnosticRecord` projection。

#### Scenario: Diagnostic code 提供 primary record identity
- **WHEN** implementation 按 purpose、producer 或 projection family 组织 diagnostic codes
- **THEN** top-level diagnostic identity 映射到 primary `DiagnosticRecord.code`
- **THEN** `docnav-diagnostics` 外部 callers 使用 diagnostics-owned constructors 或 mappings 创建 primary records

#### Scenario: Diagnostic details 从属于 primary record
- **WHEN** caller 为 specific diagnostic identity 创建 `DiagnosticRecord`
- **THEN** required fields、optional fields 和 allowed `details` list keys 遵循 diagnostics-owned rules
- **THEN** field issues、config issues、typed validation failures 和 candidate failures 从属于该 record

#### Scenario: Projection rules 来自 primary record
- **WHEN** implementation renders protocol、readable、CLI、adapter 或 stderr failure output
- **THEN** visible error code、message、details、guidance 和 exit behavior 来自 primary `DiagnosticRecord` 与 owning surface policy
- **THEN** schema、examples 和 fixtures 验证 projection；rule source 仍由 owner contract 提供

### Requirement: Boundary surfaces project diagnostic records
发现 public failure 的 modules MUST 返回或构造足够的 diagnostics-owned facts，使 owning boundary 能投影一个 primary `DiagnosticRecord`。Boundary surfaces MUST 按 owner contract 投影该 primary record。Internal events 由内部流程消费；operation/output owner 可以把需要公开表达的状态建模为 explicit business fields 或 status。

#### Scenario: Runtime module 报告 facts，boundary surface 拥有 final output
- **WHEN** core runtime、navigation input resolution、adapter selection 或 selected adapter dispatch 发现 strict failure
- **THEN** 该 module 报告自己拥有的 diagnostic identity、owner、location、received value、expected shape、guidance 和 subordinate details
- **THEN** final user-visible formatting 由 boundary surface owner 拥有

#### Scenario: Boundary surface 投影一个 primary record
- **WHEN** CLI、protocol surface 或 readable output 到达 failure output boundary
- **THEN** boundary 投影一个 primary `DiagnosticRecord`
- **THEN** stdout、stderr、process exit 和 envelope shape 遵循 `docs/cli.md`、`docs/protocol.md`、`docs/output.md` 或 `docs/adapter-contract.md`

#### Scenario: Surface docs 保持 diagnostics ownership single
- **WHEN** protocol、readable、CLI、adapter、schema 或 example docs 描述 diagnostic output
- **THEN** 这些 docs 描述各自 surface 的 display、mapping、stdout/stderr placement、envelope shape 或 exit behavior
- **THEN** diagnostics-owned identity、canonical record fields 和 subordinate details 由 diagnostics docs/specs 拥有

### Requirement: Navigation config loading only provides navigation source values
Core MUST provide config source descriptors/paths to `docnav-navigation`; `docnav-navigation` MUST read config sources and pass valid config values into navigation input resolution source values. Missing default config files mean the corresponding layer has no config source. Explicit config sources that are missing, unreadable or not files MUST fail. Existing config files that contain invalid JSON, non-object root values, unknown top-level fields, unknown `defaults` fields or invalid known values MUST fail. Runtime config handling uses source reading, fixed field projection and navigation input resolution validation; config schema/example remains documentation and validation material.

`docnav-navigation` MUST handle JSON reading and fixed field projection; navigation input resolution owns source priority。`options` object values 是 explicit adapter-owned native option source，并 MUST 由 selected adapter typed-field declarations 校验或拒绝。

#### Scenario: 未覆盖默认配置文件缺失时使用下一级默认值
- **WHEN** 项目级或用户级配置文件不存在
- **AND** 调用方没有显式覆盖该层配置来源
- **THEN** Docnav 继续按其余来源解析默认值
- **THEN** 缺失文件不产生配置源输入

#### Scenario: 显式覆盖配置路径缺失时失败
- **WHEN** 调用方显式提供的配置来源不存在或不可读
- **THEN** Docnav 返回配置输入错误
- **THEN** 用户级配置和内置默认值不被用作绕过该错误的成功路径
- **THEN** operation handler 不执行

#### Scenario: Config schema 不作为 runtime 校验入口
- **WHEN** `docs/schemas/docnav-markdown-config.schema.json` 不存在于运行环境
- **AND** 调用方执行 Markdown document operation
- **THEN** Docnav 仍按配置源读取、字段投影和 navigation input resolution 处理本次调用
- **THEN** runtime 依赖配置源读取、字段投影和 navigation input resolution

#### Scenario: 未知配置字段失败
- **WHEN** Docnav 读取到 schema 未声明但 JSON 语法有效且顶层为 object 的配置字段
- **AND** 该字段不属于 adapter-owned `options` object
- **THEN** Docnav 返回配置输入错误
- **THEN** 诊断包含字段路径和配置路径
- **THEN** 已知字段不被用于继续成功路径

#### Scenario: JSON 语法无效时失败
- **WHEN** Docnav 读取到语法无效的 JSON 配置文件
- **THEN** Docnav 返回配置输入错误
- **THEN** Docnav 不跳过该项目级配置源后继续合并用户级配置

### Requirement: Diagnostic channel changes update validation materials
Changes to diagnostic channel semantics or surface projection MUST update the relevant owner docs, JSON Schema, examples, fixtures and tests in the same implementation work. Invalid public input fixtures MUST assert strict failure and actionable diagnostic guidance.

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

### Requirement: Legacy diagnostic sources are fully migrated
Existing public error fact sources 和 strict-input diagnostic families MUST migrate to diagnostics-owned primary `DiagnosticRecord` construction and projection。Document success output 使用 owning success payload shape。Internal logging、tracing 或 owner-scoped status 可以记录 non-fatal events；document success output 不承载通用诊断通道。

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

## REMOVED Requirements

### Requirement: Rust CLI 参数解析必须服务 AI 维护和一次成功调用
**Reason**: 新契约把 Rust CLI 参数解析定义为 strict public input boundary，并把 AI 友好性落实到 actionable diagnostics。

**Migration**: 使用 strict core CLI、protocol request 和 config source validation 作为文档操作前置契约。Documentation 和 tests 证明 invalid caller input 返回 stable、repairable diagnostics，valid inputs 共享 core-linked document operation pipeline。

### Requirement: Runtime problems flow through a request-local diagnostic stack
**Reason**: 新契约将 public failure surface 收敛为每个 failed request 一个 primary `DiagnosticRecord`，相关结构化数据挂载到该诊断的 `details`。

**Migration**: 使用 primary `DiagnosticRecord` 表达 public failure。Internal logging 或 tracing 可按实现 owner 需要保留事件历史；protocol/readable/CLI contracts 以 primary record projection 为准。

### Requirement: Diagnostic stack provides scoped checkpoints and LIFO retrieval
**Reason**: Stack ids、checkpoints、marks 和 retrieval order 属于实现机制。Strict public contract 使用每个 failed request 一个 primary `DiagnosticRecord`。

**Migration**: 当 failure 需要 related structured items 时，使用 primary `DiagnosticRecord` 加从属 details lists，例如 `candidate_failures`。Internal tracing 可保留 richer event history；public projection 不暴露 DiagnosticId、mark 或 LIFO semantics。
