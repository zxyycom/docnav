本 change 定义 Markdown adapter-owned document head 可读区域，让 structured outline 暴露首个有效 heading 前的阅读上下文。

## Why

Markdown 文档在首个有效 heading 前常放置 YAML frontmatter、摘要、序言、版权、状态说明或其它阅读前提。这些内容不属于任何 heading section，但经常是选择后续章节前必须先读取的上下文。

Document head 作为一个合并的 adapter-owned readable region，可以覆盖 frontmatter 和普通前导正文，并继续使用 `outline -> ref -> read` 的结构化导航流程。

## What Changes

- `docnav-markdown` 增加 document head 区域：从文档开头到第一个有效 Markdown heading 起点之前的原文区域。
- 当 document head 包含非空、非纯空白内容，并且当前 structured outline 至少有一个可见 heading entry 时，outline 在 heading entries 前返回 `HEAD:leading` entry。
- 读取 `HEAD:leading` 返回整个 document head 原文，`content_type` 为 `text/markdown`；如果该区域包含 YAML frontmatter delimiter，content 保留 delimiter。
- Find 命中 document head 且当前 structured outline 至少有一个可见 heading entry 时，match ref 返回 `HEAD:leading`，保证可继续通过 read 获取命中文本。
- 当前 outline 参数下没有可见 heading 时继续使用 `doc:full` fallback；heading outline、heading read 和 `doc:full` read 行为保持既有语义。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `markdown-adapter`: 增加 Markdown document head 区域、`HEAD:leading` adapter-owned ref、outline/find 映射和 read roundtrip 行为。

## Impact

- 影响 `docnav-markdown` parser、outline、find、read、Markdown smoke/adapter tests、testing case ledger 和 `docs/adapters/markdown.md`。
- 不改变其它 adapter 的 ref grammar；不要求共享层理解 Markdown frontmatter 或 document head 语义。
- 不新增 outline 顶层字段；protocol/readable output 继续使用既有 structured entries 和 read content block 形态，raw item 使用 `label`、`kind`、`location`、`metadata` 等既有 entry facts。
- 不改变配置命中整篇全文读取的 `outline-unstructured-full-read` 目标；document head 是普通 structured Markdown outline 的额外可读区域。
