## MODIFIED Requirements

### Requirement: Core CLI resolves pagination defaults before adapter invoke
`docnav` document commands MUST resolve `defaults.pagination.enabled`, `defaults.pagination.limit`, `--pagination enabled|disabled`, and `--limit <n>` into an explicit positive integer `limit` and `page` before invoking an adapter. Core MUST treat `limit` as an adapter-owned numeric budget and MUST NOT interpret its unit.

#### Scenario: Core resolves pagination sources
- **WHEN** a caller runs a document operation
- **THEN** core maps pagination argv, project config, user config, and built-in defaults to the same standard parameter identities
- **THEN** direct input overrides project config, project config overrides user config, and user config overrides built-in defaults

#### Scenario: Core passes resolved limit to adapter
- **WHEN** core has resolved effective pagination enabled state, limit, and page
- **THEN** the selected adapter receives explicit operation arguments
- **THEN** the outgoing request contains `limit` and `page` rather than a protocol `pagination` field

#### Scenario: Core disables pagination through limit finalization
- **WHEN** effective pagination is disabled
- **THEN** core finalizes the outgoing limit as the configured maximum positive protocol budget
- **THEN** core does not add a separate pagination field to the adapter request

#### Scenario: Core keeps page outside configuration defaults
- **WHEN** a caller omits `page`
- **THEN** core resolves `page` to `1`
- **THEN** project and user config do not provide `defaults.page` or `defaults.pagination.page`
