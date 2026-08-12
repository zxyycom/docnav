---
title: 在标记的语义字段上集中执行输出预算
status: archived
alignment: unaligned
createdAt: 2026-08-12T03:18:35Z
purpose: 在不统一各 operation 结果结构的前提下，让输出裁剪、成本计算和完整性状态共享同一条预算数据流。
background: 各 operation 与嵌套 auto-read 的结果形状不同，adapter 分别分页和计量会产生重复且不一致的成本路径。
decision: 由统一 OutputWindow 通过静态标记的语义字段执行计量和裁剪，所有预算判断复用同一 CostCalculator，fast-read 以同一机制进行有界探测。
relations:
  - type: 替代
    target: product-direction/repair-token-cost-as-bounded-debt.md
---

## 目的
- 让不同 operation 保持各自合适的语义结果结构，同时共享一致、易调整的输出预算机制。
- 消除 adapter、分页器、fast-read selector 和 public cost 各自维护 cost 数据流所造成的重复工作与语义偏差。

## 背景
- `ReadResult.content`、structured outline 的 `entries`、unstructured outline 的 `content`、find 的 `matches` 和 nested auto-read 无法通过一个固定字段形状统一处理。
- 既有“只计算实际返回内容”的方向无法覆盖 fast-read：判断全文是否足够小，必然需要在最终输出模式确定前检查候选全文，但超过阈值后无需继续计算。
- 直接按最终 JSON 或 readable 文本裁剪会把传输包装和展示差异引入预算，并可能破坏合法序列化结果。

## 决策
- 采用: `BudgetedOutput` 是结果类型暴露预算字段的静态遍历契约，`OutputWindow` 是执行计量和裁剪的 runtime controller，`CostCalculator` 是唯一的 unit-specific measurement owner，`OutputReport` 保存实际成本与完整性状态。
- 采用: OutputWindow 位于语义结果形成之后、raw/readable rendering 之前；各 operation 保持自己的 typed payload，不为预算处理扁平化成共同传输结构。
- 采用: 结果类型通过 trait implementation 或生成式字段标记声明参与预算的 text、sequence 和 nested semantic fields；生成机制只提供类型安全遍历，不拥有 tokenizer、预算或 presentation policy。
- 采用: 所有可能随输入规模增长的输出字段必须显式参与预算，或由独立上限证明为可安全跳过；新增字段不能静默绕过预算。Text 在 calculator 给出的合法边界截止，sequence 在 item 边界停止，nested result 递归复用同一个 window。
- 采用: 请求什么 cost unit 就只执行对应 calculator；不能先计算 tokens、lines 和 bytes 再过滤结果。公开 output cost 只描述实际接纳的标记字段，不再冒充分页前完整 selection cost。
- 采用: fast-read 使用同一 CostCalculator 的 bounded probe。候选输入先结束表示可完整承载，阈值先耗尽则立即停止并回退；失败 probe 的工作不进入最终 output cost，成功测量可以复用。
- 不采用: 把 serializer 截断、operation-specific paginator 或纯 returned-content estimator 继续作为并列成本 owner。
- 不采用: 让这一输出预算重构自动阻塞无关产品 change。
