# core-cli Specification

## Purpose
定义 `docnav` 核心 CLI 的命令解析、非 navigation 命令、config source descriptor/path handoff、static registry ownership、输出分层和稳定错误映射行为。
## Requirements
### Requirement: 核心 CLI 必须作为独立 docnav 可执行入口交付
`docnav` MUST 作为独立 Rust workspace crate 和可执行二进制交付。格式解析 MUST 由选中的 adapter 完成，核心 CLI MUST 只负责命令解析、非 navigation 命令、config source descriptor/path handoff、registry ownership、协议校验和输出映射。

#### Scenario: 构建核心 CLI
- **WHEN** 构建 workspace 中的 `docnav` package
- **THEN** 产出名为 `docnav` 的可执行文件
- **THEN** 该可执行文件可以解析核心 CLI 命令

### Requirement: 核心 CLI 必须实现文档操作命令
`docnav` MUST 实现 `outline`、`read`、`find` 和 `info`，并 MUST 支持各命令对应的 `--adapter`、`--page`、`--limit` 和 `--output` 参数约束。`info` operation input MUST 只包含 info operation 需要的 core 参数。

#### Scenario: 执行 outline 命令
- **WHEN** 调用方执行 `docnav outline docs/guide.md`
- **THEN** `docnav-navigation` 解析最终 page 和 limit
- **THEN** `docnav-navigation` 解析 document path 为 absolute path
- **THEN** `docnav-navigation` dispatches the selected linked adapter operation handler

#### Scenario: 执行 info 命令
- **WHEN** 调用方执行 `docnav info docs/guide.md`
- **THEN** `docnav-navigation` dispatches the selected linked adapter info handler
- **THEN** info operation input 不包含 page 或 limit

### Requirement: 核心 CLI 必须严格拒绝未知、多余和未使用参数
`docnav` core CLI MUST 使用 `clap` 或 `clap` builder API 作为命令、子命令、固定参数、默认值、枚举值和 help 的 argv 结构解析基础。Document operation argv 必须先映射为 raw navigation command 或等价 handoff input，再连同 config source descriptors/paths 和 registry 进入 `docnav-navigation`；adapter selection、navigation request preparation 和 selected adapter dispatch 发生在 navigation input resolution 边界内。

Core CLI strict input 规则如下：

- 未知 flag、多余 positional 和当前 operation 不使用的已知参数 MUST 在执行前返回 `INVALID_REQUEST`。
- 当前 operation 实际使用的参数必须保持严格；缺值或值非法时必须返回 `INVALID_REQUEST`。
- `clap` 负责已知命令、已知参数声明、默认值、枚举和 help；Docnav 在确定 command/operation 后只允许当前 operation owning boundary 使用的参数进入 raw navigation command，不复制业务参数解释、默认值归一或 request 构造逻辑。
- 每个 rejected argv family 必须形成 primary input diagnostic；输出通道按当前输出模式投影 failure。
- `protocol-json` stdout 只输出 protocol response envelope；document success payload 不承载 caller input diagnostic。

#### Scenario: 核心 CLI 进入共享 semantic request 管道
- **WHEN** 调用方执行有效的 `docnav outline/read/find/info` CLI 命令
- **THEN** `clap` 或 `clap` builder 解析出类型化命令
- **THEN** document CLI input 映射为 raw navigation command 或等价 handoff input
- **THEN** `docnav-navigation` 负责 adapter selection、navigation request preparation 和 selected adapter dispatch
- **THEN** output mode 分流使用共享输出逻辑
- **THEN** CLI 不创建独立业务参数解释路径

#### Scenario: 未知 argv 阻断文档操作
- **WHEN** 调用方执行 `docnav outline docs/guide.md --future extra --output readable-json`
- **AND** path 和 output 可被解析
- **THEN** `docnav` 返回 `INVALID_REQUEST`
- **THEN** 不选择 adapter、不执行 outline
- **THEN** failure projection 包含 primary input diagnostic

