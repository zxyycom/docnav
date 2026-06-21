本 delta 定义 readable-view renderer 的 optional block 能力和 frontmatter block 输出；它只在 `openspec/changes/markdown-frontmatter-readable-block/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: readable-view renderer 支持 optional block pointers
Readable-view renderer config MUST 支持 required block pointers 和 optional block pointers。Required pointer 缺失、类型错误、重复或 identity 冲突仍 MUST 触发 `readable_view_render_failed`；optional pointer 缺失时 MUST 跳过该 block，存在但类型错误、重复或 identity 冲突时 MUST 按 render failure 处理。

#### Scenario: Optional block 字段缺失时跳过
- **WHEN** renderer config 为 read 声明 optional block pointer `/frontmatter/content`
- **AND** readable payload 不包含 frontmatter 字段
- **THEN** readable-view rendering 成功
- **THEN** stdout 不包含 `/frontmatter/content` block
- **THEN** required `/content` block 行为不变

#### Scenario: Optional block 字段存在时外置
- **WHEN** renderer config 为 read 声明 optional block pointer `/frontmatter/content`
- **AND** readable payload 包含字符串字段 `/frontmatter/content`
- **THEN** JSON header 在该字段位置包含 `$block: "/frontmatter/content"` 和 UTF-8 byte length
- **THEN** stdout 包含 `/frontmatter/content` block
- **THEN** block payload 等于 readable-json 中对应字段字符串

#### Scenario: Optional block 类型错误仍失败
- **WHEN** renderer config 为 read 声明 optional block pointer `/frontmatter/content`
- **AND** readable payload 中该 pointer 存在但目标值不是字符串
- **THEN** renderer 返回 `readable_view_render_failed`
- **THEN** stdout 为空且 stderr 包含稳定诊断

### Requirement: read frontmatter 使用 optional metadata block
Read readable payload 包含 frontmatter metadata 时，readable-view MUST 使用 optional `/frontmatter/content` block 承载 YAML 原文；没有 frontmatter metadata 时 readable-view MUST 不显示 frontmatter 字段或 block。

#### Scenario: Readable-view 输出 frontmatter block
- **WHEN** read result 包含 frontmatter metadata
- **THEN** readable-view header 包含 frontmatter content_type
- **THEN** readable-view header 的 frontmatter content 字段为 `/frontmatter/content` block 引用
- **THEN** `/frontmatter/content` block payload 等于 readable-json frontmatter content

#### Scenario: 无 frontmatter 时不显示 metadata
- **WHEN** read result 不包含 frontmatter metadata
- **THEN** readable-json 不包含 frontmatter 字段
- **THEN** readable-view header 不包含 frontmatter 字段
- **THEN** stdout 不包含 `/frontmatter/content` block
