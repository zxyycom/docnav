范围说明：本 delta 只调整 adapter SDK direct CLI argv 解析、有效 invoke request 归一和 warning 承载边界。Adapter `invoke` JSON 保持严格，protocol-shaped stdout 继续受 schema 约束。

## MODIFIED Requirements

### Requirement: SDK 直接 CLI 必须兼容 CLI 扩展参数
`docnav-adapter-sdk` MUST 为 adapter direct CLI 提供 AI 友好的宽松 argv 解析。SDK 必须使用 `clap` 或 `clap` builder API 作为共享命令、子命令、固定参数、默认值、枚举值和 help 的 argv 结构解析基础。

Adapter SDK 入口必须保持以下分层：

- Direct CLI 文档操作通过 `clap` 承载已知命令、已知参数声明、默认值、枚举和 help；SDK 在确定 operation 后只对当前 operation 实际使用的参数做类型、范围和枚举校验，并受控收集 unknown、extra positional 和 unused known 参数的原始 token。
- Adapter `invoke` 通过严格 protocol/schema 校验解析 stdin JSON。
- 传输层解析成功后，direct CLI 文档操作和有效 invoke request 必须映射到 canonical document operation input 或等价 semantic request。
- 共享语义归一和统一 operation handler 必须负责默认值、native options、必需参数校验和 request 构造。
- 宽松 argv 收集层只生成 warning metadata，不复制业务参数解释、默认值归一或 request 构造逻辑。
- 当前 operation 的必需语义存在且实际使用参数有效时，未知 flag、多余 positional 和当前 operation 不使用的参数不得阻断 direct CLI 成功。
- 当前 operation 实际使用的参数必须保持严格。
- Malformed invoke JSON、未知字段、缺失字段或类型错误必须在进入 canonical document operation input 或等价 semantic request 前失败。
- 每个被忽略的 argv family 必须形成阅读层 warning 或 stderr 诊断；输出通道按当前输出模式决定。
- Readable warning item 必须使用稳定 warning envelope：稳定 `id`、非空 `reason`、稳定 `effect` 和 `details` 对象。CLI argv warning 必须使用 `id: "cli_argv_ignored"`，并可在 `details.tokens` 中列出相关 argv token。
- CLI warning 不得给 adapter `invoke`、CLI `protocol-json`、manifest 或 probe stdout schema 增加字段。

#### Scenario: 未知 argv 不阻断有效操作
- **WHEN** adapter direct CLI 执行文档操作并收到未知 flag 或多余 positional
- **AND** 当前 operation 的 path/ref/query 等必需语义参数可被解析
- **THEN** SDK 继续构造 canonical document operation input 或等价 semantic request，并生成对应 operation request
- **THEN** 命令结果由业务处理和输出模式决定
- **THEN** SDK 输出阅读层 warning 或 stderr 诊断
- **THEN** CLI argv warning 使用 `id: "cli_argv_ignored"`
- **THEN** 测试不要求 exact token 分组、`reason` 文案或 token 消费顺序

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
- **THEN** SDK 以原始 token 保留该参数，不要求该参数值通过当前 operation 不会使用的业务校验

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
- **WHEN** adapter direct CLI 以 text 输出模式成功并存在 warning
- **THEN** stdout 在正常阅读文本后拼接用户可理解 warning 文本
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

#### Scenario: invoke stdin 仍严格校验
- **WHEN** adapter `invoke` 从 stdin 收到包含未知字段或参数类型错误的 JSON request
- **THEN** SDK 返回结构化 protocol failure
- **THEN** 该请求不进入 canonical document operation input 或等价 semantic request
- **THEN** SDK 不按 direct CLI argv 容错策略忽略该字段
