本 change 只在 `openspec/changes/markdown-document-head-outline-mode/` 下形成未审核临时文档，目标是让 Markdown outline 以 adapter-owned document head 区域暴露 frontmatter 与首个 heading 前正文。

## Why

Markdown 文档在首个 heading 前常放置 YAML frontmatter、摘要、序言、版权、状态说明或其它阅读前提；这些内容不属于 heading section，却经常是选择后续章节前必须先看的上下文。

仅把 YAML frontmatter 作为特殊 metadata 暴露会漏掉紧随其后的前导正文；仅使用 `doc:full` 或非结构化全文读取又会丢失普通 `outline -> ref -> read` 的结构化导航价值。

## What Changes

- `docnav-markdown` 增加 adapter-owned document head 识别：document head 是从文档开头到第一个有效 Markdown heading 之前的区域，可包含 opening YAML frontmatter 和普通前导 Markdown 正文。
- Markdown outline 增加 `document_head_outline_mode` 配置，合法值为 `combined`、`split` 和 `hidden`，默认值为 `combined`。
- `combined` 模式下，outline 在 heading entries 前返回一个 `HEAD:leading` entry；读取该 ref 时返回整个 document head 原文，`content_type` 为 `text/markdown`，包含 frontmatter delimiter。
- `split` 模式下，outline 可分别返回 frontmatter ref 与 preamble ref：frontmatter ref 返回 delimiter 内部 YAML payload，`content_type` 为 `application/yaml`；preamble ref 返回 frontmatter 后到第一个 heading 前的 Markdown 原文，`content_type` 为 `text/markdown`。
- `hidden` 模式下，outline 不暴露 document head entry，heading outline、`doc:full` fallback 和普通 heading read 行为保持既有语义。
- Markdown adapter 明确 frontmatter dialect 策略：默认只识别文件开头一个 YAML delimiter block；多 metadata block 只能通过显式 adapter option 启用，避免把普通正文误判为 metadata。
- `document_head_outline_mode`、`frontmatter_block_policy` 和任何多 metadata block 开关都必须作为 Markdown adapter-owned native option sources 声明、校验和拒绝；未声明 public input 不得被共享层隐式 passthrough。
- 本 change 计划取代较窄的 `markdown-frontmatter-outline-mode` 方案进入实现；在本 change 审计完成前，不应同时实现两个重叠方案。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `markdown-navigation`: 增加 Markdown document head 区域、adapter-owned refs、outline 模式、frontmatter dialect 边界和 find-to-read 映射。

## Impact

- 影响 `docnav-markdown` parser、outline、find、read、direct CLI 配置与 adapter-owned option schema。
- 影响 Markdown config schema/example、Markdown smoke/adapter tests 和 `docs/adapters/markdown.md`。
- 不改变其它 adapter 的 ref grammar；不要求共享层理解 Markdown frontmatter、preamble 或多 metadata block 方言。
- 不新增 outline 顶层字段；protocol/readable output 继续使用既有 outline entries 和 read content block 形态。
- 不改变配置命中整篇全文读取的 `outline-unstructured-full-read` 目标；document head 是普通结构化 Markdown outline 的额外可读区域。
