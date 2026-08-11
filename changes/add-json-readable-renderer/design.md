# Design

设计保持 JSON raw contract 不变，在 output 层以显式契约门禁确定 presentation，再由 linked core composition 选择格式专用 renderer。

## Context

- [输出模式](../../docs/output.md)当前要求 `ProtocolJson` 与 `Rendered(RenderStrategy)` 消费同一个 immutable `ProtocolResponse`；格式专用 renderer 只能改变 presentation。
- [JSON Adapter](../../docs/adapters/json.md)当前拥有 JSONC-capable parse/navigation/ref/raw facts，generic `readable-view` 已可用，JSON 专用 renderer 仍为 Planned。
- [展示契约批准后推进 JSON 专用阅读输出](../../docs/decisions/product-direction/advance-json-readable-presentation-after-contract-approval.md)确认本 change 应在 contract gate 关闭后独立推进。
- 本计划可以先执行批准门禁；门禁关闭前不得修改 owner、测试预期或 production renderer。

## Goals / Non-Goals

Goals:

- 为批准的 JSON operation/branch 提供稳定、信息密度合适且可继续导航的 presentation。
- 保持完整 opaque ref 可见而不推导层级语义。
- 让 raw facts、readable text、failure 和 renderer selection 都能独立验证。

Non-Goals:

- 不修改 JSON parsing、ordering、ref、pagination、cost 或 raw result。
- 不增加 public output value、renderer id、模板配置或 adapter callback。
- 不用 presentation 合成 response 中不存在的 hierarchy、depth、parent、indentation 或 preview fact。

## Decisions

### 1. JSON presentation 是独立 output handoff

Generic renderer 完成的是共享输出链路验收，不永久替代 JSON 专用展示；本 change 不重新打开已经完成的 raw adapter 验收。

### 2. Renderer 只消费 immutable `ProtocolResponse`

Renderer 在 stdout write 前返回完整 UTF-8 text 或 `RenderFailure`。它不读取 path、解析 ref、调用 adapter、改变 response 或在失败时 fallback。

### 3. Selection 属于 linked output composition

选定 renderer 的事实必须来自当前 linked composition 已有的可靠上下文，不进入 CLI/config/serialized contract。未选 adapter、提前 failure、非适用 branch 和 render failure 都使用门禁中批准的确定行为。

### 4. Presentation contract 先于 production 修改

Implementation 的第一组任务一次性批准 operation/branch、字段、信息密度、escaping、定位信号、preview、page/continuation 和 selection。任何答案要求新增 raw fact 时停止本 change，先重新判断 protocol/adapter owner。

## Risks / Trade-offs

- JSON presentation 可能需要当前 response 不存在的事实；门禁只允许消费已有 raw facts，否则重新定界而不是在 renderer 合成。
- 格式专用 text 可能被误当成 machine schema；测试分别证明稳定 readable contract 与 raw schema，不为 readable text 创建第二份 JSON schema。
- 其它 workstream 可能先改变 Current 输入；实施前做 scoped re-audit，但不建立统一前置依赖。
- 完整预渲染会使用额外内存；保持既有 output contract，并把资源风险纳入代表性/极端验证。

## Open Questions

以下问题由用户或指定 output/product owner 通过 Implementation 1.2–1.4 关闭，且只阻塞其后的 owner、测试与代码修改，不阻塞开始执行本计划：

1. 哪些 operation/branch 使用 JSON 专用 presentation？
2. 每个适用 branch 的稳定字段、信息密度、字段顺序、标点、escaping 和 block framing 是什么？
3. 怎样显示完整 opaque ref 作为定位信号而不解析或合成层级？
4. Preview 来自哪个现有 raw fact，是否截断以及上限和 spelling 是什么？
5. Page/continuation 怎样显示，哪些 facts 原样保留？
6. Linked composition 怎样选择 renderer，未选 adapter、提前 failure、非适用 branch 和 render failure 怎样处理？
