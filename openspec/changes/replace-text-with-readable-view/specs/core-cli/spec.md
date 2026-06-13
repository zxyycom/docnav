本 change 的目标是用仓库内版本化 renderer config 驱动的 `readable-view` 替代 document operation 的 `text` 输出模式；本文是未审核临时 delta spec，只存在于 `openspec/changes/replace-text-with-readable-view/`，不改变现有主规范或其它文档。

## MODIFIED Requirements

### Requirement: 输出模式必须按协议层、阅读层和非文档纯文本分离
`docnav --output protocol-json` MUST 输出原始 protocol response envelope，且不得增加 CLI warning 字段。默认 readable-view 和 readable-json 输出必须保持为阅读层结果，并必须从同一个完整 readable payload 派生。readable-view JSON header 和 readable-json 在存在 ignored argv 或 adapter candidate warning 时必须包含顶层 `warnings`。每个 warning item 必须使用稳定 warning envelope，包含 `id`、非空 `reason`、稳定 `effect` 和 `details` 对象。CLI argv warning 必须使用 `id: "cli_argv_ignored"`，相关 argv token 只能作为 `details.tokens` 等 family-specific detail 表达。CLI argv exact token 分组、`reason` 文案和消费顺序不稳定。

核心 CLI MUST NOT 支持 document `text` 输出模式或 text fallback。Help、version 和其它非文档纯文本输出 MUST 通过与 document output mode 分离的 `PlainText` 或等价明确命名通道承载。

#### Scenario: readable-json outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md --output readable-json`
- **THEN** 输出包含 entries、page 等 outline readable fields
- **THEN** 输出不包含 protocol envelope 字段

#### Scenario: 默认 readable-view outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md` 且未传入 `--output`
- **THEN** 输出使用 `@docnav-readable-view/1`
- **THEN** JSON header 包含 entries、page 和可选 warnings
- **THEN** 输出不包含 protocol envelope 字段

#### Scenario: readable-json warning envelope
- **WHEN** 调用方执行带有未知参数但其它参数有效的 readable-json 命令
- **THEN** 输出包含该 operation 的 readable 字段
- **THEN** 输出包含 `warnings` 数组
- **THEN** 每个 warning item 包含稳定 `id`、非空 `reason`、稳定 `effect` 和 `details` 对象
- **THEN** CLI argv warning 使用 `id: "cli_argv_ignored"`
- **THEN** 测试不要求 exact token 分组、`reason` 文案或 token 消费顺序

#### Scenario: readable-view warning envelope
- **WHEN** 调用方执行带有未知参数但其它参数有效的默认或显式 readable-view 命令
- **THEN** JSON header 包含该 operation 的 readable 字段和 `warnings` 数组
- **THEN** warning 不在 view block 后重复拼接为独立文本

#### Scenario: protocol-json warning
- **WHEN** 调用方执行带有未知参数但其它参数有效的 protocol-json 命令
- **THEN** 输出包含完整 protocol response envelope
- **THEN** 输出不包含 `warnings` 数组
- **THEN** stderr 包含 warning 诊断

#### Scenario: document text 输出模式被删除
- **WHEN** 调用方执行 `docnav outline docs/guide.md --output text`
- **THEN** `docnav` 返回 `INVALID_REQUEST`
- **THEN** CLI help 不把 text 列为 document output 可选值
- **THEN** `docnav` 不执行 document operation 后再 fallback 到其它输出模式

#### Scenario: help 和 version 仍可输出纯文本
- **WHEN** 调用方执行 `docnav --help` 或 `docnav --version`
- **THEN** CLI 可以输出普通纯文本
- **THEN** 该输出不使用 document `OutputMode::Text`
- **THEN** 该输出不需要带 `@docnav-readable-view/1` 版本行

#### Scenario: legacy defaults.output text 阻断普通 document execution
- **WHEN** 有效项目或用户配置包含 `defaults.output: "text"`
- **AND** 调用方执行 `docnav outline docs/guide.md`
- **THEN** `docnav` 返回配置错误
- **THEN** 错误包含配置路径、`defaults.output`、legacy value `text` 和修复命令
- **THEN** `docnav` 不把 text 静默映射为 readable-view

#### Scenario: config set 可以修复 legacy defaults.output
- **WHEN** 目标配置文件包含 `defaults.output: "text"`
- **AND** 调用方执行 `docnav config set defaults.output readable-view`
- **THEN** 命令成功写入 `defaults.output: "readable-view"`
- **THEN** 命令不会因为正在修复的 legacy output value 在加载阶段失败
- **THEN** 修复后 document operation 可以按 readable-view 默认输出执行

#### Scenario: config unset 可以移除 legacy defaults.output
- **WHEN** 目标配置文件包含 `defaults.output: "text"`
- **AND** 调用方执行 `docnav config unset defaults.output`
- **THEN** 命令成功从目标配置移除该字段
- **THEN** 命令不会因为正在修复的 legacy output value 在加载阶段失败
- **THEN** 修复后 document operation 使用内置默认 `readable-view`
