## 1. 建立参考基线

- [ ] 1.1 在 `docs/references/markdown-navigator.md` 记录 MarkdownNavigator 的仓库路径、参考提交、命令入口和黑盒测试验证结果。
- [ ] 1.2 按“保留、调整、推迟、移除”整理 heading、章节范围、重复项、frontmatter、代码围栏、编码和截断行为迁移表。
- [ ] 1.3 将参考项目的关键 fixtures 与黑盒场景整理为后续 Markdown 适配器测试输入清单。

## 2. 建立文档结构和术语

- [ ] 2.1 创建 `README.md`，说明产品目标、`outline -> selector -> read` 核心流程、v0 范围和规范文档索引。
- [ ] 2.2 创建 `docs/architecture.md`，定义制品职责、格式无关网关、独立适配器进程和完整调用链。
- [ ] 2.3 在主文档中统一 document、outline、node、selector、read、adapter、invoke、manifest、probe 和 capability 等术语。

## 3. 定义共享协议和 selector

- [ ] 3.1 创建 `docs/protocol.md`，定义协议版本、通用请求与响应 envelope、成功结果、结构化错误、能力声明和兼容规则。
- [ ] 3.2 在协议文档中明确 stdin、stdout、stderr、退出码和单请求 `invoke` 生命周期。
- [ ] 3.3 创建 `docs/selectors.md`，确定 selector 的结构、唯一定位、原样传递、跨进程使用、文档变化和失效语义。
- [ ] 3.4 明确禁止 read 使用最近位置静默消歧，并定义无匹配、多匹配和 selector 失效的结构化错误。
- [ ] 3.5 明确 v0 的文件编码策略，以及 `document_find` 和 `document_info` 在首版文档中的定义深度。

## 4. 定义适配器和 CLI 契约

- [ ] 4.1 创建 `docs/adapter-contract.md`，定义 `outline`、`read`、`find`、`info`、`invoke`、`manifest` 和 `probe` 的职责与共同约束。
- [ ] 4.2 定义 manifest、probe 候选结果、格式歧义和协议不兼容的稳定机器输出。
- [ ] 4.3 创建 `docs/cli.md`，区分 `docnav-mcp` 管理 CLI、MCP stdio 入口、适配器普通 CLI 和适配器 `invoke`。
- [ ] 4.4 记录人类可读输出、`--output json`、stdout、stderr、未知参数和退出码规则。

## 5. 编写端到端契约示例

- [ ] 5.1 选定一个包含嵌套和重复 heading 的最小 Markdown 示例文档。
- [ ] 5.2 提供该文档的 `document_outline` MCP 请求与响应，以及对应适配器 `invoke` 请求与响应。
- [ ] 5.3 从 outline 响应取得 selector，并提供 `document_read` MCP 请求与响应及对应适配器 `invoke` 请求与响应。
- [ ] 5.4 提供无匹配、多匹配、未知格式和协议不兼容的完整结构化错误示例。
- [ ] 5.5 将所有稳定 JSON 示例保存为可独立解析的示例文件，并验证其语法有效。

## 6. 定义验证策略并完成审计

- [ ] 6.1 创建 `docs/testing.md`，将稳定协议要求映射到 schema、单元、集成和端到端测试层级。
- [ ] 6.2 为每个适配器能力定义普通 CLI、`invoke` 和 MCP 调用链的最低测试要求。
- [ ] 6.3 检查所有协议字段、枚举、错误码和术语在各文档中的一致性，并消除重复规范来源。
- [ ] 6.4 验证所有文档链接、JSON 示例和 OpenSpec requirements，记录审计结果。
- [ ] 6.5 确认另一个实现者只依据 v0 文档即可提出协议与 Markdown 纵向链路实现变更。
