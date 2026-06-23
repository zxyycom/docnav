本 delta 定义 Markdown frontmatter 的识别、配置和 outline 暴露模式；它只在 `openspec/changes/markdown-frontmatter-outline-mode/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Markdown frontmatter 可按 outline mode 暴露
`docnav-markdown` MUST 识别文档开头的 YAML frontmatter delimiter block。Frontmatter MUST 继续排除在 heading model、heading outline entry 和 heading index 之外。Markdown outline MUST 按 adapter-owned `frontmatter_outline_mode` 决定 frontmatter 暴露方式：`inline` 在 outline result 顶层返回 `frontmatter` metadata，`ref` 在 outline entries 中返回 `FM:frontmatter` ref，`hidden` 不暴露 frontmatter。

#### Scenario: 默认 inline outline 附带 frontmatter
- **WHEN** Markdown 文档以可识别 YAML frontmatter delimiter block 开头
- **AND** Markdown outline 的生效 adapter option 为默认值 `frontmatter_outline_mode: "inline"`
- **THEN** outline result 包含顶层 `frontmatter` 字段
- **THEN** `frontmatter.content_type` 为 `application/yaml`
- **THEN** `frontmatter.content` 为当前 outline page 返回的 delimiter 内部 YAML 原文 payload slice，且不包含起止 delimiter
- **THEN** heading entries 仍只包含 heading 或全文 fallback 的 `ref` 和 `display`

#### Scenario: inline frontmatter 使用 read content 分页规则
- **WHEN** Markdown frontmatter payload 超过当前 `limit_chars`
- **AND** Markdown outline 使用 `frontmatter_outline_mode: "inline"`
- **THEN** `frontmatter.content` 按 read content 的 Unicode 字符计数规则截取当前 page slice
- **THEN** adapter 不切断 Unicode 字符
- **THEN** outline result 的 `page` 返回下一页 page
- **THEN** 当前 page 的 `entries` 可以为空
- **THEN** 使用相同 path、mode、limit 和下一页 page 继续 outline 可读取后续 frontmatter payload 或 heading entries

#### Scenario: ref 模式返回 frontmatter outline entry
- **WHEN** Markdown 文档以可识别 YAML frontmatter delimiter block 开头
- **AND** Markdown outline 的生效 adapter option 为 `frontmatter_outline_mode: "ref"`
- **THEN** outline entries 包含一个 ref 为 `FM:frontmatter` 的 frontmatter entry
- **THEN** 该 entry 的 display 非空，并提供 frontmatter 摘要或 cost
- **THEN** 该 entry 按文档顺序位于正文 heading entries 之前
- **THEN** outline result 不包含顶层 `frontmatter` 字段
- **WHEN** 调用方读取 `FM:frontmatter`
- **THEN** read result 的 primary content 为 delimiter 内部 YAML 原文 payload
- **THEN** read result 的 `content_type` 为 `application/yaml`
- **THEN** read 的 `limit_chars` 和 `page` 行为按普通 read content 分页规则处理

#### Scenario: hidden 模式不暴露 frontmatter
- **WHEN** Markdown 文档以可识别 YAML frontmatter delimiter block 开头
- **AND** Markdown outline 的生效 adapter option 为 `frontmatter_outline_mode: "hidden"`
- **THEN** outline result 不包含顶层 `frontmatter` 字段
- **THEN** outline entries 不包含 `FM:frontmatter`
- **THEN** heading entries、display 和 ref 仍只来自正文中的有效 heading 或全文 fallback

#### Scenario: Frontmatter 中的伪 heading 不进入 heading model
- **WHEN** Markdown frontmatter 中包含看似 heading 的文本
- **THEN** outline 不为该文本生成 heading entry
- **THEN** heading index、display 和 heading ref 仍只来自正文中的有效 heading

### Requirement: Markdown frontmatter outline mode 配置必须由 adapter 拥有
`docnav-markdown` MUST 拥有 adapter-owned enum option `frontmatter_outline_mode`，合法值 MUST 为 `inline`、`ref` 和 `hidden`，默认值 MUST 为 `inline`。该 option MUST 进入 `docnav-markdown` adapter config 的 `options.frontmatter_outline_mode`。共享层 MUST 只把 options 或结果当作 opaque adapter-owned data 传递。

#### Scenario: Adapter config 设置 frontmatter outline mode
- **WHEN** `.docnav/docnav-markdown.json` 包含 `options.frontmatter_outline_mode: "ref"`
- **AND** 调用方执行 `docnav-markdown outline <path>`
- **THEN** Markdown outline 的最终语义输入显式包含 `arguments.options.frontmatter_outline_mode: "ref"`
- **THEN** outline 按 `ref` 模式返回 frontmatter entry

#### Scenario: 显式 option 覆盖 adapter config
- **WHEN** adapter config 包含 `options.frontmatter_outline_mode: "hidden"`
- **AND** 调用方通过 direct CLI 或 invoke semantic request 显式传入 `frontmatter_outline_mode: "inline"`
- **THEN** Markdown outline 使用 `inline` 模式
- **THEN** 配置值 `hidden` 不覆盖显式输入

#### Scenario: 默认 inline 不需要配置文件
- **WHEN** 生效配置和显式 adapter option 都未提供 `frontmatter_outline_mode`
- **THEN** Markdown outline 使用 `inline` 模式
- **THEN** 有 frontmatter 的文档在 outline result 顶层返回 `frontmatter` 字段
