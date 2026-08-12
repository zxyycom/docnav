---
title: 用 Gate、计量策略与 Collector 组合增量输出
status: active
alignment: unaligned
createdAt: 2026-08-12T10:56:19Z
purpose: 让 operation 通过统一的逐项输出会话组合流控制、输入计量和结果构造，并将完整结果交给既有响应与展示层。
background: 不同 operation 需要边生产边形成不同结果类型；预算判断、文本投影、内容保存与最终消费具有稳定而彼此独立的变化方向。
decision: OutputSession 组合 Gate、InputCost 与 Collector；接纳输入只移交一次，finish 形成 typed result 和报告，再进入完整响应管线。
relations:
  - type: 修订
    target: product-direction/centralize-output-budgeting-over-marked-semantic-fields.md
---

## 目的
- 建立一条可组合的增量输出构造链路，让不同 operation 复用相同的逐项控制协议，同时保留各自自然的输入粒度和结果类型。
- 让流控制、输入计量、结果构造和最终展示各自拥有单一责任，使每个部分能够独立选择实现、测试和演进。
- 让调用方在每次提交后获得继续或停止所需的稳定状态，并在结束时得到可直接进入现有响应管线的完整 typed result。

## 背景
- Read 文本、outline/find 条目和 operation-owned item 的自然结果结构不同，但它们都需要同一种“生成一项、提交一项、根据结果决定下一项”的调用协议。
- 输入的预算成本来自调用场景选择的语义文本，而不是输入类型的固定结构属性。文本选择方式与 lines、bytes、tokens 的计量算法也具有不同的变化方向。
- 已接纳内容可以适合不同的增量构造方式，例如字符串 builder、条目集合或 operation-specific builder；把所有内容固定物化为同一种中间集合会增加后续转换。
- Current navigation 和 presentation 以完整、不可变的 `ProtocolResponse` 为共同事实。Producer-time 增量构造需要与该边界衔接，而不是把半成品变成 renderer 或 stdout 的输入。

## 决策
- 采用: OutputSession 把单一输入类型 `I`、Gate 和 Collector 组成共享的逐项输出边界；Limited Gate 另外组合 InputCost policy。调用方拥有 producer 和输入粒度；Session 依次使用 Gate 决定是否接纳，并只把获准的原始输入移动一次交给 Collector。`push` 返回当前输入处置、下一步流控制和 Gate 报告所需的稳定状态。
- 采用: Gate 拥有接纳与继续/停止语义。Limited Gate 使用带单位的 limit 和 InputCost policy 执行原子 admission；Unbounded Gate 直接接纳且不创建或调用计量策略。两种模式复用相同的 producer、Collector 和 finish 调用形状。
- 采用: `InputCost<I>` 是 Limited Gate 注入的输入计量策略。Gate 拥有已归一化的 unit 和 remaining threshold，并在每次 measurement 时把两者传给 policy。文本成本场景由调用点选择的 `TextProjection<I>` 与 requested-unit `TextMeter` 组合：Projection 借用输入并按语义顺序向 Meter 提供文本片段，Meter 只执行请求的 lines、bytes 或 tokens 计量，并在能够证明输入已经超出 bounded threshold 时停止继续投影。
- 采用: Projection 显式拥有输入到计量文本的选择与连接顺序，作为调用场景的可替换 policy；同一输入类型可以在不同 operation 使用不同 Projection。类型字段本身不承担全局预算语义，Window 核心也不反射或遍历业务结构。
- 采用: Collector 拥有已接纳内容的增量保存和结果构造。Collector 根据 operation 选择 String builder、`Vec<Entry>` 或 operation-specific builder；`accept(I)` 接收已经获准的原始输入，`finish` 形成 Collector 的 typed output。首个共享契约要求 accepted-item commit 不失败，或提供等价事务语义，使预算状态与 Collector 状态作为一次原子接纳提交。
- 采用: `finish` 消费 Session，返回 Collector 形成的 typed result 与 OutputReport。拥有 producer 的调用方确认 source 是否自然结束；Gate 只报告自身停止事实，不从 remaining 推断内容完整性。
- 采用: Navigation 在 finish 后校验 typed result 并包装完整、不可变的 `ProtocolResponse`；protocol-json 与 readable renderer 继续消费同一完整响应。这里的增量性发生在 producer、Session 和 Collector 之间，不改变现有最终响应与 stdout 的完整提交边界。
- 采用: Output limit 是 Limited Gate 提供的一种组合能力，用来约束接纳的输出规模并让 producer 及时停止；它不定义整条输出架构的中心。Projection 的可调用契约拥有输入语义，未来的宏、derive 或其它代码生成只可实现该契约的便利层。
