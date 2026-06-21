本 delta 定义 Docnav 共享契约如何承载 Markdown frontmatter metadata；它只在 `openspec/changes/markdown-frontmatter-readable-block/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Read 输出可以承载 adapter-owned metadata
Docnav read 契约 MUST 允许 adapter 在成功 read result 中返回可选 metadata。Core CLI、MCP bridge、readable output 和 schema/example MUST 原样承载该 metadata，并 MUST NOT 从 path、content 或 Markdown 语法重新解析 metadata。

#### Scenario: Core 和 MCP 原样承载 frontmatter metadata
- **WHEN** Markdown adapter 返回 frontmatter metadata
- **THEN** `docnav` readable-json 和 protocol-json 输出保留该 metadata
- **THEN** `docnav-mcp` structuredContent 保留该 metadata
- **THEN** core 和 MCP 不解析 YAML 内容

#### Scenario: 未启用 frontmatter 时不改变普通 read
- **WHEN** Markdown adapter 未返回 frontmatter metadata
- **THEN** read 输出不包含 frontmatter 字段
- **THEN** read primary content、page、cost 和 content_type 按既有契约输出
