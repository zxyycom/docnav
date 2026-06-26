本 design 只在 `openspec/changes/markdown-document-head-outline-mode/` 下形成未审核临时文档，说明 Markdown document head outline mode 的范围、refs、配置和取舍。

## Context

当前 Markdown heading section 从 heading 自身开始，首个有效 heading 前的内容不属于任何 heading ref。`find` 命中这一区域时当前实现只能退回 `doc:full`，这能保证 read 包含命中文本，但粒度过粗。

已有 `markdown-frontmatter-outline-mode` change 聚焦 YAML frontmatter metadata；它无法覆盖 frontmatter 后的普通前导正文，也会引入 outline 顶层 metadata 字段和 inline 分页复杂度。本 change 改用 adapter-owned refs 暴露 document head，保持普通 structured outline shape。

## Goals / Non-Goals

**Goals:**

- 将首个有效 Markdown heading 前的内容建模为 document head 可读区域。
- 默认用一个 `HEAD:leading` entry 暴露 document head，让调用方一次 read 获取 frontmatter 与前导正文原文。
- 在需要精细消费时支持 `split` 模式，分别暴露 recognized frontmatter block 与 preamble。
- 支持显式 frontmatter block policy，默认只承诺文档开头一个 YAML delimiter block，多 metadata block 必须显式启用。
- 保持 heading ref grammar、普通 heading read、`doc:full` fallback 和 shared ref opacity 不变。

**Non-Goals:**

- 不新增 outline 顶层 `frontmatter`、`metadata` 或 `document_head` 字段。
- 不解析 YAML 字段语义，不定义 metadata schema、排序、合并或规范化。
- 不把 document head 当作其它 adapter 必须复用的共享概念。
- 不改变 `outline-unstructured-full-read` 的整篇全文读取策略。
- 不在审计门禁完成前更新主规范、schema/example、测试或实现代码。

## Decisions

1. **document head 是可读区域，不是 metadata 字段。**

   Document head 定义为当前 Markdown 文档从开头到第一个有效 Markdown heading 起点之前的原文区域。默认 `combined` 模式返回一个 `HEAD:leading` outline entry；读取该 ref 返回整段原文，`content_type` 为 `text/markdown`。这保留 YAML delimiter、空行、注释、序言和其它前导文本，避免 adapter 猜测摘要或 metadata 语义。

   备选方案是把它作为 outline 顶层 metadata inline 返回。该方案会扩大 shared/readable 输出 shape，并且无法优雅承载普通 Markdown 前导正文，因此不采用。

2. **默认 `combined`，需要时再 `split`。**

   `document_head_outline_mode` 的合法值为 `combined`、`split`、`hidden`，默认 `combined`。`combined` 优先服务 agent 选择章节前“反正大概率需要看一下”的场景；`split` 给需要单独读取 YAML 或 preamble 的消费者使用；`hidden` 保留现有只看 heading outline 的行为。

   `split` 模式下，frontmatter ref 使用 line-based adapter ref，例如 `FM:L1`；preamble 使用 `P:preamble`。当没有对应区域时不返回该 entry。

3. **frontmatter dialect 显式化。**

   `frontmatter_block_policy` 默认 `opening_only`：只识别文件开头的一个 YAML delimiter block，payload 不包含起止 delimiter。`pandoc_metadata_blocks` 可作为显式模式支持多 metadata block；实现必须在 Markdown adapter 文档中写清识别边界，并用 fixture 覆盖多个 block、未闭合 block 和普通 horizontal rule 的边界。

   在 `combined` 模式下，frontmatter policy 只影响 display 摘要和 split 需要的内部分类；`HEAD:leading` read 始终返回 document head 原文。

4. **outline entry 只作为导航入口，内容通过 read 分页。**

   Outline 中的 `HEAD:leading`、`FM:L{line}` 和 `P:preamble` entries 只包含 ref/display。大段内容仍由 read 返回，并沿用 read 的 Unicode 字符预算、page 和 content block 输出规则。这样避免把 outline pagination 与大段头部内容绑在一起。

5. **document head 不破坏既有 `doc:full` fallback。**

   如果当前 outline 参数下没有可见 heading entry，Markdown outline 继续使用既有 `doc:full` fallback，而不是只返回 document head entry。Document head entry 只在普通 structured outline 中作为首个可选区域出现在 heading entries 之前。

6. **find 必须返回可继续 read 的区域 ref。**

   当 find 命中 document head 且 document head mode 暴露对应区域时，match ref 使用 `HEAD:leading` 或 split 下更精确的 `FM:L{line}` / `P:preamble`。当 mode 为 `hidden` 或当前 outline 走 `doc:full` fallback 时，find 保持可 read 的既有行为，必要时使用 `doc:full`。

## Risks / Trade-offs

- [Risk] `HEAD:leading` 包含 YAML delimiter，与 split frontmatter 的 delimiter-free payload 不同。Mitigation: 文档明确 `combined` 是原文区域读取，`split` 才是 typed region 读取。
- [Risk] `combined` 默认会让 outline entry 多一个非 heading entry。Mitigation: display 明确标注 `document head`，ref 前缀区别于 heading ref，且 `hidden` 可恢复旧 outline。
- [Risk] 多 metadata block 容易误判 horizontal rule。Mitigation: 默认 `opening_only`；多 block 只能显式启用并由测试覆盖方言边界。
- [Risk] 与现有 `markdown-frontmatter-outline-mode` change 重叠。Mitigation: 本 change 明确计划取代较窄 frontmatter 方案；进入实现前必须通过审计门禁确认只推进一个方案。

## Migration Plan

1. 先完成本 change 的阻塞级审计，确认它取代 `markdown-frontmatter-outline-mode` 或将后者关闭/归档为 superseded。
2. 审计通过后同步 `docs/adapters/markdown.md`、Markdown config schema/example 和测试计划。
3. 再实现 parser 区域识别、adapter-owned options、outline/find/read refs 和 smoke/unit tests。
4. 未配置用户默认获得 `combined` 行为；如需要完全保持旧 outline，可设置 `document_head_outline_mode: "hidden"`。

## Open Questions

- `pandoc_metadata_blocks` 是否首期实现，还是只在 schema 中预留但不暴露为 accepted value，需要审计时确认。
- `split` 模式下多个 frontmatter entries 的 display 是否需要包含行号、block 序号或 cost，进入实现前应在 Markdown adapter 文档中固定。
