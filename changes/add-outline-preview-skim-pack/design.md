# Design

设计由 core 在一次 invocation 内组合现有 outline/read，通过简单 selection 与共享预算生成统一 preview response。

## Context

- 当前标准路径是 `outline -> ref -> read`，outline/find 已可在符合条件时返回 protocol-owned `auto_read`，且同一 invocation 复用 selected adapter document。
- Adapter 继续拥有单次 operation、ref 与分页；navigation/core 拥有通用组合和生命周期。
- Output 的 machine/readable 两条路径必须消费同一个 `ProtocolResponse`，不能只在 renderer 中追加 preview 业务事实。

## Goals / Non-Goals

Goals:

- 用确定性、低成本规则减少判断章节价值时的机械 read 往返。
- 在总预算内稳定表达已预览、未预览、失败和待继续状态。
- 保持组合逻辑跨 adapter 通用，并复用现有 read 语义。

Non-Goals:

- 不做摘要、relevance ranking、多 query 搜索或智能重要性判断。
- 不增加 adapter preview operation 或改变 adapter result shape。
- 不把 skim preview 与其它 operation-composition 候选合并成通用 framework。

## Decisions

### 1. Selection 使用 outline 顺序与显式预算

候选只来自当前 returned outline entries，按稳定 result order、非空 ref、批准的 count 和总预算选择；不读取未返回 entry，也不根据模型或环境顺序重排。

### 2. Preview 复用现有 read pipeline

Core/navigation 对 selected refs 执行现有 read，并保持 path、config、adapter selection、invocation-private document state、ref echo 和 pagination 规则。

### 3. Preview facts 进入统一 protocol result

Base outline 与每个 preview 的 content、status、diagnostic 和 continuation 形成一个 closed typed result；raw JSON 直接序列化，renderer 只做 presentation。

### 4. 单项失败保持局部

Outline 已成功后，某个 ref 不可读、预算不足或 read failure 形成对应 preview status；除非 base outline 本身失败，否则不替换 primary outcome。

## Risks / Trade-offs

- Preview 会增加同一调用的工作量；用总预算、有限 count 和 existing read pagination 控制。
- Composition result 可能膨胀 protocol；只加入调用方继续判断所需的最小 facts，并通过 schema/examples 约束。
- 与现有 auto-read 可能出现重复；复用相同生命周期和 raw/readable 原则，但保持独立 public behavior 和验收。
- CLI surface 与默认预算会形成长期 contract；必须在 production 修改前完成首个 gate。

## Open Questions

以下 change-local contract 选择由 Implementation 1.1–1.3 的用户或指定 CLI/protocol product owner 关闭；它们不阻塞开始执行本计划，但关闭前不得修改 owner、schema、测试预期或 production code：

1. 显式 CLI spelling、适用 output modes，以及与普通 outline、`--auto-read` 和 operation-applicable 参数的关系是什么？
2. Preview count、总预算的单位、默认值、显式覆盖面和耗尽规则是什么；每次 nested read 怎样获得有限 `limit`？
3. Typed composition result 的 closed variant、每个 preview status、read diagnostic projection 和 continuation shape 是什么？
