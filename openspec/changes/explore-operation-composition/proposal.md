## 一句话核心

探索 Docnav 是否应在 core/SDK 层提供轻量 operation composition，用简单的现有命令组合减少常见阅读往返；具体命令形态留到后续实现 change 决定。

## 文档状态

本 change 只在 `openspec/changes/explore-operation-composition/` 下形成未审核的未来计划和探索材料，不影响现有其它文档、主规范或实现任务。

## Why

Docnav 的基础链路是 `outline -> ref -> read`。这个链路稳定、清楚，但在真实使用中会出现一些重复的机械步骤：先判断是否值得直接读全文、拿到多个 ref 后逐个读取、搜索后再读命中章节、读取某个 ref 周边上下文等。

这些能力的共同点不是某个具体命令，而是“由 core/SDK 组合现有 document operations 来提升使用体验”。它们通常不需要格式 adapter 理解新的业务语义，也不一定需要新增 public command。当前阶段更适合记录方向、候选模式、边界和后续决策问题，而不是提前固定任一方案。

## What Changes

- 将原实现导向的 change 收敛为探索导向的 `explore-operation-composition`。
- 记录一个未来方向：Docnav 可以在 core/SDK 层增加轻量 operation composition，优先复用现有 `outline`、`read`、`find` 和 `info`。
- 候选体验包括但不限于：
  - 根据文档规模在 outline 意图下直接返回内容或结构。
  - 对多个已知 ref 进行批量读取。
  - 在搜索结果明确时减少 find 后的手动 read。
  - 围绕某个 ref 获取相邻上下文。
- 当前 change 不选择主方案、不承诺命令名、不固定输入字段、不定义最终 schema。
- 当前 change 只保留架构边界：composition 优先归属 core/SDK；adapter 继续拥有格式解析、ref 生成/解析和单次 operation 语义；MCP 作为 bridge 映射到 `docnav`，不复制组合逻辑。
- 后续进入实现前，需要创建或更新更具体的 implementation change，届时再决定是扩展现有命令、增加 option、提供 SDK helper，还是新增极少数专用入口。后续 implementation change 必须按 `replace-text-with-readable-view` 的最终 typed readable shape 声明 content pointer 和 renderer config。

## Capabilities

### New Capabilities

- `operation-composition`: 记录 Docnav 在 core/SDK 层组合现有 document operations 的探索方向和边界。

### Modified Capabilities

- 暂无。本 change 不修改既有主规范的具体命令、字段或 schema。

## Impact

- 当前影响范围：OpenSpec 探索材料。
- 当前不影响代码、CLI、MCP tool、adapter protocol、schema、examples 或 docs 主规范。
- 后续实现可能影响 `core-cli`、`docnav-contracts`、`adapter-sdk`、MCP mapping、readable output schema 和示例，但需要在后续具体 change 中重新定稿。后续 change 必须按 `replace-text-with-readable-view` 的最终 typed readable shape 和 renderer config 声明 content pointer 和三种 document output mode。
