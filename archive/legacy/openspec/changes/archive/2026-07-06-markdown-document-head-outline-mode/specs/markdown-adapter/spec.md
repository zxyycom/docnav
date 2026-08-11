本 delta 定义 Markdown document head 的 outline/read/find 行为；它修改 `markdown-adapter` capability，不影响其它 adapter 的 ref grammar 或 shared protocol top-level shape。

## ADDED Requirements

### Requirement: Markdown document head 必须作为合并 ref 暴露
`docnav-markdown` MUST 将文档开头到第一个有效 Markdown heading 起点之前的原文区域定义为 document head。当 document head 包含非空、非纯空白内容，并且当前 structured outline 至少有一个可见 heading entry 时，Markdown outline MUST 在 heading entries 前暴露 ref 为 `HEAD:leading` 的 document head entry。Document head entries MUST NOT 改变 heading ref grammar，并 MUST NOT 新增 outline 顶层 `frontmatter`、`metadata` 或 `document_head` 字段。

#### Scenario: outline 返回 document head entry
- **WHEN** Markdown 文档在第一个有效 heading 前包含 YAML frontmatter 和普通前导正文
- **AND** 当前 structured outline 至少有一个可见 heading entry
- **THEN** outline entries 在 heading entries 之前包含 ref 为 `HEAD:leading` 的 document head entry
- **THEN** 该 entry 的 `label` 非空，`kind` 标明它不是 heading entry
- **THEN** outline result 不包含顶层 `frontmatter`、`metadata` 或 `document_head` 字段

#### Scenario: 空 document head 不返回 entry
- **WHEN** Markdown 文档的第一个非空结构内容就是有效 heading
- **OR** 第一个有效 heading 前只包含空白
- **AND** 当前 structured outline 至少有一个可见 heading entry
- **THEN** outline 不返回 `HEAD:leading`
- **THEN** heading entries 保持既有顺序和 ref grammar

#### Scenario: read 返回 document head 原文
- **WHEN** 调用方读取 `HEAD:leading`
- **THEN** read result 的 primary content 为从文档开头到第一个有效 Markdown heading 之前的原文区域
- **THEN** 如果该区域包含 YAML frontmatter delimiter，content 保留起止 delimiter
- **THEN** read result 的 `content_type` 为 `text/markdown`
- **THEN** read 的 `limit_chars` 和 `page` 行为按普通 read content 分页规则处理

#### Scenario: 没有可见 heading 时保留 doc full fallback
- **WHEN** 当前 outline 参数过滤后没有可见 heading entry
- **THEN** Markdown outline 仍返回 ref 为 `doc:full` 的单条 entry
- **THEN** outline 不只返回 document head entry
- **THEN** 使用该 ref 执行 read 返回整篇 Markdown 文档

### Requirement: Find 命中 document head 时必须返回可 read 的区域 ref
Markdown find MUST 搜索全文。当匹配位于 document head 内且当前 structured outline 至少有一个可见 heading entry 时，match ref MUST 指向 `HEAD:leading`。当当前 outline 使用 `doc:full` fallback 时，find MUST 保持可继续 read 的既有 fallback 行为。

#### Scenario: find 使用 HEAD ref
- **WHEN** query 命中第一个有效 Markdown heading 前的普通前导正文
- **AND** 当前 structured outline 至少有一个可见 heading entry
- **THEN** match ref 为 `HEAD:leading`
- **THEN** 使用该 ref 执行 read 返回包含命中文本的 content

#### Scenario: fallback find 保持可读
- **WHEN** query 命中 document head
- **AND** 当前 outline 使用 `doc:full` fallback
- **THEN** match ref 仍必须能通过 read 读取包含命中文本的内容

### Requirement: Markdown document head 文档和测试材料必须覆盖可观察边界
Markdown adapter 主文档、adapter tests、case ledger 和 smoke tests MUST 覆盖 `HEAD:leading`、read content type、raw entry facts、fallback 和 find-to-read roundtrip。

#### Scenario: 测试覆盖 document head roundtrip
- **WHEN** Markdown adapter tests 或 smoke tests 读取包含 frontmatter、普通前导正文和 heading 的 fixture
- **THEN** 测试覆盖 outline 到 `HEAD:leading` read 的 roundtrip
- **THEN** 测试覆盖空或纯空白 document head、无可见 heading fallback、find 命中 document head 和 Unicode 分页边界
