# Design

本设计拥有 producer-time `OutputSession` shared capability。Change 目录名保留最初的预算窗口身份；目标类型和 crate 使用架构细化后更准确的 Session 命名。Public limit cutover、真实 adapter integration 和 fast-read reuse 分别由相邻 Change 拥有。

## Context

### Authority and evidence

- [用 Gate、计量策略与 Collector 组合增量输出](../../../docs/decisions/product-direction/compose-incremental-output-through-gates-policies-and-collectors.md)是 active、unaligned 的长期方向：逐项输出由 Gate、InputCost policy 和 Collector 组合；Limited 只是 Gate 的一种模式；finish 后仍形成完整 typed result 和 `ProtocolResponse`。
- [用带单位的输出上限替代分页](../../../docs/decisions/product-direction/replace-pagination-with-unit-output-limits.md)固定 public limit/ignore-limit、移除 page/continuation 和 public complete；本 Change 不重复拥有这些产品选择。
- [保留当前 reference tokenizer](../../../docs/decisions/product-direction/retain-current-reference-tokenizer-until-qualified-replacement.md)继续固定 `tiktoken-rs / o200k_base / ordinary-text` production backend；backend identity 不进入 public contract。
- Current `AdapterDocument::outline/read/find` 返回完整结果。多条 Markdown/JSON path eager collect 或 serialize，Current navigation 只在 operation 完成后获得 `OperationResult`。
- Plan baseline 中，`docnav-text-cost` 只提供完整 `&str` 的 lines、bytes 和 tokens measurement，也没有 closed `CostUnit`、bounded text session、OutputSession、Gate、InputCost/Projection 或 Collector contract。本 Change 已新增这些 shared capabilities，但没有改变上一条 Current adapter/navigation 调用链。
- Current `docnav-output` 的 protocol-json 与 readable branches 都消费同一个完整 `ProtocolResponse`；readable renderer 先形成完整 UTF-8 `String`，再写 stdout。

### Target data flow

```text
producer --生成一个 I--> OutputSession<I, G, C>.push(I) --PushOutcome--> producer
                              |        |
                              |        +-- C: Collector 接收 accepted I
                              +----------- G: Limited<P> | Unbounded Gate
                                             |
                                             +-- P: Limited 才使用的 InputCost
                              |
                              | finish(source_completion)
                              v
                   typed output + OutputReport
                              |
                              v
                 navigation validate / ProtocolResponse
                              |
                              v
                   protocol-json / readable output
```

Producer 只有在前一次 outcome 允许继续时才生成下一项。Session 不拉取 producer；renderer 不观察 Session 的中间状态。

### Dependency order

1. 本 Change 提供 shared Session、Gate、InputCost/Projection、bounded TextMeter、Collector 和 direct evidence。
2. `replace-pagination-with-unit-output-limits` 修改 adapter/navigation boundary，让真实 producer 使用 Session 并接入 public request/report。
3. `integrate-fast-read-budget-probing` 按自己的 candidate identity/admission contract 复用 bounded measurement mechanics。

Stage、active decision 和 capability tests 都不证明 Current CLI 已经早停，也不自行授权相邻 Change 实施。

## Goals / Non-Goals

Goals:

- 为任意 caller-defined input `I` 提供同一个逐项 push/finish 协议，并允许 Limited/Unbounded 复用调用形状。
- 让 Gate、InputCost/Projection、Collector、producer 和 presentation 各自拥有一个稳定责任。
- 让 accepted input 只移动一次到 operation-selected Collector，由 Collector 增量形成 typed output。
- 让文本 Projection 逐段提供借用内容，TextMeter 在 bounded threshold 下按一个逻辑文本流执行 requested-unit measurement。
- 让 tests 直接证明组合边界、原子状态与 canonical producer loop。

Non-Goals:

- 不在本 Change 改写 public request/response、默认值、pagination 或 release behavior。
- 不让 Session 解释 operation 语义、主动拉取 producer、裁剪输入内部内容或执行 presentation transformation。
- 不要求 adapter 使用统一的结果类型或固定 `Vec<I>` 中间表示。
- 不改变完整 `ProtocolResponse` 与 stdout 的最终提交边界。
- 不建立数字 latency/RSS SLA。

## Decisions

### D1. [Inherited] 输出会话以可组合的逐项构造为中心

实现使用 `OutputSession<I, G, C>` 连接调用方选择的单一输入 `I`、Gate 和 Collector；Limited Gate 以 `LimitedGate<P>` 持有 InputCost policy，Unbounded Gate 不持有 policy。输入类型及粒度由 producer owner 选择；Collector output 由 operation 选择。

Session 统一执行一次 push 的协调顺序和 finish 生命周期，但不成为输入语义、cost 算法、结果类型或 presentation 的 owner。

### D2. [Inherited] Gate 拥有接纳与流控制

