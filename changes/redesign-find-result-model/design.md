# Design

设计把 result identity 与产生它所需的 bounded work 作为同一个人工批准 packet；获批前只维护 Current baseline 和候选比较，获批后再形成唯一 Target。

## Context

- Current shared result 是 `matches: Entry[]`；Markdown 按 source occurrence 产生项，同一 ref 可以重复。
- `Entry` 当前涉及 `ref`、`label`、`kind`、`location`、`summary`、`excerpt`、`rank`、`cost` 和 `metadata`，字段保留与语义必须逐项决定。
- Current outline/find 可以根据当前 returned result 的 unique exact ref 形成 nested auto-read；改变 page unit 或 completeness 会影响其证明范围。
- JSON 使用 source-oriented search 和 canonical opaque ref；是否需要 JSON owner delta 只能在共享模型获批后判断。

## Goals / Non-Goals

Goals:

- 选择一个跨 adapter 清楚、可分页、可继续且有 bounded proof 的 find model。
- 让 wire identity、evidence、multiplicity、ordering、page 和 auto-read 语义可独立验证。
- 明确兼容、migration 和 rollback，而不是让实现形状暗中决定 contract。

Non-Goals:

- 不由 agent、benchmark 或实现便利替 product owner 选择模型。
- 不选择 token calculator，不修改 JSON 专用 presentation。
- 不把 query-global total/group/rank 承诺建立在没有 full traversal 或 authoritative index proof 的实现上。

## Decisions

### 1. 产品模型与 work budget 一次批准

Logical unit、字段和可观察 completeness 会决定必须扫描/保留的工作；二者必须在同一 packet 中批准，不能先定 shape 再补性能边界。

### 2. 一个共享模型或显式 discriminator

所有 adapter 遵守同一获批 model；格式可以填充获批 metadata/evidence，但不能私下选择 occurrence/node/group 变体。

### 3. Current behavior 在门禁前保持权威

Planning artifacts、agent recommendation 和候选 benchmark 都不改变 Current occurrence contract。批准结果进入本 Change 的 design Decisions 和 exact tasks 后形成 change-local Target；稳定 owner 仍只描述 Current。实施前可以审计并登记 owner delta，但只有实现与行为验证成立后，才把实际成立的 contract 同步到稳定 owner、schema 和 examples。

### 4. Bounded proof 与所选事实匹配

Source-order occurrence 或 first-occurrence distinct-ref 可以使用单调 traversal、seen-set 和有限 lookahead；query-global uniqueness、exact total、complete grouping 或 global rank 只有在 full traversal/authoritative index 及其预算获批时才能承诺。

### 5. Auto-read completeness 必须显式

获批 packet 决定 unique-ref 是 current-page 还是 query-global、哪些 units 提供 ref、partial/incomplete 是否抑制 auto-read，以及现有 `reason: "unique_ref"` 是否足够。

## Risks / Trade-offs

- Grouping/distinct 可能要求无界 seen-set 或 full scan；packet 必须选择 retained/spill budget 和 exhaustion behavior。
- Breaking wire changes会影响所有 adapters、renderers 和 consumers；必须有版本/迁移/rollback 方案。
- Representative evidence 可能被误作全局 completeness；schema 和 tests 必须区分 exact、partial、lower-bound 或 absent。
- JSON natural representation 可能诱导 shared variant；除非批准 public discriminator，否则 shared model 保持一致。

## Open Questions

以下问题组成一个不可拆分的人工批准 packet。Implementation 1.1 只能由用户或其指定 product/architecture owner 关闭；1.2–1.4 负责把批准结果转成单一 Target 并完成阻断审计。1.4 通过前，owner、schema、测试和 production 修改全部被阻塞：

1. Logical unit 是 occurrence、distinct exact-ref/node 还是 grouped-by-ref；是否需要 public variant discriminator？
2. Rust/wire type 与 top-level field 是什么？
3. 一个 unit 的 identity 是什么；exact opaque ref 不足时暴露哪个 adapter-owned fact？
4. 九个 Current `Entry` 字段分别 preserve/delete/replace，精确含义、requiredness 与兼容策略是什么？
5. Multiplicity 是 absent、page-local、lower-bound 还是 exact/query-global，如何表达 completeness？
6. Deterministic ordering 是 source occurrence、first occurrence、node order、ref lexical、rank 还是其它 adapter-owned rule？
7. Page unit 是什么；group 是否完整、可否跨页、nested evidence 是否单独 continuation？
8. Lookahead 怎样证明 continuation，page `k` 怎样重放，unfinished group 怎样处理？
9. Auto-read 是 current-page 还是 query-global，partial/incomplete 怎样影响 eligibility？
10. First/later page 可扫描多少 bytes/scalars/nodes/occurrences，哪些事实要求 full traversal/index？
11. 可保留多少 refs/occurrences/excerpts/counters/offsets/spill bytes，cleanup/failure 如何定义？
12. Budget exhaustion 返回 partial page、continuation、diagnostic 还是 failure；怎样保证单调推进？
13. Compatibility/version/migration/rollback 采用什么策略？
14. 获批 shared model 是否要求完整 JSON owner delta，以及怎样与独立 renderer handoff 分离？
