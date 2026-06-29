本 change 记录为文档 cost 引入 SDK helper 与 Markdown token-informed 展示的方向；当前只在 `openspec/changes/use-token-based-document-cost/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

当前文档成本信息主要作为可读字符串散落在 adapter 输出中，复用成本低，也不利于 Markdown adapter 表达更接近模型上下文消耗的 token-informed cost。

## What Changes

- 为 adapter SDK 探索通用 cost / budget measurement helper，让 adapter 可以复用基础计算和格式化工具。
- 让 Markdown adapter 基于 SDK helper 输出 token-informed cost。
- 保持 cost 展示策略由 adapter 拥有；SDK 只提供基础类型、计算工具和简易 display helper。
- 依赖协议字段结构化探索结果确认 raw protocol 中 `cost` 的最终 shape。
- 非目标：本 change 不开放用户选择分页预算单位，也不把 token cost 强制推广到所有 adapter。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `adapter-protocol`: 为 adapter SDK cost/budget helper 留出共享抽象边界。
- `markdown-navigation`: 使用 SDK helper 生成 Markdown token-informed cost，并由 Markdown adapter 决定展示内容。

## Impact

- 受影响代码：`docnav-adapter-sdk` cost helper、`docnav-markdown` cost 计算与展示、相关测试和 fixture。
- 受影响文档：adapter SDK 边界、Markdown adapter cost 说明，以及后续协议结构化 change 确认后的 schema/example。
- 依赖关系：协议字段 shape 由 `structure-protocol-fields-and-readable-output` 定义，本 change 不单独决定 raw protocol `cost` 字段结构。
