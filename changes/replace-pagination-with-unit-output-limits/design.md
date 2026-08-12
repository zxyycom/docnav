# Design

设计把输出安全建模为统一、带单位的 request budget，并将数字分页、下一页状态和 adapter-private budget units 从 public flow 中移除。

## Context

- Current protocol 为 outline、read 和 find 提供正整数 `limit` 与 `page`，结果通过可选 page 表示是否继续。
- Current adapter contract 让各格式解释 limit；Markdown 与 JSON 的主要路径按 Unicode 字符预算分页，而 public cost 还报告 lines、bytes 和 tokens。
- Authority boundary — `.change-plan.json` 拥有本 Change 的 lifecycle；本 design 只拥有 change-local Target。稳定 owner、schema、examples 和当前实现继续定义 Current，直到实现证据成立并完成 owner 同步。
- Long-term direction — [用带单位的输出上限替代分页](../../docs/decisions/product-direction/replace-pagination-with-unit-output-limits.md)保存已经确认但尚未成为 Current 的产品方向。
- This Change owns — public request/result、CLI、configuration、schema、adapter input migration，以及现有 selection-scoped cost 与共同 output cost 的协议分界。
- Runtime dependency — [introduce-budgeted-output-window](../introduce-budgeted-output-window/design.md)拥有 typed result 上的预算执行机制。
- Calculator dependency — [adopt-low-constant-reference-tokenizer](../adopt-low-constant-reference-tokenizer/design.md)拥有 token unit 的 calculator backend 和资源证据。
- Separate migration — [integrate-fast-read-budget-probing](../integrate-fast-read-budget-probing/design.md)拥有 fast-read threshold selector；本 Change 不把 fast-read threshold 合并为最终 output limit。

## Goals / Non-Goals

Goals:

- 用一个显式 cost unit 和正整数 value 定义每次有界输出。
- 删除 page、next-page、continuation 和 pagination-enabled normalization。
- 提供显式无界请求，并让所有结果保持合法、可验证的完整性状态。

Non-Goals:

- 不在本 Change 选择 token calculator 实现。
- 不在 serializer 上按最终 JSON/readable 字节做产品预算。
- 不保留提高 page 后继续读取剩余结果的兼容层。

## Decisions

### 1. Normalized limit 始终携带单位

Public machine input 使用 `{ unit, value }`；CLI 可以提供紧凑语法，但 navigation 构造 adapter input 前必须形成显式单位和值。相同 unit 在所有 operation 和 adapter 中使用同一个 CostCalculator 语义。

### 2. Limited 与 unbounded 是互斥状态

内部 request 使用 `Limited { unit, value } | Unbounded`，避免同时保存会互相冲突的 limit 和 ignore flag。CLI `--ignore-limit` 只形成 `Unbounded`，不能与显式 `--limit` 同时出现。

### 3. 删除公开继续位置

请求和结果移除 page、next-page 与 continuation。预算先耗尽时结果标记 incomplete；调用方只能缩小 ref、提高 limit 或重新执行 unbounded 请求。

### 4. Output budget 与 navigation strategy 分离

`ignore-limit` 只解除最终结果的普通输出预算，不自动选择 unstructured full read，也不改变 fast-read threshold。

### 5. Common output cost 只描述实际接纳结果

共同 OutputReport 的 cost 描述被最终结果接纳的预算字段。现有分页前完整 selection cost 不再充当 output cost；若产品仍需暴露完整 selection size，必须使用独立命名和契约，不能参与本次 limit accounting。

## Risks / Trade-offs

- 这是 protocol、CLI、configuration、schema、examples、adapter inputs 和测试的 breaking change，需要一次完整迁移，不能静默接受旧 page。
- 没有 continuation 后，预算外内容不能从上次截止位置续读；大型单一 selection 需要更大 limit、unbounded 请求或更具体 ref。
- 项目当前“有限、可继续”不变量与目标方向不一致；只有实现和验证完成后才能把稳定 owner 改写为 Current。
- 若原子语义结果无法在 limit 内合法表达，必须选择明确的 incomplete 或错误行为，不能偷偷突破预算。

## Open Questions

以下问题仍是 draft contract 缺口；在进入 Plan、派生 tasks 或把 Target 写入稳定 owner 前必须关闭：

1. 默认 limit 的 unit 和 value 分别是什么，是否允许用户选择 lines 或 bytes？
2. Complete、limited 和实际 output cost 放在 success envelope 还是 operation result 的公共 metadata 中？
3. 第一版是否直接移除旧输入，还是提供一个明确版本边界但不保持双重运行语义？
4. 单个必需原子结果超过预算时返回空 incomplete success 还是专用错误？
5. `ignore-limit` 是否仍受独立、不可绕过的 transport emergency ceiling？
