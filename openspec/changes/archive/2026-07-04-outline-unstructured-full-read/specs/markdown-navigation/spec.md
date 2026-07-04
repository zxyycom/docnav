本 delta 定义 core `docnav outline` 对 Markdown 文件命中 navigation-level `outline_mode = "unstructured_full"` 时的非结构化全文读取覆盖，并保持 linked Markdown adapter 的正常 outline 行为不变。

## ADDED Requirements

### Requirement: Markdown 文件可以通过 navigation pre-dispatch 返回非结构化全文
Core-mediated Markdown outline MUST 支持 navigation 标准 `outline_mode = "unstructured_full"`。生效值为 `unstructured_full` 时，navigation MUST 在 linked Markdown adapter 正常 outline handler 之前返回整篇 Markdown 原文内容，并 MUST NOT 返回 heading entries、`doc:full` entry、ref、page 或 continuation。本 change 不把 `outline_mode` 定义为 Markdown adapter 私有 native option。

#### Scenario: Core CLI dispatch 到 Markdown linked adapter 后自动全文读取
- **WHEN** 调用方执行 `docnav outline docs/raw-note.md`
- **AND** navigation 解析出的标准 `outline_mode` 为 `unstructured_full`
- **THEN** 输出为非结构化 outline 结果
- **THEN** result 包含 `kind: "unstructured"`
- **THEN** content 等于整篇 Markdown 原文
- **THEN** content_type 为 `text/markdown`
- **THEN** 结果不包含 heading entries、`doc:full`、ref 或 page

#### Scenario: 默认 structured 时保留 doc:full fallback
- **WHEN** 当前 outline 参数过滤后没有可见 heading
- **AND** navigation 解析出的标准 `outline_mode` 为默认值 `structured`
- **THEN** Markdown outline result 包含 `kind: "structured"`
- **THEN** Markdown outline 仍返回 ref 为 `doc:full` 的单条 entry
- **THEN** 使用该 ref 执行 read 返回整篇 Markdown 文档

### Requirement: Markdown 非结构化 outline 必须被 smoke 覆盖
Markdown smoke 和 fixture corpus MUST 覆盖至少一个 `outline_mode = "unstructured_full"` 的非结构化 Markdown 文件，并验证 readable-json、readable-view 和 protocol-json 三种输出模式的非结构化 outline shape。Coverage MUST include at least one path selector and one adapter-scoped cost-threshold selector.

#### Scenario: Smoke 覆盖非结构化 outline shape
- **WHEN** smoke suite 对 `outline_mode = "unstructured_full"` 的 Markdown fixture 执行 outline
- **THEN** readable-json 和 protocol-json 结果不包含 entries、ref 或 page
- **THEN** readable-json 使用 documented outline success payload 分支
- **THEN** readable-view 使用 `/content` block 承载全文
- **THEN** 三种输出都包含稳定原因字段或等价可读说明
