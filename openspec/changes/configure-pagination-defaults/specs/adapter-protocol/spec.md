## ADDED Requirements

### Requirement: Adapter SDK direct CLI supports generic pagination limit sources
`docnav-adapter-sdk` direct CLI MUST support `defaults.pagination.enabled`, `defaults.pagination.limit`, `--pagination enabled|disabled`, and `--limit <n>` as generic pagination parameter sources. SDK MUST validate the limit as a positive integer and MUST leave unit interpretation to the adapter.

#### Scenario: SDK maps config and argv to pagination sources
- **WHEN** direct CLI config or argv provides pagination values
- **THEN** SDK maps them to a common pagination parameter source model
- **THEN** operation construction receives finalized operation arguments rather than config-source details

#### Scenario: SDK finalizes disabled pagination
- **WHEN** effective direct CLI pagination is disabled
- **THEN** SDK finalizes the operation limit as the maximum representable positive protocol budget
- **THEN** the adapter operation receives `limit` and `page` rather than a pagination enabled flag

#### Scenario: SDK invoke path uses standard parameter resolution
- **WHEN** adapter `invoke` receives stdin protocol JSON
- **THEN** SDK maps the decoded request JSON and `arguments` as direct input
- **THEN** SDK resolves pagination parameters through the same source priority, validation, and finalization rules as other document entrances
- **THEN** SDK does not write resolved config/default values back into the raw stdin request JSON
