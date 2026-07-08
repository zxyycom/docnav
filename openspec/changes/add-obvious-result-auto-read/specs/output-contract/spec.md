本 spec delta 为 `add-obvious-result-auto-read` 增加 readable 输出要求：组合输出必须同时表达 base result、auto-read 内容和未展开状态；当前 change 只在 `openspec/changes/add-obvious-result-auto-read/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: readable output for obvious auto-read

Readable-view and readable-json SHALL represent obvious-result auto-read as a typed readable composition payload that includes the base outline/find result, the auto-read expansion status, and any read content produced by the existing read pipeline.

#### Scenario: auto-read succeeds
- **WHEN** obvious-result auto-read reads the single candidate successfully
- **THEN** readable-json SHALL expose both the base operation facts and the read content through the typed readable payload
- **AND** readable-view SHALL render the same facts through repository renderer configuration without inventing protocol fields

#### Scenario: auto-read is skipped
- **WHEN** core decides not to auto-read because the result is ambiguous, missing a ref, paginated, or over budget
- **THEN** readable output SHALL preserve the base operation result
- **AND** readable output SHALL include a deterministic skipped reason and continuation guidance where available

#### Scenario: auto-read expansion fails
- **WHEN** the base outline/find operation succeeds but the additional read returns a diagnostic
- **THEN** readable output SHALL preserve the base operation result
- **AND** readable output SHALL project the read diagnostic as auto-read expansion status rather than replacing the base operation outcome

#### Scenario: protocol-json stays raw
- **WHEN** a caller requests `protocol-json`
- **THEN** output SHALL NOT mix readable composition fields into the protocol envelope
- **AND** any future protocol-visible composition result SHALL be specified and validated separately before implementation
