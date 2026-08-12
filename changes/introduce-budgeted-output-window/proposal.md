# Proposal

本 Plan 提供一条通用、调用方主动推送的增量输出会话。Operation 可以边生产、边计量、边构造自己的结果，并在 `finish` 后把完整 typed result 交给现有响应与展示管线。带单位的输出预算是这条会话通过 Limited Gate 获得的一种组合能力。Change 目录保留最初的预算窗口身份；本轮确认后的目标 capability 和 crate 使用更准确的 `OutputSession` 名称。

## Why

Read 文本、outline/find 条目和 operation-owned item 最终形成不同结果类型，但它们共享同一种生产节奏：生成一个原子输入、提交、根据结果决定是否继续。把这条节奏抽成共享会话，可以让流控制、输入计量、结果构造与最终展示分别由稳定 owner 承担，也让 Limited 和 Unbounded 使用同一个 producer/collector 调用形状。

本 Plan 获准时，Markdown 与 JSON adapter 会先构造完整 text 或 entry collection，再执行各自的分页处理；`docnav-text-cost` 也只计算完整 `&str`。该基线缺少一个能在 producer boundary 组合调用方输入语义、requested-unit measurement 和 operation-specific result construction 的 shared capability。

[用 Gate、计量策略与 Collector 组合增量输出](../../docs/decisions/product-direction/compose-incremental-output-through-gates-policies-and-collectors.md)已经确认长期方向：`OutputSession` 组合 Gate、`InputCost` 和 Collector。Gate 决定接纳与流控制，Limited Gate 才注入计量策略，Collector 逐项保存或构造 operation output；Navigation 仍在 finish 后包装完整 `ProtocolResponse`，raw/readable presentation 仍消费同一响应。

## Outcome

Workspace 获得一个尚未接入 Current public behavior 的 `docnav-output-session` shared crate：调用方选择单一输入类型 `I` 和 Collector，以 Limited 或 Unbounded Gate 创建 `OutputSession`，只在前一次 outcome 允许继续时提交下一项。获准输入只移动一次到 Collector；`finish` 返回 Collector 的 typed output 和 `OutputReport`。

Limited Gate 使用 `InputCost<I>` 执行原子 admission。文本场景通过显式 `TextProjection<I>` 把一个输入投影为有顺序的语义文本片段，并由 requested-unit `TextMeter` 作为一个逻辑流计量 lines、bytes 或 tokens。Unbounded Gate 跳过整个计量策略，但保持相同的 producer、Collector 和 finish 形状。

## Scope

本 Change 纳入：

- 由 `docnav-protocol` 提供、但尚不进入 Current wire shape 的 shared `CostUnit`。
- `docnav-text-cost` 中 requested-unit 的 bounded text measurement session；一个 session 把 Projection 提供的多个片段按顺序视作一个逻辑文本流，并只运行请求的 unit。
- 新 `docnav-output-session` shared crate中的泛型 `OutputSession`、Limited/Unbounded Gate、`InputCost<I>`、`TextProjection<I>`、Collector、structured push outcome 和最终 report。
- Limited Gate 的原子 admission：当前输入完整放得下才提交给 Collector；放不下则拒绝并停止。核心不裁剪输入内部 prefix。
- String/text chunk identity projection、String/`Vec<I>` Collector，以及 caller-owned structured projection 和 operation-specific Collector 的直接组合证据。
- Limited/Unbounded 复用、投影/计量组合、Collector commit、source completion 和 producer early-stop 的 focused evidence。
- capability 落地后对 architecture、testing owner、Semantic Case 和 downstream integration gate 的同步。

本 Change 不纳入：

- CLI/config/machine input、protocol `0.2` wire shape、默认 limit、public output sidecar、pagination removal 或 compatibility behavior；这些由 [replace-pagination-with-unit-output-limits](../replace-pagination-with-unit-output-limits/design.md)拥有。
- Current `AdapterDocument` operation signature 的切换、Markdown/JSON producer refactor、navigation 接入或 live CLI early-stop；这些是 downstream public integration 的工作。
- stdout 或 protocol transport streaming；本 Change 的增量性只发生在 producer、Session 和 Collector 之间。
- 最终 raw/readable serialization size、billing-grade cost、最大填充、至少接纳一项或数字 latency/RSS SLA。
- Fast-read probe reuse、跨阶段 measurement cache 或一般性 performance framework。

## Success Criteria

- 一个 Session 实例只接受一种 caller-defined input `I`；Gate、InputCost 和 Collector 核心不按 text、entry、nested 或 operation 建立 runtime variant。
- Limited Gate 对输入执行原子 admission：`cost < remaining` 为 accepted/continue，`cost == remaining` 为 accepted/stop，`cost > remaining` 为 rejected/stop；只有 accepted input 才提交给 Collector。
- Limited outcome 无歧义表达 input disposition、flow、`unit/limit/used/remaining` 和停止原因；Unbounded outcome 使用同一 input/flow 控制形状且不伪造预算快照。
- Limited 状态满足 `used + remaining = limit` 和 `used <= limit`；rejected input、measurement error 和 stopped-session push 不改变 Gate 或 Collector 状态。
- `TextProjection<I>` 可以向一个 bounded `TextMeter` 提供多个借用片段；跨片段的 lines/bytes/tokens 结果与把这些片段按相同顺序连接成一个逻辑文本后使用现有 calculator 的结果一致。Meter 只有在能够证明该输入已经超过 threshold 时才阻止 Projection 继续提供后续片段。
- Collector 收到 accepted input 的原始所有权且每项只移动一次；String builder、泛型 `Vec<I>` 和一个 operation-specific fake builder 证明 Session 不固定物化 `Vec<I>`。首版 accepted-item commit 为 infallible，`finish` failure 在完整响应形成前返回。
- `finish` 返回 Collector typed output 和 Gate report；source completion 由 producer owner 提供。Navigation/renderer 只在后续消费完整 result，不成为并列 admission、cost 或 partial-state owner。
- Limited 与 Unbounded 运行同一个 lazy producer/Collector reference loop；Unbounded 不构造或调用 InputCost，Limited stop 后不再请求 producer tail。
- 本 Change 不改变 Current CLI help、config、protocol schema/example、adapter output 或 release behavior；production crate、focused tests、architecture owner 与 Case 足以独立证明 shared capability。

## Affected Owners

- [架构](../../docs/architecture.md)：登记 future producer → Session(Gate + InputCost + Collector) → typed result → `ProtocolResponse` → presentation 边界及 shared crate 依赖方向。
- [`docnav-protocol`](../../crates/shared/protocol/src/cost_unit.rs)：shared `CostUnit` Rust enum；本 Change 不改变 `0.1` wire shape。
- [`docnav-text-cost`](../../crates/shared/text-cost/src/lib.rs)：从完整文本 helper 扩展 requested-unit bounded `TextMeter`，并拥有跨片段逻辑文本的 lines/bytes/tokens 语义。
- 新 `docnav-output-session` shared crate：Session、Gate、InputCost/Projection contract、Collector、outcome、report 及最小组合实现。
- [测试策略](../../docs/testing.md)及 Current test evidence：组合边界、atomic admission、跨片段 measurement、Collector commit、reference producer stop 和 Limited/Unbounded reuse。
- [replace-pagination-with-unit-output-limits](../replace-pagination-with-unit-output-limits/design.md)：后续将真实 adapter producer 接到 Session，并按活动决策修订其 post-result budgeting 描述；本 Change 不推进其 lifecycle。
