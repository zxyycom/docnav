本 spec delta 定义 `adopt-strict-input-boundaries` 对 `core-cli` 的目标变更：确立核心 CLI 公共输入的严格拒绝规则并提供可执行诊断。

## ADDED Requirements

### Requirement: 核心 CLI 必须严格拒绝无效显式输入
`docnav` core CLI MUST 将 caller-provided argv、operation arguments、explicit adapter selection、explicit path/ref/query/page/limit/output values 和 explicit config declarations 作为 strict public input。Unknown flags、extra positional arguments，以及 selected command 或 operation 不支持的 flags MUST 在 document execution 前返回 input diagnostic。

Core CLI MUST 将 valid document argv 映射为 raw navigation command 或等价 handoff input，再连同 config source descriptors/paths 和 registry 交给 `docnav-navigation`；navigation input resolution 负责 adapter selection 和 internal operation request construction，core/output 负责 output dispatch。CLI boundary 可见的 invalid input MUST 在 document execution 前完成 strict rejection。Direct CLI parser/help implementation MAY continue to use `clap`。

Input diagnostics MUST 包含 stable error code、argument 或 field location、可安全暴露的 received value/token、expected command shape 或 accepted values，并在 owner 可生成时包含至少一个 actionable repair hint。

#### Scenario: 未知 argv 被拒绝
- **WHEN** 调用方执行 `docnav outline docs/guide.md --future extra --output readable-json`
- **THEN** `docnav` 返回输入错误或 `INVALID_REQUEST`
- **THEN** navigation input resolution 和 document operation 不执行
- **THEN** 诊断标出未知 argv token
- **THEN** 诊断提供可接受参数或修复建议

#### Scenario: 多余 positional 被拒绝
- **WHEN** 调用方执行 `docnav outline docs/guide.md extra.md`
- **THEN** `docnav` 返回输入错误或 `INVALID_REQUEST`
- **THEN** 诊断标出多余 positional 的位置和值
- **THEN** document operation 不执行

#### Scenario: 当前 operation 不支持的已知参数被拒绝
- **WHEN** 调用方执行 `docnav info docs/guide.md --page 2`
- **THEN** `docnav` 返回输入错误或 `INVALID_REQUEST`
- **THEN** 诊断说明该参数不适用于 `info`
- **THEN** 诊断指向支持该参数的 operation 或建议删除该参数

#### Scenario: 参数值类型无效时被拒绝
- **WHEN** 调用方执行 `docnav read docs/guide.md --ref intro --limit nope`
- **THEN** `docnav` 返回输入错误或 `INVALID_REQUEST`
- **THEN** 诊断标出 `--limit` 的 received value
- **THEN** 诊断说明该参数的 expected value shape

#### Scenario: 有效 argv 仍进入共享语义管道
- **WHEN** 调用方执行有效的 `docnav outline/read/find/info` CLI 命令
- **THEN** document CLI input 映射为 canonical document operation input 或等价 semantic request
- **THEN** `docnav-navigation` 负责 adapter selection 和 internal operation request 构造
- **THEN** output mode 分流使用共享输出逻辑
- **THEN** CLI 不创建独立业务参数解释路径

#### Scenario: Help 不执行业务
- **WHEN** 调用方执行 `docnav --help`
- **OR** 执行 core 子命令 help
- **THEN** 输出列出可用命令、关键参数、默认值或可选值
- **THEN** 该命令不读取文档、不选择 adapter、不 dispatch linked adapter handler

## MODIFIED Requirements

### Requirement: adapter 选择必须先处理声明式 adapter intent
`docnav` MUST use strict adapter selection semantics for caller-declared adapter intent. When an adapter id is explicitly provided by CLI argv or effective config, failure to resolve, load, validate, or probe that adapter MUST return an adapter selection error as the final adapter-selection outcome.

When no caller-declared adapter id exists, `docnav` MAY perform automatic adapter discovery by evaluating registry candidates. Automatic candidate failures MAY be appended to an internal failure list while discovery continues. If a later candidate succeeds, the successful output MUST describe only the successful document operation. If all automatic candidates fail, `docnav` MUST return `FORMAT_UNKNOWN` or the owning adapter-selection diagnostic with a candidate failure list.

#### Scenario: 显式 adapter 解析失败时失败
- **WHEN** 调用方传入 `--adapter docnav-markdown` 但 registry 中无法解析该 adapter 记录
- **THEN** `docnav` 返回 adapter selection error
- **THEN** 错误 details 包含 adapter id、selection_source、stage 和 reason
- **THEN** adapter selection 以该诊断结束

#### Scenario: 配置声明 adapter 失败时失败
- **WHEN** effective config 提供 `defaults.adapter`
- **AND** 该 adapter 的 manifest、当前契约校验或 probe 失败
- **THEN** `docnav` 返回 adapter selection error
- **THEN** 错误 details 指向配置来源、adapter 候选失败阶段和原因
- **THEN** adapter selection 以该诊断结束

#### Scenario: 自动发现候选失败后可继续
- **WHEN** 调用方没有通过 CLI 或配置声明 adapter id
- **AND** registry 遍历中的候选 adapter manifest 或 probe 输出字段缺失、类型不符、语义校验失败或返回 `supported: false`
- **THEN** `docnav` 可以追加候选失败条目
- **THEN** `docnav` 可以继续评估后续候选
- **THEN** 若后续候选成功，成功输出只描述成功的 document operation

