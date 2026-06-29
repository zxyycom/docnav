本 design 记录 Docnav raw protocol 字段结构化的探索框架；当前只在 `openspec/changes/explore-structured-protocol-fields/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

Docnav 同时服务机器稳定协议和 readable 输出。Raw protocol 应尽量承载结构化字段；readable 输出可以基于这些字段重新聚合、排序和格式化。Adapter 仍拥有 ref、导航策略、limit 解释、cost 计算和格式专属 metadata。

## Goals / Non-Goals

**Goals:**

- 审计当前 protocol 中被字符串承载的结构化语义。
- 确认哪些字段属于 protocol-owned、adapter-owned、core-owned 或 readable-only。
- 为 `limit`、`cost` 和导航结果字段提出候选 vNext shape。
- 给后续实现 change 提供拆分边界和兼容策略。

**Non-Goals:**

- 不在本 change 中实现 schema、代码或 fixture 迁移。
- 不要求所有 adapter 暴露同样的 metadata。
- 不让 readable 输出反向决定 raw protocol 字段结构。
- 不改变 ref 的 opaque contract。

## Decisions

### Decision 1: Raw protocol 优先结构化

当字段承载机器可理解语义时，raw protocol 应优先使用对象、数组、枚举或数值字段表达。Readable 输出可以从这些字段派生更适合人读的字符串或 block。

### Decision 2: Adapter-owned 语义仍由 adapter 解释

结构化不等于 core 接管语义。`limit` 的单位、cost 的计算、entry 的导航含义和 format-specific metadata 仍由 adapter 拥有，core 只校验共享 shape 并原样映射。

### Decision 3: 先探索，再拆实现 change

本 change 只产出字段审计、候选 shape、兼容策略和后续 change 切分。具体字段迁移在用户确认后单独进入实现型 change。

## Open Questions

- `limit_chars` 到 `limit` 是否需要双字段兼容期？
- `cost` 数组元素首期是否只包含 `value` 和 `unit`？
- outline/find 是否需要为 label、summary、location、cost 或 rank 提供共享结构？
- info result 是否需要结构化 document/adapter metadata？