#### Scenario: 未知 argv 仍使用已知输出模式投影错误
- **WHEN** 调用方执行 `docnav outline docs/guide.md --future --output protocol-json`
- **THEN** `docnav` 使用 `protocol-json` 输出模式投影 failure envelope
- **THEN** stdout 是通过 protocol response schema 的 failure envelope
- **THEN** 不 dispatch adapter operation

#### Scenario: 当前 operation 使用的已知参数仍严格
- **WHEN** 调用方执行 `docnav outline docs/guide.md --page 0`
- **OR** 执行 `docnav outline docs/guide.md --limit nope`
- **OR** 执行 `docnav outline docs/guide.md --output nope`
- **THEN** `docnav` 返回 `INVALID_REQUEST`
- **THEN** CLI 不通过 strict argv 策略忽略该参数并继续

#### Scenario: 当前 operation 不使用的参数严格失败
- **WHEN** 调用方执行 `docnav info docs/guide.md --page nope --output readable-json`
- **OR** 执行 `docnav info docs/guide.md --limit nope --output readable-json`
- **THEN** `docnav` 返回 `INVALID_REQUEST`
- **THEN** info operation request 不构造
- **THEN** failure projection 指出参数不适用于当前 operation

#### Scenario: Help 不执行业务
- **WHEN** 调用方执行 `docnav --help`
- **OR** 执行 core 子命令 help
- **THEN** 输出列出可用命令、关键参数、默认值或可选值
- **THEN** 该命令不读取文档、不选择 adapter、不 dispatch adapter operation

#### Scenario: Core non-document commands 保持非文档边界
- **WHEN** 调用方执行 `docnav config get/set/unset/list`
- **OR** 执行 `docnav init`
- **OR** 执行 `docnav doctor`
- **OR** 执行 `docnav version`
- **THEN** `docnav` 通过 `clap` 或 `clap` builder 解析出类型化非文档命令
- **THEN** 这些命令不进入 raw navigation command 或 navigation input resolution
- **THEN** 这些命令不执行 adapter selection、adapter dispatch 或文档导航业务
- **THEN** 代表性成功、失败、stdout、stderr 和 exit code 行为由 core CLI smoke 或等价测试覆盖

### Requirement: path 必须规范化并支持项目根外文件
`docnav` MUST preserve caller path input and cwd/project root context for navigation handoff. `docnav-navigation` MUST resolve document path to the absolute path passed to the selected adapter handler, and document operations MUST allow accessible files outside the project root.

#### Scenario: 项目根外路径
- **WHEN** 调用方传入会解析到项目根外且可访问的 path
- **THEN** `docnav-navigation` 保留 absolute document path
- **THEN** `docnav-navigation` 仍可继续选择 linked adapter 并 dispatch operation handler

#### Scenario: 不可访问路径
- **WHEN** 调用方传入不存在或不可读的 path
- **THEN** `docnav` 返回文档路径错误
- **THEN** 不 dispatch adapter operation

### Requirement: 核心配置 MVP 必须有限且可审计
`docnav` MUST provide an auditable core config command surface for `defaults.adapter`、`defaults.pagination.enabled`、`defaults.pagination.limit` 和 `defaults.output`。For navigation commands, core MUST supply project/user config source descriptors/paths to `docnav-navigation`; `docnav-navigation` MUST load raw config sources and resolve effective navigation values with priority `explicit > project > user > built_in`. Source-level static native option registry only participates in native option source classification、merge metadata and handoff; selected adapter typed-field declarations own adapter-owned option semantics.

#### Scenario: 配置 adapter 声明式选择
- **WHEN** 调用方未传入 `--adapter`
- **AND** 项目配置设置了 `defaults.adapter`
- **THEN** `docnav-navigation` 使用该 adapter id 作为 declared adapter
- **THEN** `docnav-navigation` 只在当前 static registry 中查找该 adapter 并执行 probe
- **THEN** `docnav-navigation` 不进入 automatic discovery 或 fallback

