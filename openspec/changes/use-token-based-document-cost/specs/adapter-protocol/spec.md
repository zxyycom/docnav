本 delta spec 记录 linked adapter/shared cost-budget helper 的抽象边界；当前只在 `openspec/changes/use-token-based-document-cost/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Shared adapter helpers expose reusable cost helpers without owning adapter policy
Shared adapter cost-budget helpers MUST provide reusable cost and budget primitives without deciding which cost units an adapter reports or how an adapter formats user-facing cost summaries.

#### Scenario: Adapter keeps cost display policy
- **WHEN** an adapter uses shared cost helpers
- **THEN** the adapter chooses the measurements, ordering, tokenizer, and display policy for its own output
- **THEN** shared helpers provide reusable mechanics rather than adapter-specific policy
