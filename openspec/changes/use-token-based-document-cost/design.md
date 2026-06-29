本 design 记录 SDK cost helper 与 Markdown token-informed cost 的初始设计方向；当前只在 `openspec/changes/use-token-based-document-cost/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

Docnav 需要把机器稳定协议字段和 readable 展示分层处理。cost 的结构化协议表示由单独 change 探索；本 change 只约束 SDK/helper 和 Markdown adapter 如何承接 cost 计算与展示。

## Goals / Non-Goals

**Goals:**

- 在 adapter SDK 中提供可复用的 cost / budget measurement helper。
- 让 adapter 保留 cost 计算策略、display 选择和排序的所有权。
- 让 Markdown adapter 能使用 token-informed measurement 表达文档成本。
- 在协议结构化方案确认后，同步适配最终 `cost` shape。

**Non-Goals:**

- 不开放用户选择任意分页预算方案。
- 不规定所有 adapter 必须展示同一组 cost unit。
- 不让 core 重新计算 adapter cost。
- 不在本 change 中决定 raw protocol 字段最终结构。

## Decisions

### Decision 1: SDK 提供机制，adapter 拥有策略

SDK 可以提供基础 measurement 类型、常用计数函数、简易 formatter 和可注入 measurement function 的分页 helper。Adapter 决定实际计算哪些 cost、展示哪些 unit、使用哪个 tokenizer，以及是否把某个 measurement 用作分页预算。

### Decision 2: Markdown token cost 建立在协议探索结果之上

Markdown adapter 的 token-informed cost 需要依赖 `structure-protocol-fields-and-readable-output` 定义的 raw protocol `cost` shape 后再落地到 schema、example 和 fixture。本 change 只保留 helper 和 adapter policy 的设计方向，不另行定义协议字段。

### Decision 3: 分页预算选择暂不对用户开放

SDK helper 可以为后续预算策略留下扩展点，但本 change 不新增用户可配置的 budget unit，也不要求 CLI/protocol 暴露预算函数选择。

## Risks / Trade-offs

- SDK 抽象过度会侵占 adapter policy；实现前需要审计 helper 是否只提供机制。
- Tokenizer 依赖可能影响构建和发布；实现前需要审计 crate、encoding、许可、离线行为和性能。
- 协议字段结构化尚未确认；实现任务必须等协议探索 change 收敛后再细化。

## Open Questions

- `structure-protocol-fields-and-readable-output` 定义的 `cost` shape 如何落到 SDK helper 和 Markdown adapter 输出？
- SDK helper 首期需要支持哪些基础 measurement？
- Markdown adapter 首期展示哪些 cost unit？
