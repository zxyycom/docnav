本 delta spec 记录 Markdown direct CLI 跟随 SDK pagination limit 规则的目标；当前只在 `openspec/changes/configure-pagination-defaults/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## MODIFIED Requirements

### Requirement: docnav-markdown direct CLI consumes SDK pagination limit defaults
`docnav-markdown` direct CLI MUST consume SDK-owned pagination config and argv handling for `defaults.pagination.enabled`, `defaults.pagination.limit`, `--pagination enabled|disabled`, and `--limit <n>`.

#### Scenario: Markdown direct CLI uses SDK pagination handling
- **WHEN** Markdown direct CLI runs a paginated document operation
- **THEN** SDK pagination handling resolves the final limit and page before Markdown operation logic runs
- **THEN** Markdown-specific code keeps ownership of Markdown native options

#### Scenario: Markdown config example uses pagination limit
- **WHEN** Markdown config schema or example documents pagination defaults
- **THEN** it uses `defaults.pagination.limit` for the numeric budget default
- **THEN** it does not assign a cross-adapter unit to that limit
