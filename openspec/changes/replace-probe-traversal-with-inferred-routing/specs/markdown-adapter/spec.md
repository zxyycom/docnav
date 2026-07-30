本 delta spec 是 `replace-probe-traversal-with-inferred-routing` 的临时契约工件：它删除 Markdown-owned selection probe，让 Markdown 只在 navigation 已精确选中它之后执行既有 operation 语义。

## REMOVED Requirements

### Requirement: Probe recognizes only Markdown format support

**Reason**: Automatic format recognition moves to one navigation-private inference invocation, and the fixed adapter strategy no longer has a routing probe.

**Migration**: Map the approved inference result to normalized format id `markdown`, exact-match that id to the Markdown definition manifest, retain all actual Markdown decode, parse, ref, and operation validation inside the selected Markdown strategy, and delete Markdown probe code/tests plus shared probe schema/fixtures after the blocking compatibility gate confirms there is no real owner-backed consumer. If such a consumer is found, current apply stops and returns to artifacts/human approval; it does not retain an inspection surface.

Markdown probe behavior MUST identify Markdown support and report unsupported input without claiming non-Markdown format ownership.

#### Scenario: Markdown file

- **WHEN** probe receives a Markdown document path
- **THEN** it reports supported Markdown facts

#### Scenario: Non-Markdown file

- **WHEN** probe receives a document that is not recognized as Markdown
- **THEN** it reports unsupported without parsing it as Markdown
