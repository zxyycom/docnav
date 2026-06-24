本 change 目标是让配置命中的非结构化文档在 `outline` 时直接返回全文内容，不再要求模型先取得 ref 再调用 read；它只在 `openspec/changes/outline-unstructured-full-read/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

有些 Markdown 或文本型文档不适合通过 heading、outline entry 或局部 ref 阅读；在这些文档上强制执行 `outline -> ref -> read` 会让调用方在不知道内容的情况下得到一个无意义 ref，并额外消耗一次模型调用。

现在需要一个显式、可配置的非结构化路径：当配置声明某个文档不做结构化导航时，第一次 `outline` 就应返回原文全文，并在 CLI/readable 输出中说明这是自动全文读取。

## What Changes

- `outline` 增加配置触发的非结构化全文结果形态：成功结果直接包含全文 `content`、`content_type`、`cost` 和非结构化说明，不包含 `entries`、`ref`、`page` 或 continuation。
- **BREAKING (opt-in)**: 对命中非结构化配置的文档，`outline` 不再返回当前固定的 `entries[]` 与 `page` 字段；未命中配置的文档保持既有结构化 outline 契约。
- Docnav 在 `outline` 入口消费生效配置中的非结构化文档策略，命中后短路为原文全文读取，而不是构造 `doc:full` 或其它 adapter ref；具体配置文件、格式和合并方式不由本 change 定义。
- Markdown adapter/direct CLI 对等支持非结构化 outline 行为，确保各入口在同一生效配置语义下输出一致 readable/protocol 结果。
- 非目标：不改变普通结构化文档的 `outline -> ref -> read` 主流程；不把 `ignore` 语义解释为跳过文件；不要求其它 adapter 复用 Markdown 的 `doc:full` 或 ref grammar；不为非结构化全文读取引入分页。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `core-cli`: 增加 core CLI 对生效非结构化 outline 配置的消费方式和输出映射。
- `adapter-protocol`: 扩展共享 outline success result，使其支持结构化 entries 与非结构化全文内容两个形态。
- `markdown-navigation`: 增加 Markdown adapter/direct CLI 的非结构化 outline 行为、配置语义和测试边界。
- `readable-view-output`: 让 outline readable-view 在非结构化结果形态下可以使用 `/content` block，并保持结构化 outline 无 block 的既有行为。

## Impact

- 受影响代码：outline execution pipeline、生效配置策略判断、shared protocol/result types、readable payload/output renderer 配置、Markdown direct CLI 和 Markdown adapter operation handler。
- 不受影响范围：普通结构化 outline 默认行为、read/find/info 语义、ref opaque pass-through 契约、Markdown heading ref grammar 和其它 adapter 的私有导航策略。
