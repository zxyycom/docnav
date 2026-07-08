本 spec delta 为 `add-outline-preview-skim-pack` 增加 readable 输出要求：outline preview 必须清楚表达结构、预览内容、未预览原因和 continuation；当前 change 只在 `openspec/changes/add-outline-preview-skim-pack/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: readable output for outline preview skim pack

Readable-view and readable-json SHALL represent outline preview composition as a typed readable payload that includes outline entries, preview blocks produced by read, preview status for skipped or failed entries, and continuation guidance.

#### Scenario: preview succeeds
- **WHEN** outline preview composition reads one or more selected entries successfully
- **THEN** readable-json SHALL expose the outline facts and preview content through the typed readable payload
- **AND** readable-view SHALL render the same facts through repository renderer configuration without inventing protocol fields

#### Scenario: preview is partially skipped
- **WHEN** some outline entries are not previewed because of budget, missing refs, pagination, or preview count limits
- **THEN** readable output SHALL preserve those outline entries
- **AND** readable output SHALL include deterministic skipped reasons or continuation guidance where available

#### Scenario: preview read fails
- **WHEN** outline succeeds but an additional preview read returns a diagnostic
- **THEN** readable output SHALL preserve the outline result
- **AND** readable output SHALL project the read diagnostic as preview status for that entry rather than replacing the outline outcome

#### Scenario: protocol-json stays raw
- **WHEN** a caller requests `protocol-json`
- **THEN** output SHALL NOT mix preview composition fields into the protocol envelope
- **AND** any future protocol-visible preview result SHALL be specified and validated separately before implementation
