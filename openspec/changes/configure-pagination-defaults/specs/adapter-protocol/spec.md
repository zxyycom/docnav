本 delta spec 记录 adapter SDK direct CLI 使用通用 pagination limit 参数来源模型的目标；当前只在 `openspec/changes/configure-pagination-defaults/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Adapter SDK direct CLI supports generic pagination limit sources
`docnav-adapter-sdk` direct CLI MUST support `defaults.pagination.enabled` and `defaults.pagination.limit` as generic pagination parameter sources. SDK MUST validate the limit as a positive integer and MUST leave unit interpretation to the adapter.

#### Scenario: SDK maps config and argv to pagination sources
- **WHEN** direct CLI config or argv provides pagination values
- **THEN** SDK maps them to a common pagination parameter source model
- **THEN** operation construction receives finalized operation arguments rather than config-source details

#### Scenario: SDK invoke path ignores direct CLI config
- **WHEN** adapter `invoke` receives stdin protocol JSON
- **THEN** SDK validates that request as explicit protocol input
- **THEN** SDK does not read direct CLI pagination config for that invoke request