#### Scenario: 自动发现全部失败
- **WHEN** 没有 adapter 能校验目标文档
- **THEN** `docnav` 返回 `FORMAT_UNKNOWN`
- **THEN** failure output 包含候选失败列表
- **THEN** 候选失败列表是 JSON 数组
- **THEN** 每条失败项只包含 adapter_id、stage 和 reason

### Requirement: 输出模式必须按协议层和阅读层分离
`docnav --output protocol-json` MUST output the raw protocol response envelope for document operations. `readable-view` and `readable-json` MUST remain readable-layer results derived from the same typed readable payload. Invalid caller input MUST be projected as an error.

Successful document output MUST follow the owning operation payload schema. Rejected argv, invalid config sources and automatic discovery all-failed lists are represented by failure diagnostics; discovery attempts that later succeed remain internal state.

Core CLI document output enum/help/config MUST expose only `readable-view`, `readable-json` and `protocol-json`. Help, version and other non-document plain text output MUST use `PlainText` or an equivalent explicitly separated channel.

#### Scenario: readable-json outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md --output readable-json`
- **THEN** 输出包含 entries、page 等 outline readable fields
- **THEN** 输出不包含 protocol envelope 字段

#### Scenario: 默认 readable-view outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md` 且未传入 `--output`
- **THEN** 输出使用 readable-view pretty JSON header
- **THEN** JSON header 包含 entries 和 page
- **THEN** 输出不包含 protocol envelope 字段

#### Scenario: invalid argv 在 readable-json 中失败
- **WHEN** 调用方执行带有未知参数的 readable-json document 命令
- **THEN** `docnav` 返回输入错误或 `INVALID_REQUEST`
- **THEN** stdout 按 readable error owner 输出结构化失败诊断
- **THEN** stderr 只承载 owner 允许的 supplemental human text
- **THEN** stdout 不包含成功 operation payload
- **THEN** 输出使用 readable error shape

#### Scenario: invalid argv 在 protocol-json 中失败
- **WHEN** 调用方执行带有未知参数的 protocol-json document 命令
- **THEN** `docnav` 返回输入错误或 `INVALID_REQUEST`
- **THEN** stdout 按 protocol failure response contract 输出
- **THEN** stdout 不包含成功 result
- **THEN** stdout 使用 protocol failure shape

#### Scenario: document output 值按三种模式校验
- **WHEN** 调用方执行 `docnav outline docs/guide.md --output <invalid-output>`
- **THEN** `docnav` 返回 `INVALID_REQUEST`
- **THEN** CLI help 只列出 readable-view、readable-json 和 protocol-json
- **THEN** `docnav` 在 navigation input resolution 和 document operation 执行前返回

### Requirement: Core CLI 必须复用共享 helper 且保留 core policy owner
`docnav` core CLI MUST reuse shared diagnostics, direct CLI argv parsing/mapping, JSON IO and document output orchestration helpers where they exist. Core CLI MUST continue to own command classification, configuration command behavior, project root context for handoff, static registry ownership, registry command resolution, non-document command behavior and concrete core exit code enum.

Shared argv helper usage MUST serve strict parsing/mapping and diagnostics. It MAY be implemented with `clap` command definitions and shared error mapping. Unknown flags, extra positional values and operation-inapplicable known flags MUST fail before successful document execution.

#### Scenario: Core document argv 使用严格 parser 或 helper
- **WHEN** core document CLI parses unknown flags, extra positional values or known flags that are not applicable to the selected operation
- **THEN** it uses `clap` strict parsing, shared direct CLI argv mapping or equivalent strict diagnostics helper
- **THEN** core CLI returns an input diagnostic before navigation input resolution
- **THEN** core document CLI and protocol request handling both reject invalid direct input at their owner boundary

#### Scenario: Core diagnostic construction 只服务 primary DiagnosticRecord
- **WHEN** core CLI renders a strict input failure
- **THEN** it constructs or maps one primary `DiagnosticRecord`
- **THEN** the primary `DiagnosticRecord` identity is derived from the owning error code
- **THEN** protocol-json stdout 使用 protocol failure response fields

#### Scenario: Core document output 使用共享输出编排
- **WHEN** core CLI 得到 document operation success 或 diagnostic failure outcome
- **THEN** 它将 outcome、operation、request id、output mode 和允许投影的 diagnostics 传给 `docnav-output` 的 document-only facade
- **THEN** `readable-json` 和 `readable-view` 从同一个 readable payload 派生
- **THEN** `protocol-json` 向 stdout 写出一个 protocol response envelope
- **THEN** rejected caller input 使用 diagnostic failure outcome

#### Scenario: Core exit code enum 仍由 core 拥有
- **WHEN** core CLI 将 `DiagnosticCode` 对应的共享分类映射为 process exit code
- **THEN** 它可以使用共享 classification helper
- **THEN** concrete core exit code enum 和最终 process exit decision 仍由 `docnav` core 拥有

## REMOVED Requirements

### Requirement: 核心 CLI 必须兼容未知、多余和未使用参数
**Reason**: 新契约把 caller argv 作为 strict public input，由 CLI owner 在 document execution 前返回 input diagnostic。

**Migration**: Unknown argv、extra positional arguments 和 operation-inapplicable known flags 统一投影为带 actionable repair guidance 的 input diagnostics。Valid document argv 继续映射为 canonical document operation input 或等价 semantic request。
