## ADDED Requirements

### Requirement: Standard parameter resolution produces typed values from declared sources
The standard parameter resolver MUST combine declared direct input, project config, user config, and default sources into typed runtime values while preserving source information and passthrough fields for owner-specific handling.

#### Scenario: Unmapped input remains outside standard parameter validation
- **WHEN** an input field is not mapped to a standard parameter identity
- **THEN** the standard parameter resolver does not validate it as a standard parameter
- **THEN** the resolver returns it through the entry policy as retained, discarded, or delegated passthrough
