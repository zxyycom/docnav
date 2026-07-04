本 design 只记录 `outline-unstructured-full-read` 的关键取舍。字段级 MUST、场景和测试矩阵由本 change 下的 capability spec delta 与 tasks 承接。

## Context

Docnav 的默认文档阅读流程是 `outline -> ref -> read`。这适合结构化文档，但对明确不需要结构化导航的文档，或 selected adapter 能证明全文成本很低的文档，会多消耗一次调用。

本 change 增加 opt-in 的 navigation execution policy：navigation input resolution 产出 `outline_mode = "unstructured_full"` 后，`outline` 在 selected adapter 的正常 outline handler 前直接返回全文内容。

## Decisions

1. **`outline_mode` 由 navigation 拥有。**
   `structured` 是默认值；`unstructured_full` 是标准 outline execution policy，不是 adapter 私有 outline option，也不是 ref policy。Resolution 顺序为 path rules > adapter-scoped cost thresholds > built-in default。Public CLI 不新增 outline-mode override flag。

2. **`OutlineResult` 使用带 `kind` 的 union。**
   Structured branch 增加 `kind: "structured"`；unstructured branch 使用 `kind: "unstructured"`、`reason`、`content`、`content_type` 和稳定 `cost`。这是一项 shape-level breaking change，但避免调用方同时依赖 shape probing 和 discriminator probing。

3. **非结构化全文结果不分页、不裁剪。**
   `unstructured_full` 命中后成功结果不返回 entries、ref、page 或 continuation。Cost threshold 只决定是否进入全文读取，不成为输出上限。

4. **Cost threshold 先筛选再按需计算。**
   Cost 计算可能昂贵，因此 navigation 必须先完成 adapter selection，只保留 selected adapter 的 candidate thresholds，再按 `unit` 合并为最小有效阈值。没有 candidate thresholds 时不调用 adapter cost measurement hook；有 candidate thresholds 时只请求有效 units。具体比较语义归 `navigation-input-resolution` delta。

5. **Adapter 只通过非结构化全文 hook set 补充格式事实。**
   Adapter 可以声明 `unstructured_full_read`、full-read cost measurement hook/declaration 和结果事实补充 hook。Hook set 只服务非结构化全文路径，可以补 `content`、`content_type`、`Cost.measurements[]` 或其它稳定 result facts；不得生成 entries、ref、page、continuation 或 readable-only wrapper。未声明 content hook 时，navigation 使用默认 UTF-8 原文读取 fallback。

6. **Readable output 只投影成功 payload。**
   `unstructured_full` 是成功执行策略，不通过 diagnostic wrapper 或 Markdown `doc:full` ref 表达。Readable-view 使用 `/content` block，cost 展示只能从 stable result facts 派生。

## Risks

- Existing outline consumers must handle `kind`; structured outline behavior remains the same apart from the discriminator.
- Full-read output can be large by design; selectors and adapter thresholds are the opt-in guard.
- Cost threshold hooks must be callable only after selected-adapter candidate filtering, otherwise small-document optimization can become a performance regression.
- The default UTF-8 fallback must stay shallow: file read, UTF-8 decode and basic `content_type`; format-specific facts belong to adapter hooks.

## Implementation Order

1. Update owner docs, schema/examples and result types for the new outline union.
2. Implement navigation resolution for path rules, cost threshold candidate filtering and pre-dispatch execution.
3. Add adapter hook set support, default UTF-8 fallback and readable-view `/content` rendering.
4. Add focused tests for structured compatibility, path-triggered full read, cost-triggered full read, hook/fallback behavior and output mapping.
