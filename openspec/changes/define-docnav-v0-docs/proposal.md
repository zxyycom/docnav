## Why

Docnav 当前只有项目目标和架构原则，尚缺少足以指导独立实现的 v0 契约文档。先定义产品流程、协议边界和 Markdown 行为基线，可以在编写 Rust 代码前消除网关、适配器和调用方之间的关键歧义。

## What Changes

- 定义 Docnav v0 的规范文档集合，覆盖产品目标、架构、共享协议、selector、适配器契约、CLI 和测试策略。
- 为 `outline -> selector -> read` 提供贯穿 MCP、适配器 `invoke` 和 Markdown 文档的完整机器可读示例。
- 将 `D:\project\skills\MarkdownNavigator` 中已验证的 Markdown 导航行为整理为参考基线，并明确哪些行为需要在 Docnav 中重新设计。
- 定义文档一致性、示例可校验性和后续实现可验收性的要求。
- 本变更不创建 Rust workspace，不实现 `docnav-mcp`、共享 library 或任何格式适配器，也不定义适配器安装与更新实现细节。

## Capabilities

### New Capabilities

- `v0-contract-documentation`: 定义 Docnav v0 必须提供的规范文档、跨文档一致性规则和端到端示例要求。
- `markdown-reference-baseline`: 定义从 MarkdownNavigator 提取并记录的 Markdown 导航行为、边界案例及迁移决策。

### Modified Capabilities

无。

## Impact

- 新增项目级 `README.md` 和 `docs/` 下的 v0 规范文档。
- 新增 OpenSpec requirements，作为后续 `docnav-protocol`、`docnav-adapter-sdk`、`docnav-markdown` 和 `docnav-mcp` 实现变更的输入。
- 不影响任何现有可执行制品、API 或依赖；当前仓库尚无实现代码。
