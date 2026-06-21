本 delta 定义 Markdown frontmatter metadata 如何进入 read success result；它只在 `openspec/changes/markdown-frontmatter-readable-block/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: ReadResult 支持可选 adapter-owned metadata
`docnav-protocol` MUST 允许 read success result 携带可选 adapter-owned metadata。Metadata MUST 是可省略字段；存在时 MUST 保持格式专属语义由 adapter 拥有，共享协议、core CLI 和 MCP 不解析 metadata 内容。

#### Scenario: ReadResult 携带 Markdown frontmatter metadata
- **WHEN** Markdown adapter read 返回启用的 YAML frontmatter metadata
- **THEN** protocol read result 仍包含 ref、content、content_type、cost 和 page
- **THEN** protocol read result 包含可选 frontmatter metadata 字段
- **THEN** frontmatter metadata 包含 `content_type: "application/yaml"` 和 YAML 原文 content
- **THEN** 共享协议不解析 YAML 字段语义

#### Scenario: 无 metadata 时保持旧 read shape
- **WHEN** adapter read 没有返回 metadata
- **THEN** protocol read result 省略 metadata/frontmatter 字段
- **THEN** read result 的 primary content、content_type、cost 和 page 行为保持不变
