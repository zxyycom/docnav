本 design 说明 `add-obvious-result-auto-read` 的实现边界：用确定性 core/readable 编排覆盖 outline 和 find 的唯一明确结果自动 read；当前 change 只在 `openspec/changes/add-obvious-result-auto-read/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

Docnav 的基础阅读链路是 `outline -> ref -> read`，搜索链路常见形式是 `find -> ref -> read`。当 outline 只有一个 entry，或 find 只有一个 match，调用方仍需手动复制 ref 再调用 read。该场景不需要模型判断，也不需要 adapter 增加语义；它只需要 core 在满足明确条件时复用现有 read。

## Goals / Non-Goals

**Goals:**

- 为 outline 和 find 的单候选结果提供确定性自动 read 组合。
- 把组合逻辑放在 core/readable 层或可复用 shared helper，保持 adapter 只处理单次 operation。
- 在 readable-view / readable-json 中清楚表达 base result、auto-read 内容、跳过原因、read 失败和 continuation。
- 保持 `protocol-json` 基础 operation result 不被组合内容污染，除非本 change 明确增加并验证新的机器契约。

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

第一版 obvious 判定只接受零智能规则：base operation 成功、当前结果中恰好一个 item 带非空 ref、预算允许追加 read、输出模式支持组合 readable payload。零候选、多候选、无 ref、预算不足或分页未完成时不自动 read。

替代方案是使用 rank、label、kind 或 query 相似度判断“最明显”候选；这些规则更像搜索相关性或模型判断，当前阶段不进入实现。

### Decision 3: auto-read 是非致命展开

base outline/find 成功后，追加 read 的失败不应抹掉 base result。read 失败应作为 auto-read expansion status 投影到 readable output，并保留用户可继续手动 read 或修正输入的上下文。

替代方案是让追加 read 失败导致整个 command 失败；这会把 convenience 行为变成比基础链路更脆弱的入口。

### Decision 4: protocol-json 默认不承载组合结果

`protocol-json` 现有 contract 将 operation identity 绑定到单一 result shape。把 read result 混入 outline/find protocol result 会扩大 raw protocol contract；第一版应把组合限定在 readable-view / readable-json，或在 protocol-json 下明确拒绝组合控制。

替代方案是新增 composite protocol result，但这需要额外 schema、examples、pagination 和 compatibility 审计，不适合这个窄 change 的第一步。

## Risks / Trade-offs

- [Risk] 单候选自动 read 可能让用户误以为 outline/find 语义改变。-> Mitigation: 将行为记录为显式 composition surface，并在 readable output 中同时保留 base result 与 auto-read 状态。
- [Risk] 追加 read 消耗预算导致输出不可预测。-> Mitigation: 使用总预算检查；预算不足时不 read，并输出稳定 skipped reason。
- [Risk] read 失败处理不清会破坏错误模型。-> Mitigation: base operation 成功后追加 read 失败作为 expansion status，不改写 primary diagnostic；base operation 本身失败仍走原错误投影。
- [Risk] 后续想支持 protocol-json 时返工。-> Mitigation: 第一版明确 protocol-json 不混入组合内容；需要 raw machine contract 时另行扩展本 change 的 spec/schema。

## Migration Plan

该 change 是 additive；默认不要求 adapter 迁移。实现阶段应先更新 docs/spec/schema/examples，再实现 core/readable 编排，最后补充 outline/find 单候选、多候选、预算不足、read 失败和 protocol-json 组合控制测试。

## Open Questions

无未回答开放问题，可以进入实现前审计。实现阶段仍需在 CLI owner 文档中定稿显式 surface 的具体拼写。
