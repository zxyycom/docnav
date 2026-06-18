# adapter-protocol Specification

## Purpose
定义 Docnav v0 原始协议共享类型、协议兼容判断、adapter SDK invoke 生命周期，以及 schema 和示例验证的实现要求。
## Requirements
### Requirement: 共享协议类型完整覆盖 v0 原始协议
`docnav-protocol` MUST 定义 v0 request envelope、response envelope、operation、operation arguments、operation result、page、stable error、manifest 和 probe 的共享类型，并 MUST 不包含格式专属解析字段。

#### Scenario: 构造 outline 成功响应
- **WHEN** 调用方使用共享协议类型构造 `outline` 成功响应
- **THEN** 响应包含 `protocol_version`、`request_id`、`operation: "outline"`、`ok: true` 和 outline result
- **THEN** result 只包含扁平 entries 和 page

#### Scenario: 拒绝格式专属字段进入共享协议
- **WHEN** 实现者需要表达 Markdown heading path
- **THEN** 该信息只能存在于 adapter 生成的 `ref` 或 `display`
- **THEN** `docnav-protocol` 不新增 Markdown 专属 result 字段

### Requirement: operation 必须绑定成功 result 类型
protocol response schema 和共享校验 MUST 使用响应 `operation` 绑定成功 result 类型，且成功响应 operation MUST 与请求 operation 一致。

#### Scenario: read 响应绑定 ReadResult
- **WHEN** 请求 operation 为 `read`
- **THEN** 成功响应 operation 为 `read`
- **THEN** result 必须符合 ReadResult

### Requirement: SDK 必须实现单请求 invoke 生命周期
`docnav-adapter-sdk` MUST 提供 adapter invoke 单请求生命周期：从 stdin 读取一个完整 request，分发到对应 operation handler，向 stdout 输出一个 protocol JSON 响应，并在完成后退出。

#### Scenario: invoke 输出单个响应
- **WHEN** adapter 通过 SDK 处理一次 invoke 请求
- **THEN** stdout 只输出一个 JSON 响应
- **THEN** 诊断信息只能写入 stderr

### Requirement: SDK 必须提供 manifest 和 probe 分发基础
`docnav-adapter-sdk` MUST 支持 adapter 实现 manifest 和 probe 命令，并 MUST 保持 manifest/probe 与 invoke protocol envelope 分离。

#### Scenario: manifest 输出专属 schema
- **WHEN** 调用方执行 adapter `manifest --output protocol-json`
- **THEN** 输出符合 manifest schema
- **THEN** 输出不包含 invoke request/response envelope

### Requirement: 自动化验证必须覆盖 schema 与示例
Docnav 协议与 adapter SDK 实现 MUST 提供自动化验证，覆盖 protocol request/response、manifest、probe 和 readable schema 的 strict 编译，以及关键示例 fixture 的解析和语义校验。

#### Scenario: 校验协议响应 fixture
- **WHEN** 验证脚本读取 protocol response 示例
- **THEN** 示例通过 protocol response schema
- **THEN** 响应 operation 与 result 类型匹配

### Requirement: 协议边界必须按当前契约硬校验
Docnav 协议与 adapter SDK MUST 使用当前 protocol、manifest 和 probe schema 以及语义校验判断输出是否符合当前契约。`protocol_version`、`manifest_version` 和 `probe_version` MUST 保留为固定 schema 识别字段，但 MUST NOT 参与 adapter 路由、安装、更新或 invoke 的版本区间协商。

#### Scenario: 当前契约校验通过
- **WHEN** adapter manifest、probe 和 invoke 响应符合当前 schema
- **AND** 必需字段、字段类型、operation/result shape 和语义校验全部通过
- **THEN** 协议层认为该 adapter 输出符合当前契约

#### Scenario: 当前契约校验失败
- **WHEN** adapter 输出缺少当前 schema 必需字段或字段类型不符
- **THEN** 校验失败原因包含字段或 schema 路径信息
- **THEN** 当前阶段失败
- **THEN** 未选中的 adapter 记录为候选失败证据，已选中的 adapter 返回稳定 adapter/protocol 错误

#### Scenario: 请求版本字段不匹配当前 schema
- **WHEN** invoke 请求中的 `protocol_version` 不是当前 schema 固定值
- **THEN** request schema 校验失败
- **THEN** SDK 返回 `INVALID_REQUEST`
- **THEN** SDK 不返回 `PROTOCOL_INCOMPATIBLE`

