# Design

设计把 fast-read 建模为同一预算引擎的非提交 probe session，并在 adapter document lifecycle 内保留 prepared view 复用。

## Context

- Current navigation 按 configured threshold units 调用 selected adapter 的 `measure_unstructured_full_read_cost`，命中后再调用 full-read content hook。
- Current Markdown hook 先计算 lines、bytes 和 tokens 的完整 cost，再过滤 requested units；full-read result 随后再次形成完整 cost。
- Authority boundary — `.change-plan.json` 拥有本 Change 的 lifecycle；本 design 只拥有 fast-read admission Target。Current threshold、adapter hook 和 outline-mode behavior 仍由稳定 owner 与 source 定义。
- Long-term direction — [在标记的语义字段上集中执行输出预算](../../docs/decisions/product-direction/centralize-output-budgeting-over-marked-semantic-fields.md)要求 fast-read 与最终输出复用同一个 CostCalculator。
- Token direction — [保留当前 reference tokenizer，直到可靠替代已具备](../../docs/decisions/product-direction/retain-current-reference-tokenizer-until-qualified-replacement.md)固定现有单一 token 语义，同时允许 fast-read 为自身 workload 设置独立的 latency 与 early-stop 准入门。
- Runtime dependency — 本 Change 只消费 [introduce-budgeted-output-window](../introduce-budgeted-output-window/design.md)的 probe abstraction 和共同 CostCalculator contract。
- Separate contract — [replace-pagination-with-unit-output-limits](../replace-pagination-with-unit-output-limits/design.md)拥有最终 output limit；fast-read threshold 是 navigation admission budget，不是 output budget。

## Goals / Non-Goals

Goals:

- 只处理 selected-adapter threshold 实际请求的 units。
- 在足以证明 threshold exceeded 后停止 cost work。
- 保持同一 invocation-private adapter document/prepared view，并复用成功 probe 的测量。
- 明确区分 admission evidence 和最终 public output cost。

Non-Goals:

- 不让 `ignore-limit` 自动强制 fast read。
- 不把 fast-read threshold 当成最终 output limit。
- 不在 navigation 中解析 format-private document content。
- 不选择 token backend，也不把 fast-read 的局部性能门提升为 public output-limit 的门禁。

## Decisions

### 1. Probe 不直接构造或写出 response

Fast-read selector 创建独立 probe session。它只返回 `Fits(measurement)` 或 `Exceeded`；navigation 在 probe 完成后才选择 unstructured full read 或 structured outline。

### 2. 输入结束与预算结束决定 outcome

Calculator 消费候选的标记内容。内容在 threshold 以内先耗尽时返回 `Fits(measurement)`；threshold 先耗尽且仍有未消费内容时返回 `Exceeded`，此后 calculator 不继续扫描剩余输入。

### 3. Admission accounting 与 output accounting 分离

失败 probe 的内部 measurement 不出现在 structured fallback result。成功 probe 的完整 measurement 可以作为 full-read candidate sidecar 传给最终 OutputWindow，但只有最终接纳字段形成 public output cost。

### 4. Adapter 提供候选语义，不拥有 calculator

Adapter 继续拥有 prepared document view 和 full-read content selection；shared navigation/output budget layer 拥有 unit dispatch、threshold comparison 和 CostCalculator。现有 measurement hook 在迁移完成后删除或缩减为不重复 calculator 的候选投影能力。

## Risks / Trade-offs

- 为 probe 暴露 full-read candidate 不能把 document parsing、format selection 或 cross-invocation cache 移入 navigation。
- 成功 probe 与最终 output limit 可能使用不同 unit 或 value；测量复用必须以 token 语义、文本身份、unit 和 scope 一致为前提，不能依赖 backend identity 偶然相同。
- 多 unit thresholds 可能需要多个 unit calculator session 或不同提前停止点，容易重新产生先算全部再过滤。
- Current backend 或 correctness-first wrapper 可能满足结果正确性却不满足 fast-read 的提前停止目标；这是本 Change 的局部 admission 阻塞，不影响 public output-limit rollout。
- 默认 UTF-8 full-read fallback 与 adapter content hook 的 ownership 不同，需要保持当前错误和 lifecycle 边界。

## Open Questions

以下问题仍是 adapter ownership 和复用安全性的 draft 缺口；关闭前不能并存新旧两套 full-read cost owner：

1. Fast-read thresholds 是否继续允许多个 units，还是规范化为单一带单位 budget？
2. Candidate content 通过 borrowed text、opaque probe handle 还是 adapter-owned callback 暴露，才能保持 ownership 与提前停止？
3. 成功 measurement 以何种 sidecar 身份传给最终 OutputWindow，避免不安全的内容等价假设？
4. 现有 full-read cost capability 和 measurement hook 如何分阶段退役而不维护双路径？
5. Path-rule 强制 unstructured full read 是否跳过 admission probe，只执行最终 output budget？
