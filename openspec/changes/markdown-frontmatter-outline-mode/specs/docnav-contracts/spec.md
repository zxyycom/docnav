本 delta 定义 Docnav 共享契约如何承载 Markdown outline frontmatter metadata；它只在 `openspec/changes/markdown-frontmatter-outline-mode/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Outline 输出可以承载 adapter-owned metadata
Docnav outline 契约 MUST 允许 adapter 在成功 outline result 中返回可选 metadata。Core CLI、MCP bridge、readable output 和 schema/example MUST 原样承载该 metadata，并 MUST NOT 从 path、content 或 Markdown 语法重新解析 metadata。Markdown frontmatter inline metadata 的稳定承载字段为 `frontmatter`。

#### Scenario: Core 和 MCP 原样承载 outline frontmatter metadata
- **WHEN** Markdown adapter outline 返回顶层 `frontmatter` metadata
- **THEN** `docnav` readable-json 和 protocol-json outline 输出保留 `frontmatter.content_type` 和 `frontmatter.content`
- **THEN** `docnav-mcp` `document_outline` structuredContent 保留 `frontmatter.content_type` 和 `frontmatter.content`
- **THEN** core 和 MCP 不解析 YAML 内容

#### Scenario: 非 inline 模式不改变普通 outline envelope
- **WHEN** Markdown adapter outline 未返回顶层 `frontmatter` metadata
- **THEN** outline 输出不包含顶层 `frontmatter` 字段
- **THEN** outline entries、page 和 heading ref/display 按既有契约输出

#### Scenario: Adapter config 仍由 Markdown adapter direct CLI 拥有
- **WHEN** `.docnav/docnav-markdown.json` 或默认用户配置中的 `options.frontmatter_outline_mode` 设置为 `ref`、`inline` 或 `hidden`
- **AND** 调用方执行 `docnav-markdown outline <path>`
- **THEN** `docnav-markdown` direct CLI 按 adapter config 优先级合并该值
- **THEN** core `docnav` 和 `docnav-mcp` 不读取该 adapter config 文件