#### Scenario: 无法解析请求时使用当前协议识别字段
- **WHEN** SDK 无法解析 invoke stdin 为有效请求 envelope
- **THEN** failure response 的 `protocol_version` 使用当前 `PROTOCOL_VERSION` 常量
- **THEN** failure response 的 `request_id` 使用未知请求 id 占位
- **THEN** failure response 的 `operation` 为 `null`

### Requirement: Manifest 必须只承载 adapter 能力声明
Adapter manifest MUST restrict its field ownership to adapter identity, supported formats, extensions, content types, and capabilities. Manifest schema MUST reject protocol range fields and `recommended_parameters`.

#### Scenario: 读取 manifest
- **WHEN** adapter 输出 manifest
- **THEN** manifest 字段集合只表达 adapter 身份、支持格式、扩展名、content type 和 capabilities
- **THEN** 格式专属默认值不通过 manifest 传给 `docnav`

#### Scenario: Manifest 包含旧字段
- **WHEN** adapter manifest 包含 `protocol.min`、`protocol.max` 或 `recommended_parameters`
- **THEN** manifest schema 校验失败
- **THEN** adapter 在当前阶段不可用

### Requirement: Invoke options 必须保留为 adapter 拥有的显式参数
Protocol request argument types MUST keep optional `options` as an opaque adapter-owned object. `docnav-protocol` and `docnav-adapter-sdk` MUST NOT derive `options` from manifest `recommended_parameters`.

#### Scenario: Adapter 直接 CLI 生成 options
- **WHEN** adapter 直接 CLI 根据 adapter 自有 flag 生成 `arguments.options`
- **THEN** invoke 请求 schema 接受该 `options` 对象
- **THEN** SDK 将该 `options` 对象原样传给 adapter operation handler

#### Scenario: Manifest 不提供 options 来源
- **WHEN** adapter manifest 通过当前 schema 校验
- **THEN** manifest 不包含 `recommended_parameters`
- **THEN** SDK 不从 manifest 合成 `arguments.options`

### Requirement: SDK 直接 CLI 必须兼容 CLI 扩展参数
`docnav-adapter-sdk` MUST 为 adapter direct CLI 提供 AI 友好的宽松 argv 解析。SDK 必须使用 `clap` 或 `clap` builder API 作为共享命令、子命令、固定参数、默认值、枚举值和 help 的 argv 结构解析基础。

Adapter SDK 入口必须保持以下分层：

- Direct CLI 文档操作通过 `clap` 承载已知命令、已知参数声明、默认值、枚举和 help；SDK 在确定 operation 后只对当前 operation 实际使用的参数做类型、范围和枚举校验，并受控收集 unknown、extra positional 和 unused known 参数的原始 token。
- Adapter `invoke` 通过严格 protocol/schema 校验解析 stdin JSON。
- 传输层解析成功后，direct CLI 文档操作和有效 invoke request 必须映射到 canonical document operation input 或等价 semantic request。
- 共享语义归一和统一 operation handler 必须负责默认值、native options、必需参数校验和 request 构造。
- 宽松 argv 收集层只生成 warning metadata；业务参数解释、默认值归一和 request 构造逻辑由共享语义归一与 operation handler 承担。
- 当前 operation 的必需语义存在且实际使用参数有效时，未知 flag、多余 positional 和当前 operation 不使用的参数进入 warning metadata，direct CLI 继续成功路径。
- 当前 operation 实际使用的参数必须保持严格。
- Malformed invoke JSON、未知字段、缺失字段或类型错误必须在进入 canonical document operation input 或等价 semantic request 前失败。
- 每个被忽略的 argv family 必须形成阅读层 warning 或 stderr 诊断；输出通道按当前输出模式决定。
- Readable warning item 必须使用稳定 warning envelope：稳定 `id`、非空 `reason`、稳定 `effect` 和 `details` 对象。CLI argv warning 必须使用 `id: "cli_argv_ignored"`，并可在 `details.tokens` 中列出相关 argv token。
- Adapter `invoke`、CLI `protocol-json`、manifest 和 probe stdout 保持各自 schema；CLI warning 在这些通道中通过 stderr 或诊断表达。
- Direct CLI document operation 的阅读输出必须通过共享 readable payload 和 readable-view renderer 生成；SDK document output surface 只暴露 readable-view、readable-json 和 protocol-json。
- Manifest、probe、help 和其它非 document operation 通道保持各自既有结构化或纯文本边界。

