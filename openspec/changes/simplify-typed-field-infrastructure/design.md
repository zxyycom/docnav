## Context

核心句：本 change 通过“审计一个、实施一个、验证一个”的状态转移逐步收缩 typed-field 维护面。

`type-field-maintenance-report.md` 是方案证据；本文只拥有责任边界、slice 流程和回滚决策。当前 artifacts 未审核，仅存在于本 change 目录，不影响主规范。

## Goals / Non-Goals

**Goals:**

- 为每次 typed-field 优化提供相同的消费者审计、行为基线、验证和回滚门槛。
- 一次只激活一个 bounded slice，使其可以独立评审和回退。
- 保留 report 已证明有价值的 validation、selection、precedence 与 provenance 边界。

**Non-Goals:**

- 全面重组、多个候选的批量删除和新输入能力不属于同一 slice。
- Public CLI、config、protocol、schema、diagnostic 和 output behavior 保持不变。

## Decisions

### Decision 1: 共享层按稳定语义划界

Typed-fields 保留多个生产 owner 共同使用的 identity、validation、projection 和 failure facts；CLI/config resolution 保留 precedence、presence、fallback、validation 和 provenance。Source-specific extraction、adapter behavior 与 public mapping 由现有 owner 负责。

### Decision 2: 一次只激活一个 slice

每个候选先核对 production callers、owner contract、tests 和 downstream packages。一个 slice 只处理一个候选，完成验证与回滚检查后才能选择下一项。

首个 slice 从无生产消费者、不改变 public behavior、可独立回滚的叶子维护面中选择。仓库搜索用于确认调用关系，owner tests 和 downstream verification 用于证明行为保持。

### Decision 3: Supersede 关系在 checkpoint 收口

`derive-document-cli-options-from-fields` 暂时保留为历史计划。本 change 首个 slice 通过后，再单独决定标记、归档或保留方式；两套 deltas 不并行实施。

## Risks / Trade-offs

- [Risk] 消费者清单遗漏契约或动态调用。→ 同时检查 Cargo dependency、生产调用、owner docs 和行为测试。
- [Risk] Change 演变为隐含 backlog。→ 每个 checkpoint 只允许关闭 change，或显式追加一个下一 slice。

## Migration Plan

1. 审计候选并选择一个首个 slice。
2. 记录保护行为、基线、downstream 和回滚边界。
3. 实施最小改动，运行 owner tests 与 workspace verification。
4. 在 checkpoint 关闭 change，或显式追加一个下一 slice。

回滚只回退当前 slice；不引入 runtime fallback 或 feature flag。

## Open Questions

- 首个 implementation slice 尚未选择。阻塞审计完成选择并更新本节前，不进入实现。
