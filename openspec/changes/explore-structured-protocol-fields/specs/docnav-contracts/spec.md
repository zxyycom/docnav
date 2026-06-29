本 delta spec 记录 raw protocol 字段结构化探索的长期契约方向；当前只在 `openspec/changes/explore-structured-protocol-fields/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Protocol field restructuring starts from structured-field audit
Docnav protocol field restructuring MUST start with an audit that separates machine-readable protocol fields from readable output aggregation. The audit MUST identify field ownership before proposing schema or implementation changes.

#### Scenario: Audit separates protocol and readable concerns
- **WHEN** a protocol field currently carries machine-relevant information as readable text
- **THEN** the audit records whether that information should become raw protocol structure, adapter-owned metadata, or readable-only presentation
- **THEN** subsequent implementation changes use that ownership decision as input
