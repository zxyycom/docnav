**一句话核心：本 delta 将 adapter SDK 直接 CLI 从精确手写兼容 parser 迁移为 `clap` 优先、成功路径优先的宽松 argv 解析要求。当前 change 只在 `openspec/changes/adopt-clap-cli-parsing/` 下形成未审核临时文档，不影响现有其它文档或主规范。**

## MODIFIED Requirements

### Requirement: SDK 直接 CLI 必须兼容 CLI 扩展参数
`docnav-adapter-sdk` MUST 为 adapter 直接 CLI 提供 AI 友好的宽松参数解析，并 MUST 优先使用 `clap` 定义共享命令、子命令、固定参数、默认值、枚举值和 help。未知 flag、多余 positional 和当前 operation 不使用的参数 MUST NOT 在其它必需语义参数正确时阻断成功执行；SDK MAY 生成用户可见 warning 或诊断提示被忽略输入，但 MUST NOT 将 ignored token 分组、warning kind 枚举或 token 消费顺序作为长期稳定契约。已知必需参数缺失、当前 operation 实际使用的已知参数缺少值或值非法时 MUST 返回输入错误。Warnings MUST NOT 扩展 adapter `invoke`、CLI `protocol-json`、manifest 或 probe 的 stdout schema。

#### Scenario: 未知 argv 不阻断有效操作
- **WHEN** adapter 直接 CLI 执行文档操作并收到未知 flag 或多余 positional
- **AND** 当前 operation 的 path/ref/query 等必需语义参数可被解析
- **THEN** SDK 继续构造对应 operation request
- **THEN** 命令执行结果由业务处理和输出模式决定
- **THEN** SDK 可以输出阅读层 warning 或诊断，但不要求 warning 包含稳定 ignored token shape

#### Scenario: 未知 argv 不阻断已知输出模式
- **WHEN** adapter 直接 CLI 收到未知 argv 和可解析的 `--output protocol-json`
- **AND** 当前 operation 的其它必需语义参数可被解析
- **THEN** SDK 仍按 `protocol-json` 输出模式执行
- **THEN** stdout 是该输出模式对应 schema-valid payload
- **THEN** 若存在 warning，warning 不写入 protocol-shaped stdout

#### Scenario: 多余 positional 容错
- **WHEN** adapter 直接 CLI 执行文档操作并收到多余 positional
- **AND** 当前 operation 已能解析所需 path 和其它必需参数
- **THEN** SDK 忽略该多余 positional 或将其记录为阅读层诊断
- **THEN** SDK 不因该多余 positional 单独失败

#### Scenario: 已知使用参数仍严格校验
- **WHEN** adapter 直接 CLI 收到当前 operation 实际使用的已知参数
- **AND** 该参数缺少必需值或值无法通过该参数的类型与范围校验
- **THEN** SDK 返回输入错误
- **THEN** SDK 不以宽松解析策略静默替换为默认值

#### Scenario: 必需语义缺失仍失败
- **WHEN** adapter 直接 CLI 执行 `outline` 但缺少 path
- **OR** 执行 `read` 但无法解析 ref
- **OR** 执行 `find` 但无法解析 query
- **THEN** SDK 返回输入错误
- **THEN** stderr 或阅读错误 payload 提供可帮助 AI 修正调用的简洁诊断

#### Scenario: Help 暴露可纠错入口
- **WHEN** 调用方执行 adapter 直接 CLI 的 `--help` 或子命令 help
- **THEN** 输出列出支持的命令、固定参数、默认值或可选值
- **THEN** help 文本可作为 AI 纠正参数调用的主要入口

#### Scenario: invoke stdin 仍严格校验
- **WHEN** adapter `invoke` 从 stdin 收到包含未知字段或参数类型错误的 JSON request
- **THEN** SDK 返回结构化 protocol failure
- **THEN** SDK 不按直接 CLI argv 容错策略忽略该字段
