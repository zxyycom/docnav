本 delta 定义 Markdown frontmatter metadata 如何进入 outline success result；它只在 `openspec/changes/markdown-frontmatter-outline-mode/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: OutlineResult 支持可选 adapter-owned metadata
`docnav-protocol` MUST 允许 outline success result 携带可选 adapter-owned metadata。Metadata MUST 是可省略字段；存在时 MUST 保持格式专属语义由 adapter 拥有，共享协议、core CLI 和 MCP 不解析 metadata 内容。Markdown frontmatter inline metadata 的可观察字段名 MUST 为 `frontmatter`。

#### Scenario: OutlineResult 携带 Markdown frontmatter inline metadata
- **WHEN** Markdown adapter outline 使用 `frontmatter_outline_mode: "inline"` 返回 YAML frontmatter metadata
- **THEN** protocol outline result 仍包含 `entries` 和 `page`
- **THEN** protocol outline result 包含可选 `frontmatter` 字段
- **THEN** `frontmatter.content_type` 为 `application/yaml`
- **THEN** `frontmatter.content` 为当前 outline page 返回的 frontmatter YAML 原文 payload slice，且不包含起止 delimiter
- **THEN** outline result 的 `page` 继续表示整个 outline operation 的下一页或结束
- **THEN** 共享协议不解析 YAML 字段语义

#### Scenario: 非 inline frontmatter 模式保持 outline metadata 省略
- **WHEN** Markdown adapter outline 使用 `frontmatter_outline_mode: "ref"` 或 `frontmatter_outline_mode: "hidden"`
- **OR** 当前文档没有可识别 frontmatter
- **THEN** protocol outline result 省略顶层 `frontmatter` 字段
- **THEN** outline result 的 `entries` 和 `page` 使用既有 outline envelope