#### Scenario: 未知 argv 不阻断有效操作
- **WHEN** adapter direct CLI 执行文档操作并收到未知 flag 或多余 positional
- **AND** 当前 operation 的 path/ref/query 等必需语义参数可被解析
- **THEN** SDK 继续构造 canonical document operation input 或等价 semantic request，并生成对应 operation request
- **THEN** 命令结果由业务处理和输出模式决定
- **THEN** SDK 输出阅读层 warning 或 stderr 诊断
- **THEN** CLI argv warning 使用 `id: "cli_argv_ignored"`
- **THEN** 测试断言 stable warning envelope、`cli_argv_ignored` id 和诊断存在性

#### Scenario: direct CLI 和 invoke 共享文档操作语义归一
- **WHEN** adapter direct CLI input 与 adapter `invoke` schema-valid JSON 表达同一个 outline/read/find/info 操作
- **THEN** 两者在传输解析后进入 canonical document operation input 或等价 semantic request
- **THEN** 默认值、native options、必需参数校验和 operation handler 不因入口不同分叉
- **THEN** 测试可通过等价 request、等价结果或共享 helper 覆盖该约束

#### Scenario: 未知 argv 不阻断已知输出模式
- **WHEN** adapter direct CLI 收到未知 argv 和可解析的 `--output protocol-json`
- **AND** 当前 operation 的其它必需语义参数可被解析
- **THEN** SDK 仍按 `protocol-json` 输出模式执行
- **THEN** stdout 是该输出模式对应的 schema-valid payload
- **THEN** warning 不写入 protocol-shaped stdout

#### Scenario: 多余 positional 容错
- **WHEN** adapter direct CLI 执行文档操作并收到多余 positional
- **AND** 当前 operation 已能解析所需 path 和其它必需参数
- **THEN** SDK 忽略该多余 positional，或将其记录为阅读层 warning / stderr 诊断
- **THEN** SDK 不因该多余 positional 单独失败

#### Scenario: 当前 operation 不使用的参数宽松
- **WHEN** adapter direct CLI 收到当前 operation 不使用的已知参数
- **AND** 该参数值无法通过其它 operation 的类型、枚举或范围校验
- **AND** 当前 operation 的必需语义参数仍可被解析
- **THEN** SDK 不因该参数单独失败
- **THEN** SDK 将该参数记录为阅读层 warning 或 stderr 诊断
- **THEN** SDK 以原始 token 保留该参数，并只校验当前 operation 实际使用的业务参数

#### Scenario: 当前 operation 使用的已知参数仍严格
- **WHEN** adapter direct CLI 收到当前 operation 实际使用的已知参数
- **AND** 该参数缺少必需值或值无法通过类型、枚举或范围校验
- **THEN** SDK 返回输入错误
- **THEN** SDK 不通过宽松解析策略静默替换为默认值

#### Scenario: 必需语义缺失仍失败
- **WHEN** adapter direct CLI 执行 `outline` 但缺少 path
- **OR** 执行 `read` 但无法解析 ref
- **OR** 执行 `find` 但无法解析 query
- **THEN** SDK 返回输入错误
- **THEN** stderr 或阅读错误 payload 提供可帮助 AI 修正调用的简洁诊断

#### Scenario: Help 暴露可纠错入口
- **WHEN** 调用方执行 adapter direct CLI 的 `--help` 或子命令 help
- **THEN** 输出列出支持的命令、固定参数、默认值或可选值
- **THEN** help 文本可作为 AI 纠正参数调用的主要入口
- **THEN** help 不执行文档导航业务

#### Scenario: warning 按阅读输出模式承载
- **WHEN** adapter direct CLI 以 readable-view 输出模式成功并存在 warning
- **THEN** stdout JSON header 包含正常 readable 字段和顶层 `warnings`
- **THEN** stdout 的 warning 只由 JSON header 的 `warnings` 数组承载
- **WHEN** adapter direct CLI 以 readable-json 输出模式成功并存在 warning
- **THEN** stdout payload 必须包含顶层 `warnings` 数组
- **THEN** warning item 包含稳定 `id`、非空 `reason`、稳定 `effect` 和 `details` 对象
- **THEN** CLI argv warning 使用 `id: "cli_argv_ignored"`
- **THEN** stdout 仍是该输出模式下的合法 payload

