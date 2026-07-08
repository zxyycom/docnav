本 spec delta 为 `add-outline-preview-skim-pack` 增加 core CLI 行为要求：outline 可以通过显式组合 surface 附带预算内 preview；当前 change 只在 `openspec/changes/add-outline-preview-skim-pack/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: outline preview skim pack composition

Core CLI SHALL expose an explicit outline preview composition control that sequences existing read operations after outline succeeds to attach budgeted preview content for selected outline entries.

#### Scenario: selected entries receive previews
- **WHEN** a caller invokes outline preview composition
- **AND** outline returns entries with non-empty refs
- **THEN** core SHALL select preview candidates by deterministic documented rules
- **AND** core SHALL call the existing read pipeline for selected refs without interpreting adapter-owned ref grammar

#### Scenario: selection is deterministic
- **WHEN** outline preview composition selects entries for preview
- **THEN** the first implementation SHALL use outline result order, configured preview count, and active budget as selection inputs
- **AND** it SHALL NOT rank entries by inferred importance, semantic relevance, or model-generated judgment

#### Scenario: budget stops preview reads
- **WHEN** the preview budget is exhausted before all selected entries are read
- **THEN** core SHALL stop issuing additional preview reads
- **AND** core SHALL preserve the outline result with deterministic skipped or pending preview status

#### Scenario: protocol output does not mix preview payload
- **WHEN** a caller requests `protocol-json` together with outline preview composition
- **THEN** core SHALL NOT append preview read content into the outline protocol result
- **AND** core SHALL either reject the unsupported combination or route to a separately documented protocol contract before implementation is considered complete
