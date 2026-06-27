本 delta spec 记录 token-informed document cost 对共享 Docnav 契约的影响；当前只在 `openspec/changes/use-token-based-document-cost/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Token-informed cost remains adapter-owned
Document operation cost summaries MUST remain adapter-owned readable strings. Core `docnav`, shared protocol helpers, readable output helpers, and non-owning access layers MUST preserve adapter-provided `cost` and display strings without recomputing token counts, parsing tokenizer-specific values, or changing protocol/readable schema field shapes.

#### Scenario: Core preserves token-informed read cost
- **WHEN** a selected adapter returns a read result with `cost` containing a token-informed summary
- **THEN** core `docnav` preserves that `cost` string in protocol-json, readable-json, and readable-view output
- **THEN** core `docnav` does not recalculate the token count from the document path or content
- **THEN** the read result schema continues to treat `cost` as a string

#### Scenario: Pagination remains character-budgeted
- **WHEN** an adapter reports token-informed cost for outline, read, or find output
- **THEN** `limit_chars` continues to be interpreted as the existing Unicode character budget
- **THEN** page calculation and display truncation do not use token count as their budget input

#### Scenario: Cost is not a machine parsing contract
- **WHEN** a readable consumer needs a machine-stable value
- **THEN** the consumer cannot derive new required protocol behavior from the human-readable `cost` string
- **THEN** schema validation continues to assert field presence and type rather than a tokenizer-specific text format