#### Scenario: page 不可配置
- **WHEN** 调用方省略 `--page`
- **THEN** `docnav-navigation` 使用 page `1`
- **THEN** 项目配置和用户配置保持初始 page 为 `1`

#### Scenario: 未知配置 key
- **WHEN** 调用方执行 `docnav config get unknown.key`
- **THEN** `docnav` 返回 `INVALID_REQUEST`

#### Scenario: 默认写项目配置
- **WHEN** 调用方执行 `docnav config set defaults.output readable-json`
- **THEN** `docnav` 写入 `<project-root>/.docnav/docnav.json`
- **THEN** 当前项目的生效配置包含 `defaults.output`

#### Scenario: 写用户配置
- **WHEN** 调用方执行 `docnav config set defaults.output readable-json --user`
- **THEN** `docnav` 写入用户配置文件
- **THEN** 未设置项目同名 key 时该用户配置成为生效值

#### Scenario: 列出当前生效配置
- **WHEN** 调用方执行 `docnav config list`
- **THEN** 输出包含所有支持 key 的当前生效值
- **THEN** 输出标明每个值来自项目配置、用户配置、内置默认值或未设置状态

#### Scenario: 按文档上下文列出最终配置
- **WHEN** 调用方执行 `docnav config list --path docs/guide.md --operation outline`
- **THEN** core 使用 navigation input resolution 或等价只读 helper 解析该文档上下文
- **THEN** 输出包含选中 adapter id
- **THEN** 输出包含该文档和 operation 下的最终默认参数及其来源

### Requirement: 核心管理命令必须提供 MVP 行为
`docnav` MUST 实现 `init`、`doctor`、`version` 和 core release 内置 adapter inspection 的可验证基础行为。核心管理命令 MUST keep adapter inspection tied to adapter implementations registered in the current core release static adapter registry. `docnav` MUST NOT expose `adapter install`、`adapter register`、`adapter update` or `adapter remove` as valid default CLI commands.

#### Scenario: init 幂等创建项目配置
- **WHEN** 调用方执行 `docnav init`
- **THEN** `docnav` 创建 `<project-root>/.docnav/docnav.json`
- **AND** 重复执行不会破坏已有配置

#### Scenario: version 输出 crate 版本
- **WHEN** 调用方执行 `docnav version`
- **THEN** stdout 包含 `docnav` crate version

#### Scenario: doctor 检查配置和 core release 内置 adapter libraries
- **WHEN** 调用方执行 `docnav doctor`
- **THEN** `docnav` 检查项目配置和用户配置
- **THEN** `docnav` 检查当前 core release static adapter registry metadata 和 adapter layer 可用性
- **THEN** 输出包含 checks 数组
- **AND** 存在失败检查项时进程非零退出

#### Scenario: dynamic adapter management commands are removed
- **WHEN** 调用方执行 `docnav adapter install ./target/release/custom-adapter`
- **OR** 调用方执行 `docnav adapter register ./target/release/custom-adapter`
- **OR** 调用方执行 `docnav adapter update custom-adapter`
- **OR** 调用方执行 `docnav adapter remove custom-adapter`
- **THEN** `docnav` 按标准 CLI unknown/unsupported command 行为返回失败
- **THEN** 该命令不会写入 adapter registry、project config 或 user config

### Requirement: adapter 选择必须区分声明式 adapter 和自动发现
`docnav-navigation` MUST first honor a declared adapter id from `--adapter` or `defaults.adapter`. Declared adapter failure MUST return an adapter selection diagnostic with the declared source and candidate failure stage. When no declared adapter id exists, `docnav-navigation` MUST enter automatic discovery by traversing the current static registry order and probing each linked adapter until the first `supported: true` result. Extension, content type, manifest metadata and descriptor metadata remain inspection or adapter-owned recognition facts. Adapter selection MUST use registry membership for implementation lookup, registry order for automatic discovery order and adapter probe results for format support.

