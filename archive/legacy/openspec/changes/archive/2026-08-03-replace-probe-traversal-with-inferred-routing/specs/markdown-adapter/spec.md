本 delta spec 定义 `replace-probe-traversal-with-inferred-routing` 对 `markdown-adapter` 尚未应用的 Target：删除 Markdown-owned selection probe，Markdown 只在 navigation 已通过 manifest pathname hint 或显式 id 精确选中后执行既有 operation 语义；它不表示 Current 主规范或实现已经迁移。

## REMOVED Requirements

### Requirement: Probe recognizes only Markdown format support

**Reason**: Automatic selection moves to one navigation-private, manifest-derived pathname lookup, and the fixed adapter strategy no longer has a routing probe.

**Migration**: Map the manifest-owned `.md` and `.markdown` basename suffix hints to normalized format id `markdown`, exact-match that id to the Markdown definition manifest, retain all actual Markdown decode, parse, ref, and operation validation inside the selected Markdown strategy, and delete Markdown probe code/tests plus shared probe schema/fixtures after the blocking removal inventory is complete. Every discovered consumer is deleted, migrated, or recorded as an explicit breaking impact; no compatibility or inspection surface is retained.

Markdown probe behavior MUST identify Markdown support and report unsupported input without claiming non-Markdown format ownership.

#### Scenario: Markdown file

- **WHEN** probe receives a Markdown document path
- **THEN** it reports supported Markdown facts

#### Scenario: Non-Markdown file

- **WHEN** probe receives a document that is not recognized as Markdown
- **THEN** it reports unsupported without parsing it as Markdown
