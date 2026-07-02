## ADDED Requirements

### Requirement: core and navigation MUST consume derived parameter source values without owning adapter source
Core CLI and `docnav-navigation` MUST consume entry parameter source registration and typed runtime values for shared document operation parameters while preserving each entrypoint's owner policy. They MUST NOT treat parameter source resolution output as an adapter implementation source.

#### Scenario: Core consumes derived values for request construction
- **WHEN** a document command has been classified and parameter sources have been resolved
- **THEN** core passes derived operation values to `docnav-navigation`
- **THEN** adapter selection still uses static registry membership and probe result

#### Scenario: Non-document command skips document parameter source resolution
- **WHEN** a caller executes help, version, init, doctor, config without document context, or `adapter list`
- **THEN** the command keeps its owner output boundary
- **THEN** it does not enter document parameter source resolution
