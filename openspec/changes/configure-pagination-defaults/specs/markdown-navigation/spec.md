## MODIFIED Requirements

### Requirement: docnav-markdown direct CLI consumes SDK pagination limit defaults
`docnav-markdown` direct CLI MUST consume SDK-owned pagination config and argv handling for `defaults.pagination.enabled`, `defaults.pagination.limit`, `--pagination enabled|disabled`, and `--limit <n>`. Markdown-specific code MUST keep ownership of Markdown native options and Markdown's adapter-specific interpretation of `limit`.

#### Scenario: Markdown direct CLI uses SDK pagination handling
- **WHEN** Markdown direct CLI runs a paginated document operation
- **THEN** SDK pagination handling resolves the final limit and page before Markdown operation logic runs
- **THEN** Markdown-specific code keeps ownership of Markdown native options

#### Scenario: Markdown config example uses pagination limit
- **WHEN** Markdown config schema or example documents pagination defaults
- **THEN** it uses `defaults.pagination.enabled` for the default pagination state
- **THEN** it uses `defaults.pagination.limit` for the numeric budget default
- **THEN** it does not describe that budget as a core or SDK unit
- **THEN** any Markdown-specific unit description remains owned by the Markdown adapter documentation
