## ADDED Requirements

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
