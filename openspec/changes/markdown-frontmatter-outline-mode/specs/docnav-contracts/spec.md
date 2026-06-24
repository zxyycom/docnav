本 delta 定义 Docnav 共享契约如何承载 Markdown outline frontmatter metadata；它只在 `openspec/changes/markdown-frontmatter-outline-mode/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Outline 输出可以承载 adapter-owned metadata

Docnav outline output MUST allow an adapter to return optional metadata in successful outline results. Core CLI, readable output and schema/example material MUST preserve that metadata and MUST NOT parse it from path, content or Markdown syntax.

#### Scenario: Core 原样承载 outline frontmatter metadata
- **WHEN** Markdown adapter outline 返回顶层 `frontmatter` metadata
- **THEN** `docnav` readable-json 和 protocol-json outline 输出保留 `frontmatter.content_type` 和 `frontmatter.content`

#### Scenario: 非 inline 模式不改变普通 outline envelope
- **WHEN** Markdown adapter outline 未返回顶层 `frontmatter` metadata
- **THEN** outline 输出不包含顶层 `frontmatter` 字段
- **THEN** outline entries、page 和 heading ref/display 按既有契约输出

#### Scenario: Adapter config 仍由 Markdown adapter direct CLI 拥有
- **WHEN** `.docnav/docnav-markdown.json` 或默认用户配置中的 `options.frontmatter_outline_mode` 设置为 `ref`、`inline` 或 `hidden`
- **AND** 调用方执行 `docnav-markdown outline <path>`
- **THEN** `docnav-markdown` direct CLI 按 adapter config 优先级合并该值