#### Scenario: 显式 adapter 记录解析失败后返回诊断
- **WHEN** 调用方传入 `--adapter docnav-markdown` 但 registry 中无法解析该 adapter 记录
- **THEN** `docnav` 返回 adapter selection diagnostic
- **THEN** 错误 details 包含 adapter id、selection_source、stage 和 reason
- **THEN** `docnav` 不把显式 adapter failure 转为 automatic discovery success path

#### Scenario: linked probe 有效不支持后继续
- **WHEN** 候选 adapter probe 返回符合当前语义的 unsupported result
- **THEN** `docnav` 保留 unsupported 候选证据
- **THEN** `docnav` 继续 registry 遍历

#### Scenario: 未声明 adapter 时按 registry 顺序 probe
- **WHEN** 调用方没有传入 `--adapter`
- **AND** 配置没有指定 `defaults.adapter`
- **THEN** `docnav` 按当前 static registry 顺序遍历 linked adapter
- **THEN** `docnav` 对每个 adapter 执行 probe
- **THEN** `docnav` 选择第一个返回 `supported: true` 的 adapter
- **THEN** descriptor metadata、manifest metadata、扩展名和 content type 不改变 traversal order
- **THEN** probe result 决定每个候选是否支持该文档

#### Scenario: descriptor metadata 不参与 runtime candidate 选择
- **WHEN** registry entry 的 descriptor 声明 extension、content type 或 format metadata
- **AND** 调用方没有 declared adapter
- **THEN** `docnav` 仍按 static registry 顺序执行 probe
- **THEN** descriptor metadata 保持 inspection metadata
- **THEN** registry order 和 probe result 决定 runtime selection outcome

#### Scenario: registry 遍历候选当前契约不一致后继续
- **WHEN** registry 遍历中的候选 adapter probe 输出字段缺失、类型不符或语义校验失败
- **THEN** `docnav` 保留候选证据
- **THEN** `docnav` 继续 registry 遍历
- **THEN** 若后续候选成功，前序候选失败只保留为 internal discovery state，不进入 success output

#### Scenario: 所有阶段失败
- **WHEN** 没有 adapter 能校验目标文档
- **THEN** `docnav` 返回 `FORMAT_UNKNOWN`
- **THEN** 错误 details 包含候选摘要
- **THEN** 候选摘要是 JSON 数组
- **THEN** 每条候选摘要只包含 adapter_id、stage 和 reason

### Requirement: Core release static registry 必须提供 adapter implementation source
`docnav` core MUST own the current core release static adapter registry as the adapter implementation source for document operations and MUST pass that registry to `docnav-navigation`. Registry entries MUST resolve to linked adapter library handles and source-level descriptor metadata, including adapter id、manifest metadata、native option registry entries and operation handlers.

#### Scenario: 使用 static registry entry
- **WHEN** registry 中存在 Markdown adapter id 和 linked library handle
- **THEN** `docnav` 可以将该 registry entry 作为 handoff registry 的一部分提供
- **THEN** `docnav-navigation` 使用该 entry 的 linked probe 和 operation handler 执行格式识别与 document operation

#### Scenario: registry 保留遍历顺序
- **WHEN** registry 的 `adapters` 数组包含多个候选
- **THEN** `docnav` 按数组顺序遍历候选

#### Scenario: registry entry 缺少 linked implementation
- **WHEN** registry entry 缺少 linked adapter handle 或 descriptor metadata invalid
- **THEN** `docnav` 返回 adapter selection diagnostic
- **THEN** 不从 external executable、command path 或 historical adapter record 补足该 implementation