Gate 使用统一的控制事实：当前输入 `Accepted | Rejected`，下一步 `Continue | Stop(reason)`。Limited Gate 额外拥有带单位的预算状态；Unbounded Gate 对每项直接返回 accepted/continue，并在 source 自然结束后 finish。

Limited 与 Unbounded 复用 Session、producer 和 Collector；Unbounded 不使用伪造的最大 limit，也不创建或调用 InputCost。为使 caller 不依赖模式分支，outcome/report 使用显式 mode variant：

```text
PushOutcome
  input: Accepted | Rejected
  flow: Continue | Stop(StopReason)
  gate:
    Limited(BudgetSnapshot)
    Unbounded
```

### D3. [Change-local] Limited Gate 对当前输入执行原子 admission

Limited Gate 先借用 `&I` 调用 InputCost，只有完整输入可容纳时才把 `I` 移交 Collector：

| 条件 | 当前输入 | Gate 变化 | Collector | 下一步 |
| --- | --- | --- | --- | --- |
| `cost < remaining` | Accepted | `used += cost` | `accept(I)` 一次 | Continue |
| `cost == remaining` | Accepted | `used = limit` | `accept(I)` 一次 | Stop(`LimitExhausted`) |
| `cost > remaining` | Rejected | 不变 | 不调用 | Stop(`InputDoesNotFit`) |

Session 不返回 partial input，也不为至少一项突破 limit。需要更细早停时，producer 选择更小但仍合法的 `I`。

### D4. [Change-local] Accepted commit 同时覆盖 Gate 与 Collector

首版 `Collector::accept(I)` 是 infallible。Session 在 InputCost 成功且输入可容纳后调用 accept，并提交预算状态；由于 accept 不返回业务失败，外部观察不到“预算已扣除但内容未保存”或相反的半提交状态。

Measurement failure、违反 `InputCost` bounded contract 的 measurement 和 stopped-session misuse 使用 error channel，并发生在 Collector commit 前。Gate 已证明 `cost <= remaining` 后，`used + cost <= limit` 是局部程序不变量，不建立不可达的 overflow error variant。`Collector::finish` 可以失败；此时尚未形成 `ProtocolResponse`，整个 operation 按既有 failure boundary 结束且不写 partial stdout。未来若需要 fallible accept，必须先设计显式 transaction/rollback contract，不能直接扩张首版 trait。

### D5. [Change-local] Limited outcome 暴露稳定预算事实

`BudgetSnapshot` 包含 `unit`、`limit`、`used` 和 `remaining`。所有 snapshot 满足：

- `used + remaining = limit`；
- `used <= limit`；
- snapshot unit 是 Limited Gate 构造时传入并在每次 measurement 中传给 InputCost 的同一单位；
- rejected/error 不改变 Gate 或 Collector；
- stop 后不能重新 open。

Outcome 不暴露 tokenizer token、incremental buffer、projection cursor 或 trial checkpoint。`LimitExhausted` 表示当前输入已接纳且预算恰好耗尽；`InputDoesNotFit` 表示当前输入未接纳。

### D6. [Inherited] InputCost 组合 Projection 与 TextMeter

`InputCost<I>` 接收 Gate 已归一化的 unit 和 remaining threshold，只为一个借用输入返回 Limited Gate 所需的 bounded measurement。文本场景使用两个独立 owner：

```text
TextProjection<I>
  project(&I, &mut TextSink)

TextMeter
  requested unit
  remaining threshold
  consume(&str) -> Continue | ProvenExceedsThreshold
  finish() -> Fits(cost) | ExceedsThreshold
```

Projection 决定哪些语义文本、以什么顺序和哪些显式连接片段进入 cost；它可以零分配地提供多个借用 `&str`。TextMeter 决定 lines、bytes、tokens 的 measurement，并且只有在当前 unit 的算法已经能够证明完整输入必然放不下时，才让 Projection 停止提供后续片段。

String/text chunk 使用 identity projection。结构化业务类型的 projection 位于理解该 operation 语义的调用方 policy 模块，而不进入 shared Session crate；本 Change 使用 caller-owned structured test projection 证明该组合边界。同一 `I` 可以在不同 operation 使用不同 Projection。

### D7. [Change-local] 多片段按一个逻辑文本流计量

一次 Projection 提供的有序片段及其显式连接文本共同构成该输入的逻辑计量文本。TextMeter 的最终结果必须等于把这些片段按相同顺序连接后调用现有 `line_cost`、`byte_cost` 或 `token_cost` 的结果；不能简单相加每段的独立 line/token cost，因为行首尾和 tokenizer merge 会跨片段产生状态。

首版可以为 token unit 在 Meter 内部暂存当前输入的 projected text，再调用唯一 `o200k_base` calculator；这仍避免构造完整 operation output，并让 Session 在当前 input boundary 后停止 producer tail。接口允许后续在保持等价语义时替换为能够提供安全 bounded proof 的 incremental backend。Bytes 和 lines 可以直接维护跨片段状态。任何优化都由等价性与 bounded-stop tests 约束，不改变 Projection 或 Session contract。

