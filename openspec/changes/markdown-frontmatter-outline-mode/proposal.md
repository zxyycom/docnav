本 change 目标是让 Markdown frontmatter 按 `docnav-markdown` adapter config 中的 enum 策略在 outline 阶段暴露，减少 `outline -> ref -> read` 选择章节前遗漏文档级元数据的风险；它只在 `openspec/changes/markdown-frontmatter-outline-mode/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

Markdown frontmatter 是文档级元数据，不应进入 heading model；但它常包含标题、摘要、分类、状态或渲染配置。调用方在 outline 阶段选择章节前，经常需要先看到这类上下文。

现在需要由 Markdown adapter 识别 frontmatter，并通过一个 adapter-owned enum 配置决定 outline 如何暴露它：直接 inline 原文、提供单独 ref，或完全隐藏。

## What Changes

- Markdown parser 识别文档开头的 YAML frontmatter delimiter block，并继续保证 frontmatter 中的伪 heading 不作为 heading entry 进入 outline。
- `docnav-markdown` adapter config 增加 `options.frontmatter_outline_mode` enum，合法值为 `inline`、`ref` 和 `hidden`，默认值为 `inline`。
- `inline` 模式下，Markdown outline success result 在顶层携带可选 `frontmatter` 字段，包含 `content_type: "application/yaml"` 和不含起止 delimiter 的 YAML 原文 payload；该 payload 的字符预算和续读规则沿用 read content 的分页规则。
- `ref` 模式下，Markdown outline 额外返回一个 frontmatter entry，使用 adapter-owned ref `FM:frontmatter`；调用方读取该 ref 时，read result 以 `application/yaml` primary content 返回 YAML 原文。
- `hidden` 模式下，Markdown outline 不返回 `frontmatter` 字段，也不返回 frontmatter entry。
- 共享协议、readable payload、readable-view 和 MCP mapping 扩展 outline success result，原样承载 Markdown adapter 返回的可选 `frontmatter` 字段；core、MCP 和 shared output 层不解析 YAML。
- readable-view renderer config 增加 optional block pointer 能力：配置声明的可选 block 字段不存在时不报错，存在且为字符串时按普通 block framing 输出。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `adapter-protocol`: 扩展 outline success result，使 adapter 可以返回可选 adapter-owned outline metadata；本 change 的 Markdown-owned 字段为 `frontmatter`。
- `docnav-contracts`: 更新协议、readable schema/example、MCP mapping 和 adapter ownership 边界，声明 frontmatter 由 Markdown adapter 解析并由共享层原样承载。
- `markdown-navigation`: 增加 Markdown frontmatter 识别、`frontmatter_outline_mode` enum、inline/ref/hidden outline 行为、frontmatter ref read 行为和测试边界。
- `readable-view-output`: 增加 optional block pointer 契约，使 outline 的 `/frontmatter/content` 等可选 metadata block 在字段缺失时不触发 render failure。

## Impact

- 受影响 public surface：Markdown outline 的 protocol/readable/MCP 成功结果、Markdown read 对 `FM:frontmatter` 的处理、readable-view block framing、readable-json schema、MCP `document_outline` structuredContent、Markdown `frontmatter_outline_mode` config 语义和示例。
- 受影响代码：Markdown parser/frontmatter extraction、Markdown outline result construction、Markdown read ref handling、shared protocol/readable outline result types、readable renderer config validation 和 block extraction、direct CLI native option handling、schema/example validation。
- 受影响文档与验证材料：`docs/protocol.md`、`docs/output.md`、`docs/mcp.md`、`docs/adapters/markdown.md`、`docs/schemas/**`、`docs/examples/**`、readable conformance vectors 和 Markdown CLI smoke fixtures。
- 不受影响范围：Markdown heading ref grammar、正文 heading outline 可见性、普通 heading/section read content、非 Markdown adapter 的 metadata 策略、YAML 字段语义。