### Requirement: navigation request 必须包含最终 typed operation arguments
`docnav-navigation` MUST load raw project/user config sources before selected adapter dispatch, resolve configuration and defaults, and prepare final page、limit、ref、query and merged native options as operation arguments. Core `docnav` MUST pass raw navigation command、config source descriptors/paths and registry to `docnav-navigation`; it MUST NOT synthesize format-specific `options`. Source-level native option registry MAY declare public option sources and owner/namespace/type variants, but descriptor metadata MUST NOT be treated as request `options` values. Selected adapter typed-field declarations own unsupported/type/range diagnostics before handler dispatch.

#### Scenario: 省略 page
- **WHEN** 调用方省略 page
- **THEN** `docnav-navigation` resolves page as `1` before request construction

#### Scenario: 不写入格式 options
- **WHEN** selected registry entry metadata passes current validation
- **THEN** `docnav-navigation` 仍能解析 navigation defaults
- **THEN** navigation request 不包含由 manifest metadata 或隐式格式默认值合成的格式 options

#### Scenario: Missing adapter selection precedes option validation
- **WHEN** caller declares an adapter id absent from the current static registry
- **AND** the same request contains an invalid-looking native option value
- **THEN** `docnav` returns adapter selection diagnostic
- **THEN** `docnav` does not project that request as adapter option validation failure

#### Scenario: selected adapter operation result 当前契约不一致
- **WHEN** 选中 adapter 的 operation result 字段缺失、类型不符、operation/result shape 不匹配或语义校验失败
- **THEN** `docnav` 返回 adapter/protocol 错误
- **THEN** `docnav` 不继续尝试其它 adapter

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

### Requirement: 核心 CLI 必须保留 adapter 业务语义
`docnav` MUST 在输出映射中保留 adapter 返回的 ref、display、content、content_type、cost 和 page。

#### Scenario: read 保留 content_type
- **WHEN** adapter read 返回 `content_type: "text/markdown"`
- **THEN** `docnav` readable-json read 输出包含相同 content_type

### Requirement: 核心 CLI 必须实现 diagnostic code 和退出码映射
`docnav` MUST 将输入错误、文档/ref/格式错误、adapter layer structured errors、协议结果校验错误和内部错误映射到主规范定义的退出码，并 MUST 保持 `DiagnosticCode` 对应的可见错误 code 稳定。Concrete process exit code enum and final exit decision belong to core CLI/output owner.

#### Scenario: ref 不存在
- **WHEN** adapter 返回 `REF_NOT_FOUND`
- **THEN** `docnav` 保留该错误 code
- **THEN** CLI 退出码为文档/ref/格式错误对应值

#### Scenario: config key 不存在
- **WHEN** 调用方执行 `docnav config get <key>` 且 key 不存在
- **THEN** `docnav` 返回 `INVALID_REQUEST`

#### Scenario: adapter operation result 无法校验
- **WHEN** 选中 adapter 的 operation result 不能映射为当前 protocol result shape
- **THEN** `docnav` 返回协议或 adapter contract 错误
- **THEN** CLI 退出码由 core CLI/output owner 映射为对应失败类别

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

### Requirement: Navigation input resolution resolves pagination defaults before selected adapter dispatch
`docnav-navigation` MUST resolve `defaults.pagination.enabled`, `defaults.pagination.limit`, `--pagination enabled|disabled`, and `--limit <n>` into an explicit positive integer `limit` and `page` before dispatching a linked adapter handler. Core MUST treat `limit` as an adapter-owned numeric budget and MUST NOT interpret its unit.

#### Scenario: Navigation resolves pagination sources
- **WHEN** a caller runs a document operation
- **THEN** navigation input resolution maps pagination argv, project config, user config, and built-in defaults to the same declared parameter identities
- **THEN** direct input overrides project config, project config overrides user config, and user config overrides built-in defaults

#### Scenario: Navigation passes resolved limit to adapter
- **WHEN** navigation input resolution has resolved effective pagination enabled state, limit, and page
- **THEN** the selected adapter receives explicit operation arguments
- **THEN** the outgoing request contains `limit` and `page` rather than a protocol `pagination` field

