本 change 的目标是用仓库内 renderer config 驱动的 `readable-view` 替代 document operation 的 `text` 输出模式。

## MODIFIED Requirements

### Requirement: 输出模式必须按协议层、阅读层和非文档纯文本分离
`docnav --output protocol-json` MUST 输出原始 protocol response envelope；CLI warning 在该模式下通过 stderr 诊断表达。默认 readable-view 和 readable-json 输出必须保持为阅读层结果，并必须从同一个完整 readable payload 派生。readable-view JSON header 和 readable-json 在存在 ignored argv 或 adapter candidate warning 时必须包含顶层 `warnings`。每个 warning item 必须使用稳定 warning envelope，包含 `id`、非空 `reason`、稳定 `effect` 和 `details` 对象。CLI argv warning 必须使用 `id: "cli_argv_ignored"`，相关 argv token 只能作为 `details.tokens` 等 family-specific detail 表达。CLI argv conformance 断言 stable warning envelope、`cli_argv_ignored` id 和诊断存在性。

核心 CLI document output enum/help/config MUST expose only `readable-view`、`readable-json` 和 `protocol-json`。Help、version 和其它非文档纯文本输出 MUST 通过与 document output mode 分离的 `PlainText` 或等价明确命名通道承载。

#### Scenario: readable-json outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md --output readable-json`
- **THEN** 输出包含 entries、page 等 outline readable fields
- **THEN** 输出不包含 protocol envelope 字段

#### Scenario: 默认 readable-view outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md` 且未传入 `--output`
- **THEN** 输出使用 readable-view pretty JSON header
- **THEN** JSON header 包含 entries、page 和可选 warnings
- **THEN** 输出不包含 protocol envelope 字段

#### Scenario: readable-json warning envelope
- **WHEN** 调用方执行带有未知参数但其它参数有效的 readable-json 命令
- **THEN** 输出包含该 operation 的 readable 字段
- **THEN** 输出包含 `warnings` 数组
- **THEN** 每个 warning item 包含稳定 `id`、非空 `reason`、稳定 `effect` 和 `details` 对象
- **THEN** CLI argv warning 使用 `id: "cli_argv_ignored"`
- **THEN** 测试断言 stable warning envelope、`cli_argv_ignored` id 和诊断存在性

#### Scenario: readable-view warning envelope
- **WHEN** 调用方执行带有未知参数但其它参数有效的默认或显式 readable-view 命令
- **THEN** JSON header 包含该 operation 的 readable 字段和 `warnings` 数组
- **THEN** warning 只由 JSON header 的 `warnings` 数组承载

#### Scenario: protocol-json warning
- **WHEN** 调用方执行带有未知参数但其它参数有效的 protocol-json 命令
- **THEN** 输出包含完整 protocol response envelope
- **THEN** 输出不包含 `warnings` 数组
- **THEN** stderr 包含 warning 诊断

#### Scenario: document output 值按三种模式校验
- **WHEN** 调用方执行 `docnav outline docs/guide.md --output <invalid-output>`
- **THEN** `docnav` 返回 `INVALID_REQUEST`
- **THEN** CLI help 只列出 readable-view、readable-json 和 protocol-json
- **THEN** `docnav` 在 adapter routing 和 document operation 执行前返回

#### Scenario: help 和 version 仍可输出纯文本
- **WHEN** 调用方执行 `docnav --help` 或 `docnav --version`
- **THEN** CLI 可以输出普通纯文本
- **THEN** 该输出使用非文档 `PlainText` 或等价明确命名通道
- **THEN** 该输出不需要使用 readable-view header 或 block framing

#### Scenario: defaults.output 无效值返回配置错误
- **WHEN** 有效项目或用户配置包含无效 `defaults.output` 值
- **AND** 调用方执行 `docnav outline docs/guide.md`
- **THEN** `docnav` 返回配置错误
- **THEN** 错误包含配置路径、`defaults.output`、收到的值和可接受值
- **THEN** document execution 在配置错误路径直接返回
