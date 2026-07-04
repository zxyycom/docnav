本 delta 定义 readable 输出层如何渲染非结构化 outline 全文结果。

## ADDED Requirements

### Requirement: readable 输出支持非结构化 outline 全文结果
Readable payload MUST 支持 outline 的结构化 entries 形态和 navigation pre-dispatch 产出的非结构化全文形态。普通 outline readable-view 仍不使用 block；非结构化 outline readable-view MUST 使用 `/content` block 承载全文内容，并在 header 中保留 `kind: "unstructured"`、稳定 `cost` 事实和稳定原因字段或等价可读说明。Reason semantics MUST distinguish `path_rule` and `cost_threshold` triggers. Adapter hook set MAY provide missing `Cost.measurements[]` as result facts before output rendering, but MUST NOT provide readable-only cost summary strings or readable wrapper fields.

#### Scenario: 普通 outline readable-view 仍无 block
- **WHEN** outline result 为结构化 entries 形态
- **THEN** readable-view header 包含 `kind: "structured"`、entries 和 page
- **THEN** stdout 不包含 `/content` block

#### Scenario: 非结构化 outline readable-view 使用 content block
- **WHEN** outline result 为 resolution-triggered 非结构化全文形态
- **THEN** readable-view header 包含 content 的 `$block` 引用
- **THEN** `/content` block payload 等于 readable-json 中的 content 字符串
- **THEN** header 不包含 entries、ref、page 或 continuation
- **THEN** header 包含 `kind: "unstructured"`、cost 事实和稳定原因字段或等价可读说明

#### Scenario: 非结构化 outline readable-json 与 readable-view 同源
- **WHEN** 同一个非结构化 outline result 分别渲染为 readable-json 和 readable-view
- **THEN** 两者的 content、content_type、cost 和 reason 语义一致
- **THEN** readable-view 的 `/content` block payload 等于 readable-json 的 content 字符串

#### Scenario: cost 展示只从稳定 result facts 派生
- **WHEN** 非结构化 outline result 的 adapter hook set 提供 `Cost.measurements[]`
- **THEN** readable output MAY derive a cost display from those measurements
- **WHEN** `Cost.measurements[]` is empty
- **THEN** readable output preserves the stable empty cost fact instead of inventing adapter-specific measurements
