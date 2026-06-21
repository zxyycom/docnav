本 delta 定义 Markdown frontmatter 的识别、配置和 read metadata 输出；它只在 `openspec/changes/markdown-frontmatter-readable-block/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Markdown frontmatter 可作为 read metadata 输出
`docnav-markdown` MUST 识别文档开头的 YAML frontmatter。Frontmatter MUST 继续排除在 heading outline 之外；当生效配置或显式 adapter option 启用时，普通 heading/section read MUST 在 primary content 之外返回 frontmatter metadata。

#### Scenario: 普通 section read 附带 frontmatter
- **WHEN** Markdown 文档以合法 YAML frontmatter 开头
- **AND** Markdown read 的生效配置或显式 adapter option 启用 frontmatter metadata
- **AND** 调用方读取一个 heading section
- **THEN** read result 的 primary content 仍为该 heading section
- **THEN** read result 包含 frontmatter metadata
- **THEN** frontmatter metadata content_type 为 `application/yaml`
- **THEN** frontmatter metadata content 等于 YAML frontmatter 原文内容

#### Scenario: Frontmatter 不进入 outline
- **WHEN** Markdown frontmatter 中包含看似 heading 的文本
- **THEN** outline 不为该文本生成 heading entry
- **THEN** heading index、display 和 ref 仍只来自正文中的有效 heading

#### Scenario: 全文读取默认不重复 frontmatter
- **WHEN** 调用方读取 `doc:full`
- **OR** 非结构化 outline 全文读取返回整篇原文
- **THEN** primary content 已包含 frontmatter 原文
- **THEN** 默认不额外返回 frontmatter metadata block

### Requirement: Markdown frontmatter 配置必须由 adapter 拥有
`docnav-markdown` MUST 拥有普通 read 是否默认附带 frontmatter metadata 的 adapter-owned 开关。本 change MUST NOT 定义具体配置文件、格式或合并方式。Direct CLI 或上游调用方 MUST 在调用 Markdown read 前把最终选择写入 adapter-owned option 或等价语义输入；共享层 MUST 只把 options 或结果当作 opaque adapter-owned data 传递。

#### Scenario: Adapter 开关启用 frontmatter metadata
- **WHEN** Markdown read 的生效配置或显式 adapter option 启用普通 read 附带 frontmatter metadata
- **AND** 调用方执行 `docnav-markdown read <path> --ref <heading-ref>`
- **THEN** Markdown read 的最终语义输入显式包含该 adapter-owned option 或等价开关
- **THEN** read 成功结果包含 frontmatter metadata

#### Scenario: 默认 omit 保持兼容
- **WHEN** 生效配置和显式 adapter option 都未启用 frontmatter metadata
- **THEN** Markdown read 成功结果不包含 frontmatter metadata
- **THEN** 既有 read content、pagination 和 content_type 行为不变
