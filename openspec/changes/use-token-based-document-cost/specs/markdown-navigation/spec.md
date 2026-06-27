本 delta spec 记录 Markdown adapter 将 section/read cost 改为 token-informed 估算的可观察行为；当前只在 `openspec/changes/use-token-based-document-cost/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Markdown cost uses token-informed estimates
`docnav-markdown` MUST include a token-informed count in read result `cost` and outline section cost summaries. The token count MUST be calculated from the Markdown text represented by the read target or outline section using the tokenizer selected and documented by the implementation, and a byte- or KB-only file size summary MUST NOT be the sole document cost signal.

#### Scenario: Read result reports token cost for selected target
- **WHEN** a caller reads a Markdown heading ref
- **THEN** the read result `cost` contains a token count for the selected heading section before pagination
- **THEN** the `content`, `content_type`, `ref`, and `page` fields keep their existing semantics

#### Scenario: Outline display reports section token cost
- **WHEN** a caller outlines a Markdown document with visible heading entries
- **THEN** each outline entry display contains the heading navigation text and a token-informed section cost
- **THEN** the entry ref remains the adapter-owned opaque ref and is not derived from the cost text

#### Scenario: Full-document outline reports token cost
- **WHEN** Markdown outline visibility filtering produces the `doc:full` entry
- **THEN** the full-document display contains a token-informed cost for the full Markdown document
- **THEN** a file-size-only KB summary is not the sole cost signal

#### Scenario: Token cost does not control truncation
- **WHEN** an outline or read result is paginated by `limit_chars`
- **THEN** Markdown adapter still applies the existing character-budget paging behavior
- **THEN** token count remains part of the readable cost summary rather than the paging budget
