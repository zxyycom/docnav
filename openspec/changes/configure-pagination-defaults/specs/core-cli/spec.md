本 delta spec 记录 core CLI 使用通用 pagination limit 默认值的目标；当前只在 `openspec/changes/configure-pagination-defaults/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## MODIFIED Requirements

### Requirement: Core CLI resolves pagination defaults before adapter invoke
`docnav` document commands MUST resolve pagination defaults into an explicit positive integer `limit` and page before invoking an adapter. Core MUST treat `limit` as an adapter-owned numeric budget and MUST NOT interpret its unit.

#### Scenario: Core passes resolved limit to adapter
- **WHEN** a caller runs a document operation
- **THEN** core resolves pagination enabled state, limit, and page before adapter invoke
- **THEN** the selected adapter receives explicit operation arguments

#### Scenario: Core disables pagination through limit finalization
- **WHEN** effective pagination is disabled
- **THEN** core finalizes the outgoing limit as the maximum representable positive protocol budget
- **THEN** core does not add a separate pagination field to the adapter request
