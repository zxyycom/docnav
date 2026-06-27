# standard-parameter-adoption Specification

## Purpose
Define the adoption requirement for migrating core CLI and adapter SDK entrypoints onto shared standard parameter registration and typed runtime values without changing their existing ownership boundaries.
## Requirements
### Requirement: Core and adapter SDK consume standard parameter resolution
Core CLI and adapter SDK entrypoints MUST consume standard parameter registration and typed runtime values for standard parameters while preserving each entrypoint's existing ownership boundary.

#### Scenario: Direct CLI compatibility is preserved during migration
- **WHEN** a direct CLI invocation contains unknown argv, extra positional input, or a known flag unused by the selected operation
- **THEN** the migrated entrypoint preserves the compatible warning behavior
- **THEN** parameters actually consumed by the selected operation are strictly validated through the standard parameter result
