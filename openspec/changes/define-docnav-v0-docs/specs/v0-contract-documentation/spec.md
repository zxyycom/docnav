## ADDED Requirements

### Requirement: 提供完整的 v0 契约文档集合
项目 MUST 提供 `README.md`、架构、协议、selector、适配器契约、CLI、测试策略和 Markdown 参考基线文档，并为每类规范指定唯一主文档。

#### Scenario: 审计者定位规则所有权
- **WHEN** 审计者需要确认 selector 的规范行为
- **THEN** 项目文档索引将其引导至 `docs/selectors.md`
- **THEN** 其他文档不提供相互冲突的 selector 定义

### Requirement: 文档以中文为主要审计语言
所有面向审计者的说明、设计理由、规范要求和任务 MUST 以中文为主，必要的命令、路径、协议字段、枚举和错误码 MUST 保留英文机器标识。

#### Scenario: 审计协议示例
- **WHEN** 审计者阅读包含 JSON 的协议章节
- **THEN** 字段名和枚举值使用稳定英文标识
- **THEN** 字段含义、约束和设计理由使用中文说明

### Requirement: 定义完整的 outline 到 read 示例
文档 MUST 使用同一个 Markdown 输入提供完整的 `outline -> selector -> read` 示例，并展示 MCP tool 边界与适配器 `invoke` 边界上的请求和响应。

#### Scenario: 从 outline 结果读取章节
- **WHEN** 实现者按照文档示例取得 outline 节点中的 selector
- **THEN** 该 selector 可以原样放入 read 请求
- **THEN** 示例明确展示预期读取内容和响应 envelope

### Requirement: 区分稳定机器契约与可定制文案
协议文档 MUST 明确字段名、枚举值、错误码和 schema 属于稳定机器契约，并明确 guidance、usage 和错误建议属于可定制用户文案。

#### Scenario: 项目配置修改 guidance
- **WHEN** 项目配置修改 guidance 或错误建议文本
- **THEN** 请求和响应 schema 保持不变
- **THEN** 调用方无需解析文案即可判断结果

### Requirement: 定义协议兼容性和进程边界
架构与协议文档 MUST 定义 `docnav-mcp stdio`、适配器 `invoke`、stdin、stdout、stderr、退出码和协议版本的职责及兼容规则。

#### Scenario: 适配器 invoke 失败
- **WHEN** 适配器处理单个 `invoke` 请求失败
- **THEN** 文档要求其尽可能向 stdout 输出单个结构化错误响应
- **THEN** 文档要求其返回非零退出码并只向 stderr 写诊断日志

### Requirement: 文档契约可映射到自动化验证
测试策略文档 MUST 将每项稳定协议行为和关键端到端示例映射到后续 schema、单元、集成或端到端测试。

#### Scenario: 实现首条 Markdown 调用链
- **WHEN** 后续变更实现 Markdown `outline -> selector -> read`
- **THEN** 实现者能够从测试策略文档识别必须新增的协议兼容、invoke 和 MCP 调用链测试