#### Scenario: protocol-shaped stdout 不承载 CLI warning
- **WHEN** adapter direct CLI 以 protocol-json、manifest 或 probe 输出模式成功并存在 CLI warning
- **THEN** stdout 不包含 `warnings` 字段
- **THEN** stdout 仍通过该输出模式对应的 schema
- **THEN** stderr 包含 warning 或诊断

#### Scenario: document direct output 值按三种模式校验
- **WHEN** 调用方对 adapter document operation 传入无效 `--output` 值
- **THEN** SDK 返回输入错误
- **THEN** help 只列出 readable-view、readable-json 和 protocol-json
- **THEN** adapter 在 document operation handler 执行前返回

#### Scenario: 非文档 direct CLI 通道不受 document output mode 约束
- **WHEN** 调用方执行 adapter direct CLI 的 manifest、probe 或 help
- **THEN** SDK 按对应通道既有 schema 或纯文本 help 输出
- **THEN** 该输出不需要使用 readable-view
- **THEN** document operation help 仍只列出 readable-view、readable-json 和 protocol-json

#### Scenario: invoke stdin 仍严格校验
- **WHEN** adapter `invoke` 从 stdin 收到包含未知字段或参数类型错误的 JSON request
- **THEN** SDK 返回结构化 protocol failure
- **THEN** 该请求不进入 canonical document operation input 或等价 semantic request
- **THEN** SDK 不按 direct CLI argv 容错策略忽略该字段

### Requirement: Protocol 和 adapter SDK helper 必须保持进程边界契约

`docnav-protocol`、`docnav-json-io` 和 `docnav-adapter-sdk` MUST 只在不破坏当前 protocol、direct CLI 和 adapter process boundary 的位置暴露共享 helper。Adapter `invoke` stdin JSON MUST 保持严格 protocol input；adapter direct CLI document command MAY 复用 direct CLI loose argv 和 document output helper。

#### Scenario: Protocol decode helper 保持严格 schema 和 semantic validation

- **WHEN** 共享代码 decode protocol request、protocol response、manifest 或 probe JSON value
- **THEN** decode pipeline 在把 JSON value 当作 typed contract data 前，先按 owning schema 校验
- **THEN** typed deserialization 和 semantic validation 在 schema validation 之后执行
- **THEN** 调用方 surface 保持既有 stable error category、field path、diagnostic text 和 exit behavior

#### Scenario: Adapter invoke 保持严格 protocol decoding

- **WHEN** adapter `invoke` 收到包含 unknown fields、missing required fields 或 wrong argument types 的 stdin JSON
- **THEN** SDK strict protocol decoding 按 invoke contract 拒绝该请求
- **THEN** 不应用 `docnav-cli-args` loose argv rule
- **THEN** failure 仍是 protocol-shaped failure response

#### Scenario: Adapter direct CLI document command 复用共享 helper

- **WHEN** adapter direct CLI document operation 成功或返回 stable error
- **THEN** SDK 可以使用共享 diagnostics 表达 warning envelope 和 stderr warning text
- **THEN** SDK 可以使用 `docnav-output` 执行 document output mode dispatch
- **THEN** manifest、probe 和 help output 保持既有 adapter contract 或 plain text boundary
- **THEN** manifest 和 probe 的 machine-readable JSON 可以复用 `docnav-json-io`
- **THEN** manifest、probe 和 help output 不通过 `docnav-output` 编排

#### Scenario: Adapter SDK paging helper 保持 format-neutral

- **WHEN** adapter 使用 SDK paging helper
- **THEN** helper 处理 character budget、text 或 entry pagination、next page calculation 和 truncation mechanics
- **THEN** helper 不生成 refs、不解析 refs、不检查 Markdown heading hierarchy，也不定义 adapter-specific display semantics

#### Scenario: Request id helper 只拥有格式不拥有 surface policy

- **WHEN** core、SDK 或 output code 需要 generated request id fallback
- **THEN** 它可以使用 `docnav-protocol` 提供的共享 request id helper
- **THEN** 调用方仍决定何时保留 incoming request id、何时使用 unknown placeholder，以及 request id 暴露在哪个 surface