### D8. [Inherited] Collector 拥有保存方式和 typed output

Collector contract 使用关联 output，而不是 Session 固定持有 `Vec<I>`：

```text
Collector<I>
  type Output
  accept(I)                  // 首版 infallible
  finish() -> Result<Output>
```

String builder 可以直接连接 text chunk，entry collector 可以形成 `Vec<Entry>`，operation-specific builder 可以直接形成自己的 result。Session 只保证 accepted item 的顺序和 exactly-once transfer，不要求 Collector 暴露中间内容。

### D9. [Inherited] Finish 后进入完整响应管线

`finish(source_completion)` 消费 Session 和 Collector，返回 typed output 与 `OutputReport`。`InputDoesNotFit` 能证明 incomplete；`LimitExhausted` 只说明 Gate 已满，如果 caller 同时证明当前输入就是 producer 最后一项，结果仍可 complete。Unbounded 在 source 自然结束时 complete。

Navigation 随后校验 typed output 并包装完整、不可变的 `ProtocolResponse`；protocol-json 与 readable renderer 消费同一响应。Producer-time push 是内部增量构造，不把 renderer、channel 或 stdout 变成 partial-state consumer。

### D10. [Change-local] Shared crate 与依赖方向

新增 `crates/shared/output-session` / `docnav-output-session`，拥有 Session、Gate、InputCost/Projection contract、Collector、outcome/report 和最小组合实现。它依赖 `docnav-protocol` 的 `CostUnit` 与 `docnav-text-cost` 的 TextMeter，不依赖 navigation、output 或具体 adapter。

`docnav-protocol` 只新增 internal shared Rust `CostUnit::{Lines, Bytes, Tokens}`；本 Change 不改变 Current wire。Future adapter/operation 作为 producer 和 policy owner，navigation 选择 Limited/Unbounded 并在 finish 后形成 `ProtocolResponse`，`docnav-output` 保持 presentation owner。

### D11. [Change-local] Downstream integration handoff

Downstream `AdapterDocument`、Markdown、JSON 与 navigation integration 必须共同满足以下义务，才能把 shared capability 描述为真实 document-operation early stop：

1. Operation producer 选择一种可独立接纳的输入 `I`，只在前一次 outcome 为 Continue 时生成下一项；收到 Stop 后不再访问 producer tail。
2. 理解 operation 语义的调用点选择 `TextProjection<I>` 和显式 fragment/连接顺序；shared Session、renderer 和 serializer 不推断业务字段。
3. Navigation 从规范化后的 constraint 选择 Limited 或 Unbounded Gate；Unbounded 不构造 InputCost，Limited 的 unit/limit 与最终 public report 使用同一事实。
4. Operation 选择能够直接形成目标 typed result 的 Collector；accepted input exactly once 移交，rejected input 不进入 result。Collector finish 先于 result validation 和 `ProtocolResponse` 构造。
5. Producer owner 向 finish 提供 source completion；navigation 校验完整 typed result 后才交给 protocol-json/readable presentation，任何层都不把 partial Session state 写到 stdout。

`replace-pagination-with-unit-output-limits` 当前的 typed-result 后 traversal/prefix cropping 描述，以及 `integrate-fast-read-budget-probing` 当前对旧字段标记方向的引用，都需要在各自下一实施步骤前按活动长期决策重新审阅。本 Change 只交付并证明上述 shared capability，不修改相邻 artifacts、推进其 lifecycle 或代替真实 integration evidence。

## Risks / Trade-offs

- 本 Change 只交付 capability；Current adapters 在 downstream 接入前仍然 eager，不能声称 CLI 已减少生成工作。
- Session 只能在 input boundary 停止。异常大的单项仍可能产生显著临时生成和计量工作；真实 integration 需要按 operation 选择合理粒度。
- Atomic rejection 可能留下 unused remaining；契约优化可预测的逐项构造，而不追求最大填充。
- Token unit 首版可能暂存一个输入的 projected text；这提供组合 API 和 operation-level early stop，但不是已证明的跨片段 constant-memory tokenizer streaming。
- Projection 明确选择成本文本，因此 window cost 可能受 input granularity 和连接策略影响；它是可调整的输出控制 accounting，不是最终 response serialization size。
- `replace-pagination-with-unit-output-limits` 与 `integrate-fast-read-budget-probing` 仍引用旧 post-result/field-traversal 方向；它们在下一次实施前必须按活动决策重审，但本 Change 不自行修改或推进其 lifecycle。

## Open Questions

无。用户已确认单输入 push、结构化 outcome、Gate/InputCost/Collector 分层、显式 TextProjection、requested-unit TextMeter、Limited/Unbounded 复用以及 finish 后完整响应边界。当前 Rust 落点是 `OutputSession<I, G, C>`、`LimitedGate<P>` 和借用式 `TextProjection<I>`；tests 约束其行为契约。
