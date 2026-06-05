## 1. 建立参考基线

- [x] 1.1 在 `docs/references/markdown-navigator.md` 记录 MarkdownNavigator 的仓库路径、参考提交、命令入口和黑盒测试验证结果。
- [x] 1.2 按“保留、调整、推迟、移除”整理 heading、章节范围、重复项、frontmatter、代码围栏、编码和分页行为迁移表。
- [x] 1.3 将参考项目的关键 fixtures 与黑盒场景整理为后续 Markdown 适配器测试输入清单。

## 2. 建立文档结构和术语

- [x] 2.1 创建 `README.md`，说明产品目标、`outline -> ref -> read` 核心流程、v0 范围和规范文档索引。
- [x] 2.2 创建 `docs/architecture.md`，定义 CLI-first 制品职责、接入层、独立 adapter 进程和完整调用链。
- [x] 2.3 在主文档中统一 document、outline、ref、page、read、adapter、invoke、manifest、probe 和 capability 等术语。

## 3. 定义原始协议和 ref

- [x] 3.1 创建 `docs/protocol.md`，定义原始协议版本、请求响应 envelope、紧凑语义结果、结构化错误、page 和兼容规则。
- [x] 3.2 在协议文档中明确 stdin、stdout、stderr、退出码和单请求 `invoke` 生命周期。
- [x] 3.3 创建 `docs/refs.md`，确定 path 与 ref 的职责边界、唯一定位和原样传递。
- [x] 3.4 明确禁止 read 使用最近位置静默消歧，并定义无匹配和多匹配的结构化错误。
- [x] 3.5 明确 v0 的文件编码策略，以及 `document_find` 和 `document_info` 在首版文档中的定义深度。

## 4. 定义适配器和 CLI 契约

- [x] 4.1 创建 `docs/adapter-contract.md`，定义 `outline`、`read`、`find`、`info`、`invoke`、`manifest` 和 `probe` 的职责与共同约束。
- [x] 4.2 定义 manifest、probe 候选结果、格式歧义和协议不兼容的稳定机器输出。
- [x] 4.3 创建 `docs/cli.md`，区分 `docnav` 核心 CLI、MCP bridge、adapter 直接 CLI 和 adapter `invoke`。
- [x] 4.4 记录独立配置域、人类可读输出、`protocol-json`、`readable-json`、stdout、stderr、未知参数和退出码规则。

## 5. 编写端到端契约示例

- [x] 5.1 选定一个包含嵌套和重复 heading 的最小 Markdown 示例文档。
- [x] 5.2 提供该文档的 `document_outline` MCP 请求与精简响应，以及对应适配器 `invoke` 请求与原始协议响应。
- [x] 5.3 从 outline 响应取得 ref，并提供 `document_read` MCP 请求与精简响应及对应适配器 `invoke` 请求与原始协议响应。
- [x] 5.4 提供 page、无匹配、多匹配、未知格式和协议不兼容的完整结构化示例。
- [x] 5.5 将所有稳定 JSON 示例保存为可独立解析的示例文件，并验证其语法有效。

## 6. 定义验证策略并完成审计

- [x] 6.1 创建 `docs/testing.md`，将原始协议稳定性和阅读输出信息密度映射到 schema、单元、集成和端到端测试层级。
- [x] 6.2 为每个适配器能力定义普通 CLI、`invoke` 和 MCP 调用链的最低测试要求。
- [x] 6.3 检查所有协议字段、枚举、错误码和术语在各文档中的一致性，并统一为扁平 outline、ref 定位和分层输出契约。
- [x] 6.4 验证所有文档链接、JSON 示例和 OpenSpec requirements，记录审计结果。
- [x] 6.5 确认另一个实现者只依据 v0 文档即可提出协议、Markdown 四项能力和 `outline -> ref -> read` 纵向链路实现变更。
- [x] 6.6 吸收审计后的产品决策：Markdown v0 全部四项能力、GitHub/本地 exe adapter 安装来源、本地 exe hash 校验、MCP 直连核心 CLI、正式 adapter 安装更新、显式格式选择顺序和 read `content_type` 阅读输出。
- [x] 6.7 优化文档阅读路径：README 作为角色化入口，主规范拥有规则，schema/示例为校验材料，OpenSpec 为审计历史。
