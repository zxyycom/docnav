# protocol-and-adapter-sdk-implementation Specification

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
`docnav-adapter-sdk` MUST 为 adapter 直接 CLI 提供兼容性参数解析：未知 flag、多余 positional 和当前 operation 不使用的已知 flag MUST 生成列明具体被忽略 argv token、kind 和 reason 的 warning 后忽略；已知必需参数缺失、当前 operation 实际使用的已知 flag 缺少值或值非法时 MUST 返回输入错误。Warnings MUST NOT 扩展 adapter `invoke`、CLI `protocol-json`、manifest 或 probe 的 stdout schema。

#### Scenario: 未知 flag 被 warning 后忽略
- **WHEN** adapter 直接 CLI 执行文档操作并收到未知 flag
- **THEN** SDK 生成 `kind` 为 unknown flag 且 `ignored_tokens` 包含该 flag 原始 token 的 warning
- **THEN** SDK 只忽略该未知 flag token
- **THEN** 若其它参数有效，命令继续执行并保持成功退出码

#### Scenario: 未知 flag 的后续 token 继续普通解析
- **WHEN** adapter 直接 CLI 收到 `--unknown value`
- **THEN** SDK 只把 `--unknown` 归属为 unknown flag warning 的 ignored token
- **THEN** SDK 将 `value` 继续按普通 token 解析
- **THEN** 若没有 positional 槽位接收 `value`，SDK 为 `value` 生成多余 positional warning

#### Scenario: 未知 flag 不吞已知 flag
- **WHEN** adapter 直接 CLI 收到 `--future --output protocol-json`
- **THEN** SDK 忽略 `--future` 并生成 warning
- **THEN** SDK 仍解析 `--output protocol-json`

#### Scenario: 多余 positional 被 warning 后忽略
- **WHEN** adapter 直接 CLI 执行文档操作并收到多余 positional
- **THEN** SDK 生成 `kind` 为 extra positional 且 `ignored_tokens` 包含该 positional 原始 token 的 warning
- **THEN** SDK 忽略该 positional

#### Scenario: warning 按阅读输出模式承载
- **WHEN** 命令以 text 输出模式成功并存在 warning
- **THEN** stdout 在正常阅读文本后拼接包含 ignored token 和 reason 的 warning 文本
- **WHEN** 命令以 readable-json 或其它阅读层 structured 输出模式成功并存在 warning
- **THEN** stdout payload 包含顶层 `warnings` 数组
- **THEN** stdout 仍是该输出模式下的合法 payload

#### Scenario: protocol-shaped stdout 不承载 CLI warning
- **WHEN** adapter 直接 CLI 以 protocol-json、manifest 或 probe 输出模式成功并存在 CLI warning
- **THEN** stdout 不包含 `warnings` 字段
- **THEN** stdout 仍通过该输出模式对应的 schema
- **THEN** stderr 包含该 warning 的 ignored token、kind 和 reason

#### Scenario: 已知 flag 的值紧跟解析
- **WHEN** adapter 直接 CLI 收到需要值的已知 flag
- **THEN** SDK 将紧跟该 flag 的下一个 token 作为值
- **THEN** 即使该 token 以 `--` 开头也作为值处理
- **THEN** 只有不存在下一个 token 时才返回该 flag 缺少值的输入错误

#### Scenario: 当前 operation 不使用的已知有值 flag 记录被消费 value
- **WHEN** adapter 直接 CLI 收到当前 operation 不使用但需要值的已知 flag
- **THEN** SDK 按该 flag 的形状消费紧跟的 value token
- **THEN** SDK 生成 `kind` 为 unused operation flag 且 `ignored_tokens` 同时包含 flag token 和 value token 的 warning
- **THEN** SDK 不校验该 value 在当前 operation 中不会生效的业务合法性

#### Scenario: invoke stdin 仍严格校验
- **WHEN** adapter `invoke` 从 stdin 收到包含未知字段或参数类型错误的 JSON request
- **THEN** SDK 返回结构化 protocol failure
- **THEN** SDK 不按直接 CLI 兼容策略忽略该字段
