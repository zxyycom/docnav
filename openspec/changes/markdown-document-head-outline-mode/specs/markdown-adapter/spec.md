本 delta 定义 Markdown document head 的 outline/read/find 行为；它只在 `openspec/changes/markdown-document-head-outline-mode/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Markdown document head 可按 outline mode 暴露
`docnav-markdown` MUST 将第一个有效 Markdown heading 前的原文区域定义为 document head。Markdown outline MUST 按 adapter-owned `document_head_outline_mode` 决定 document head 暴露方式：`combined` 在 outline entries 中返回 `HEAD:leading` ref，`split` 在 outline entries 中分别返回 frontmatter 和 preamble refs，`hidden` 不暴露 document head。默认值 MUST 为 `combined`。Document head entries MUST 位于 heading entries 之前，且 MUST NOT 改变 heading ref grammar。

#### Scenario: 默认 combined 返回 document head entry
- **WHEN** Markdown 文档在第一个有效 heading 前包含 YAML frontmatter 和普通前导正文
- **AND** Markdown outline 的生效 adapter option 未提供 `document_head_outline_mode`
- **THEN** outline 使用默认 `combined` 模式
- **THEN** outline entries 在 heading entries 之前包含 ref 为 `HEAD:leading` 的 document head entry
- **THEN** 该 entry 的 display 非空，并标明它不是 heading entry
- **THEN** outline result 不包含顶层 `frontmatter`、`metadata` 或 `document_head` 字段

#### Scenario: combined read 返回 document head 原文
- **WHEN** 调用方读取 `HEAD:leading`
- **THEN** read result 的 primary content 为从文档开头到第一个有效 Markdown heading 之前的原文区域
- **THEN** 如果该区域包含 YAML frontmatter delimiter，content 保留起止 delimiter
- **THEN** read result 的 `content_type` 为 `text/markdown`
- **THEN** read 的 `limit_chars` 和 `page` 行为按普通 read content 分页规则处理

#### Scenario: split 返回 frontmatter 和 preamble refs
- **WHEN** Markdown 文档在第一个有效 heading 前包含可识别 frontmatter block 和 frontmatter 后的普通前导正文
- **AND** Markdown outline 的生效 adapter option 为 `document_head_outline_mode: "split"`
- **THEN** outline entries 在 heading entries 之前按文档顺序包含 frontmatter entry 和 preamble entry
- **THEN** frontmatter entry 的 ref 使用 `FM:L{line}` 形式，其中 line 为该 frontmatter block 起始行号
- **THEN** preamble entry 的 ref 为 `P:preamble`
- **THEN** outline result 不包含顶层 `frontmatter`、`metadata` 或 `document_head` 字段

#### Scenario: split read 返回 typed region content
- **WHEN** 调用方读取 split 模式返回的 `FM:L{line}` ref
- **THEN** read result 的 primary content 为对应 frontmatter delimiter 内部 YAML 原文 payload，且不包含起止 delimiter
- **THEN** read result 的 `content_type` 为 `application/yaml`
- **WHEN** 调用方读取 `P:preamble`
- **THEN** read result 的 primary content 为 recognized frontmatter block 之后到第一个有效 Markdown heading 之前的 Markdown 原文
- **THEN** read result 的 `content_type` 为 `text/markdown`

#### Scenario: hidden 保持 heading outline
- **WHEN** Markdown outline 的生效 adapter option 为 `document_head_outline_mode: "hidden"`
- **THEN** outline entries 不包含 `HEAD:leading`
- **THEN** outline entries 不包含 `FM:L{line}` 或 `P:preamble`
- **THEN** heading entries、display 和 heading ref 仍只来自正文中的有效 heading 或全文 fallback

#### Scenario: 没有可见 heading 时保留 doc full fallback
- **WHEN** 当前 outline 参数过滤后没有可见 heading entry
- **THEN** Markdown outline 仍返回 ref 为 `doc:full` 的单条 entry
- **THEN** outline 不只返回 document head entry
- **THEN** 使用该 ref 执行 read 返回整篇 Markdown 文档

### Requirement: Markdown frontmatter block policy 必须显式决定多 block 方言
`docnav-markdown` MUST 拥有 adapter-owned `frontmatter_block_policy`，默认值 MUST 为 `opening_only`。`opening_only` MUST 只识别文档开头的一个 YAML delimiter block。多 metadata block 识别 MUST 只能通过显式 policy 启用，并 MUST 保持普通 Markdown horizontal rule 和未闭合 delimiter 不被误判为 frontmatter。

#### Scenario: opening only 只识别第一个开头 block
- **WHEN** Markdown 文档以一个可识别 YAML delimiter block 开头
- **AND** 后续 document head 中再次出现 `---` delimiter 文本
- **AND** 生效 `frontmatter_block_policy` 为默认 `opening_only`
- **THEN** adapter 只把文档开头第一个 delimiter block 识别为 frontmatter
- **THEN** 后续 `---` 文本保留在 combined `HEAD:leading` 原文或 split `P:preamble` 内容中

#### Scenario: pandoc metadata blocks 必须显式启用
- **WHEN** 生效 `frontmatter_block_policy` 为多 metadata block 方言值
- **AND** document head 中存在多个符合该方言规则的 metadata blocks
- **THEN** split outline 为每个 recognized metadata block 返回一个 `FM:L{line}` entry
- **THEN** 每个 entry 的 read content 为对应 block 的 delimiter 内部 payload
- **THEN** entries 按源码顺序稳定排列

#### Scenario: 未闭合 delimiter 不是 frontmatter
- **WHEN** Markdown 文档开头出现未闭合的 YAML delimiter
- **THEN** adapter 不把该文本识别为 frontmatter block
- **THEN** combined `HEAD:leading` read 保留该文本作为 Markdown 原文
- **THEN** split outline 不为该文本返回 `FM:L{line}` entry

### Requirement: Find 命中 document head 时必须返回可 read 的区域 ref
Markdown find MUST 搜索全文。当匹配位于 document head 内且当前 outline 以 `combined` 或 `split` 暴露对应 document head 区域时，match ref MUST 指向可通过 read 读取该命中文本的 document head ref。当 document head 被隐藏或当前 outline 使用 `doc:full` fallback 时，find MUST 保持可继续 read 的既有 fallback 行为。

#### Scenario: combined find 使用 HEAD ref
- **WHEN** query 命中第一个有效 Markdown heading 前的普通前导正文
- **AND** Markdown find 的生效 adapter option 为 `document_head_outline_mode: "combined"`
- **THEN** match ref 为 `HEAD:leading`
- **THEN** 使用该 ref 执行 read 返回包含命中文本的 content

#### Scenario: split find 使用最具体 region ref
- **WHEN** query 命中 recognized frontmatter block 内部 payload
- **AND** Markdown find 的生效 adapter option 为 `document_head_outline_mode: "split"`
- **THEN** match ref 为对应 `FM:L{line}`
- **WHEN** query 命中 frontmatter 后、第一个有效 heading 前的普通前导正文
- **THEN** match ref 为 `P:preamble`
- **THEN** 使用返回 ref 执行 read 返回包含命中文本的 content

#### Scenario: hidden find 保持 fallback 可读
- **WHEN** query 命中 document head
- **AND** Markdown find 的生效 adapter option 为 `document_head_outline_mode: "hidden"`
- **THEN** adapter 不返回 document head ref
- **THEN** match ref 仍必须能通过 read 读取包含命中文本的内容

### Requirement: Markdown document head 配置和测试材料必须覆盖可观察边界
`docnav-markdown` config schema/example、adapter tests 和 smoke tests MUST 覆盖 `document_head_outline_mode`、`frontmatter_block_policy`、document head refs、read content type、fallback 和 find-to-read roundtrip。实现前 MUST 同步更新 Markdown adapter 主文档，明确本 change 取代或 supersede 较窄的 frontmatter-only outline 方案。

#### Scenario: 配置 schema 和示例包含新 options
- **WHEN** docs validator 校验 `docs/examples/json/docnav-markdown-config.json`
- **THEN** 示例符合 `docs/schemas/docnav-markdown-config.schema.json`
- **THEN** schema 描述 `options.document_head_outline_mode` 的合法值 `combined`、`split` 和 `hidden`
- **THEN** schema 描述 `options.frontmatter_block_policy` 的合法值和默认语义

#### Scenario: 测试覆盖 document head roundtrip
- **WHEN** Markdown adapter tests 或 smoke tests 读取包含 frontmatter、preamble 和 heading 的 fixture
- **THEN** 测试覆盖 combined outline 到 `HEAD:leading` read 的 roundtrip
- **THEN** 测试覆盖 split outline 到 `FM:L{line}` 和 `P:preamble` read 的 roundtrip
- **THEN** 测试覆盖 hidden 模式、无可见 heading fallback、find 命中 document head 和 Unicode 分页边界