#### Scenario: Navigation disables pagination through limit finalization
- **WHEN** effective pagination is disabled
- **THEN** navigation input resolution finalizes the outgoing limit as the configured maximum positive protocol budget
- **THEN** request construction does not add a separate pagination field to the adapter request

#### Scenario: Core keeps page outside configuration defaults
- **WHEN** a caller omits `page`
- **THEN** navigation input resolution resolves `page` to `1`
- **THEN** project and user config do not provide `defaults.page` or `defaults.pagination.page`

### Requirement: Core release 内置 adapter-layer workspace crates 必须成为默认 document operation implementation 来源
`docnav` MUST use adapter-layer workspace crates shipped with the current core release as default document operation adapter implementations. The default release MUST include all built-in adapters without using feature gates to select the default adapter set. CLI input and effective project/user config MAY declare an adapter id only through `--adapter` or `defaults.adapter`, and that id MUST resolve to an implementation registered in the current core release static adapter registry. Registry entries MUST expose source-level static descriptors containing adapter identity、native option registry entries and operation handler bindings. Project/user config、installed packages、external executables、command paths and historical adapter artifact records MUST NOT provide default document operation implementation. The adapter layer MUST remain a code and contract boundary, not a separate default distribution boundary.

#### Scenario: 默认发行物包含 adapter implementation
- **WHEN** 构建默认 `docnav` 发行物
- **THEN** 所有内置 adapter-layer workspace crates 随 `docnav` core release artifact 交付
- **THEN** 默认发行物不需要启用额外 feature 才能获得内置 adapter set
- **THEN** 默认发行物可直接执行已支持格式的 document operation

#### Scenario: Static descriptor supplies operation bindings
- **WHEN** core registry resolves the built-in Markdown adapter
- **THEN** the registry entry exposes a static descriptor with Markdown identity, native option registry entries and handler bindings
- **THEN** navigation input resolution can merge, validate/extract and hand off final native option values for linked dispatch
- **THEN** selected Markdown typed-field declarations validate consumed option support, type and range semantics before handler dispatch

#### Scenario: Core passes absolute path to linked adapter
- **WHEN** caller executes `docnav outline docs/guide.md` from a project subdirectory
- **THEN** `docnav` resolves the document path against cwd/project context to an absolute path
- **THEN** `docnav-navigation` and the linked adapter handler receive the absolute path
- **THEN** adapter IO does not depend on process cwd

#### Scenario: Historical adapter config does not provide implementation
- **WHEN** `<project-root>/.docnav/adapters.json` 存在并包含 adapter command path
- **AND** 调用方执行 `docnav outline docs/guide.md`
- **THEN** `docnav` 不把 adapter command path 当作 implementation source
- **THEN** `docnav` 只从当前 core release static adapter registry 选择 adapter implementation

#### Scenario: 声明式 adapter 只解析内置 adapter id
- **WHEN** 调用方通过 `--adapter custom-local-adapter` 声明 adapter id
- **AND** 该 id 不存在于当前 core release static adapter registry
- **THEN** `docnav` 返回 adapter selection diagnostic
- **THEN** `docnav` 不尝试从 installed package、external executable、command path 或 historical artifact record 加载该 adapter

#### Scenario: 列出 core release 内置 adapter libraries
- **WHEN** 调用方执行 `docnav adapter list`
- **THEN** 输出只包含当前 core release static adapter registry 中的 adapter id、version 和 supported formats

#### Scenario: Adapter diagnostics do not define process exit code
- **WHEN** a linked adapter handler returns a structured diagnostic
- **THEN** core/output maps that diagnostic to the selected protocol/readable surface
- **THEN** final process exit code remains owned by `docnav` core CLI
- **THEN** the adapter layer does not expose an exit-code API

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

