本 design 说明 `add-obvious-result-auto-read` 的实现边界：用确定性 core 编排覆盖 outline 和 find 的唯一明确结果自动 read；当前 change 只在 `openspec/changes/add-obvious-result-auto-read/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

Docnav 的基础阅读链路是 `outline -> ref -> read`，搜索链路常见形式是 `find -> ref -> read`。当 outline 只有一个 entry，或 find 只有一个 match，调用方仍需手动复制 ref 再调用 read。该场景不需要模型判断，也不需要 adapter 增加语义；它只需要 core 在满足明确条件时复用现有 read。

## Goals / Non-Goals

**Goals:**

- 为 outline 和 find 的单候选结果提供确定性自动 read 组合。
- 把组合逻辑放在 core 或可复用 shared helper，保持 adapter 只处理单次 operation。
- 在 typed composition result 中清楚表达 base result、auto-read 内容、跳过原因、read 失败和 continuation。
- 让 `protocol-json` 与 `readable-view` 消费同一个 `ProtocolResponse`，并在实现前验证新的 machine contract。

**Non-Goals:**

- 不新增 adapter-level operation。
- 不解析、生成或推断 adapter-owned ref grammar。
- 不在多个候选中排序、猜测最相关结果或理解用户意图。
- 不改变普通 `read`、`outline`、`find` 的 adapter result shape。

## Decisions

### Decision 1: 覆盖 outline 和 find，而不是 outline-only

本 change 使用 `add-obvious-result-auto-read` 命名，避免把范围限制为 fast outline。outline 单 entry 和 find 单 match 的机械步骤相同：都已经有一个 adapter-owned ref，且下一步自然是 read。

替代方案是只做 outline，但这会漏掉 find 的同类低歧义场景，并让后续再补 find 时重复设计输出和失败边界。

### Decision 2: obvious 的第一版只等于“唯一可读候选”

第一版 obvious 判定只接受零智能规则：base operation 成功、当前结果中恰好一个 item 带非空 ref、预算允许追加 read、调用方显式启用 composition control。零候选、多候选、无 ref、预算不足或分页未完成时不自动 read。

替代方案是使用 rank、label、kind 或 query 相似度判断“最明显”候选；这些规则更像搜索相关性或模型判断，当前阶段不进入实现。

### Decision 3: auto-read 是非致命展开

base outline/find 成功后，追加 read 的失败不应抹掉 base result。read 失败应作为 auto-read expansion status 进入 typed composition result，并保留用户可继续手动 read 或修正输入的上下文。

替代方案是让追加 read 失败导致整个 command 失败；这会把 convenience 行为变成比基础链路更脆弱的入口。

### Decision 4: auto-read facts 进入统一 ProtocolResponse

Auto-read selection、追加 read 和 expansion status 都是 core-owned 业务语义，不能只存在于 renderer。第一版必须定义 typed composition result，包含 base outline/find facts、auto-read content、status 和 continuation；`protocol-json` 直接暴露这些事实，内置 renderer 从同一 result 生成 `readable-view`。

Adapter-owned outline/find/read result 保持不变；core 在组合边界构造新的或扩展后的 operation result。该 machine contract 的 schema、examples、pagination 和 compatibility 审计是实现前置条件。

## Risks / Trade-offs

- [Risk] 单候选自动 read 可能让用户误以为 outline/find 语义改变。-> Mitigation: 将行为记录为显式 composition surface，并在 typed result 中同时保留 base result 与 auto-read 状态。
- [Risk] 追加 read 消耗预算导致输出不可预测。-> Mitigation: 使用总预算检查；预算不足时不 read，并输出稳定 skipped reason。
- [Risk] read 失败处理不清会破坏错误模型。-> Mitigation: base operation 成功后追加 read 失败作为 expansion status，不改写 primary diagnostic；base operation 本身失败仍走原错误投影。
- [Risk] machine result 扩大 compatibility surface。-> Mitigation: 采用显式 composition control、独立 typed result 和 schema/example validation，不改变未启用 composition 的基础 result。

## Migration Plan

该 change 是 additive；默认不要求 adapter 迁移。实现阶段应先更新 docs/spec/schema/examples，再实现 core composition 编排，最后补充 outline/find 单候选、多候选、预算不足、read 失败，以及两条 output path 对同一 composition facts 的投影测试。

## Open Questions

无未回答开放问题，可以进入实现前审计。实现阶段仍需在 CLI owner 文档中定稿显式 surface 的具体拼写。
