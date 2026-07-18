# markdown-adapter Specification

## Purpose
Define Markdown adapter behavior: Markdown format probing, parser boundaries, outline/read/find/info semantics, Markdown ref grammar, display facts, pagination and cost behavior, native options, and declared unstructured full-read support. Core CLI, navigation, protocol, output, diagnostics, and shared ref opacity are consumed capabilities, not Markdown-owned rules.
## Requirements
### Requirement: Markdown adapter provides v0 document operations
The Markdown adapter MUST implement outline, read, find, and info for Markdown documents through the linked adapter contract.

#### Scenario: Supported Markdown document
- **WHEN** the selected adapter is Markdown and the document is supported
- **THEN** outline, read, find, and info are available through the standard document operation flow

### Requirement: Probe recognizes only Markdown format support
Markdown probe behavior MUST identify Markdown support and report unsupported input without claiming non-Markdown format ownership.

#### Scenario: Markdown file
- **WHEN** probe receives a Markdown document path
- **THEN** it reports supported Markdown facts

#### Scenario: Non-Markdown file
- **WHEN** probe receives a document that is not recognized as Markdown
- **THEN** it reports unsupported without parsing it as Markdown

### Requirement: Markdown outline returns flat bounded entries
Markdown outline MUST return document-order flat entries with adapter-generated refs and compact display. Code-fence pseudo headings MUST NOT become entries. When filtering leaves no visible heading entry, outline MUST return the whole-document ref entry.

#### Scenario: Nested headings
- **WHEN** a Markdown document contains H1, H2, and H3 headings
- **THEN** outline returns flat entries in document order
- **THEN** each entry contains a unique Markdown ref

#### Scenario: Code fence pseudo heading
- **WHEN** a fenced code block contains text that looks like a heading
- **THEN** outline does not emit an entry for that text

#### Scenario: No visible heading
- **WHEN** current outline parameters leave no heading entry visible
- **THEN** outline returns the whole-document ref entry
- **THEN** read can use that ref to return the whole Markdown document

### Requirement: Markdown read matches canonical refs precisely
Markdown read MUST parse Markdown-owned refs and return the exact referenced region. It MUST distinguish invalid ref grammar, valid-but-unmatched refs, ambiguous refs, and whole-document refs.

#### Scenario: Heading roundtrip
- **WHEN** a caller passes an outline heading ref to read
- **THEN** read returns the corresponding Markdown section
- **THEN** `content_type` is `text/markdown`

#### Scenario: Duplicate heading path
- **WHEN** a document contains duplicate complete heading paths
- **THEN** outline emits distinct refs
- **THEN** read can locate each region separately

#### Scenario: Invalid grammar
- **WHEN** a ref does not match Markdown ref grammar
- **THEN** Markdown reports an invalid-ref diagnostic

#### Scenario: Valid grammar with no match
- **WHEN** a ref matches Markdown grammar but no current region matches it
- **THEN** Markdown reports a ref-not-found diagnostic

### Requirement: Markdown heading refs use canonical snapshot grammar
Markdown heading refs MUST use a canonical, field-tagged grammar that captures the structural snapshot needed for precise matching without requiring shared layers to parse it.

#### Scenario: Heading ref is emitted
- **WHEN** outline emits a heading ref
- **THEN** the ref includes Markdown-owned structural fields
- **THEN** shared layers still treat the ref as opaque

### Requirement: Markdown find returns bounded readable matches
Markdown find MUST return bounded matches with refs that can be read. Match display MUST preserve readable match context without becoming the machine owner for the match facts.

#### Scenario: Match in section
- **WHEN** find matches text inside a Markdown section
- **THEN** the match includes a Markdown ref
- **THEN** read with that ref returns content corresponding to the match region

### Requirement: Markdown info returns compact format facts
Markdown info MUST return a compact summary of Markdown document facts without exposing parser-internal structures as public contract.

#### Scenario: Info request
- **WHEN** info is called for a Markdown document
- **THEN** the result includes stable summary facts useful for navigation
- **THEN** it does not expose private parser state

### Requirement: Markdown pagination and cost use selected output text
Markdown outline, read, and find MUST apply the active pagination budget to selected output text and MUST report cost through shared protocol-compatible cost measurements.

#### Scenario: Read exceeds budget
- **WHEN** a Markdown section exceeds the active limit
- **THEN** read returns bounded content
- **THEN** it exposes the next page value

### Requirement: Markdown supports declared unstructured full-read outline
Markdown unstructured full-read outline support MUST be declared through adapter hook metadata before navigation can use it. Normal structured outline behavior MUST remain unchanged when the policy does not apply.

#### Scenario: Policy triggers unstructured full read
- **WHEN** navigation pre-dispatch selects unstructured full-read for a Markdown document
- **THEN** Markdown supplies the full content through the declared hook
- **THEN** the result is not represented as heading entries

#### Scenario: Policy does not trigger
- **WHEN** unstructured full-read policy does not apply
- **THEN** Markdown uses normal structured outline behavior

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

### Requirement: Markdown consumes core-defined adapter-scoped parameters

Markdown adapter MUST implement the fixed adapter strategy interface and consume the closed operation-specific input defined by the shared operation contract and populated from core catalog resolution. For `max_heading_level`, core MUST own its public flag/config path, standard integer type, public range `1..=6`, default `3`, source resolution, outline/find binding, exact `docnav-markdown` marker, pre-dispatch validation policy, and compile-time standard-input binding. Markdown MUST receive the resolved integer through the typed input field/accessor rather than generic parameter or protocol lookup. Markdown MUST own how that integer filters headings and MAY repeat the range check or perform additional algorithmic semantic validation before use. Markdown schema, examples, and strategy checks remain validation material rather than independent parameter declarations.

#### Scenario: Markdown parameter is configured

- **WHEN** project or user config provides valid `options.docnav-markdown.max_heading_level`
- **THEN** navigation resolves the source through core catalog
- **THEN** Markdown receives the standard integer accessor without parsing protocol `Options`
- **THEN** outline/find apply that value through the standard strategy input

#### Scenario: Core rejects the current public range

- **WHEN** caller input provides `max_heading_level` outside `1..=6`
- **THEN** core-owned input resolution reports the diagnostic before dispatch
- **THEN** the existing caller-visible behavior remains compatible

#### Scenario: Markdown defensively repeats a semantic check

- **WHEN** a well-typed standard input reaches Markdown through an internal or deliberately deferred validation path
- **THEN** Markdown may validate `max_heading_level` before applying the heading strategy
- **THEN** rejection maps to a compatible diagnostic
- **THEN** the check does not declare the parameter or participate in source resolution

