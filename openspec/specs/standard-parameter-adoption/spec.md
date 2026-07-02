# standard-parameter-adoption Specification

## Purpose
Define the adoption requirement for core document entrypoints, navigation dispatch and source-level native option registry entries to consume shared standard parameter registration and runtime values without changing owner boundaries.
## Requirements
### Requirement: Core and linked adapter dispatch consume standard parameter resolution
Core CLI and protocol-facing document entrypoints MUST consume standard parameter registration and typed runtime values before dispatching to linked adapter handlers. Adapter-owned native options MAY participate through source-level static native option registry entries; core resolves sources, merges values and hands them to adapter handlers, while adapter handlers own support, type and range validation.

#### Scenario: Core input boundary is preserved during adoption
- **WHEN** a core document invocation contains unknown argv, extra positional input, or a known flag unused by the selected operation
- **THEN** core returns the same blocking input diagnostic as the owner entrypoint
- **THEN** parameters actually consumed by the selected operation are strictly validated through the standard parameter result

#### Scenario: Source-level registry supplies native option handoff
- **WHEN** the source-level native option registry contains the Markdown `options.max_heading_level` entry
- **AND** a config or request source provides that option
- **THEN** standard parameter resolution preserves owner, namespace, key, type variant and source metadata while merging the final option value
- **THEN** linked adapter dispatch receives the final option value and the Markdown handler validates consumed option semantics
