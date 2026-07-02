## ADDED Requirements

### Requirement: adapter list MUST inspect only the core release static registry
`docnav adapter list` MUST output metadata for adapter layer implementations compiled into the current core release static registry. It MUST NOT read user-level installation registries, project-level adapter policy registries, managed artifact records, external executable paths, command paths, or fingerprints.

#### Scenario: Static registry inspection
- **WHEN** a caller executes `docnav adapter list`
- **THEN** output includes adapter id, version, supported formats, and capabilities for built-in adapter layer implementations
- **THEN** no historical adapter registration file or installed adapter artifact is read

### Requirement: dynamic adapter management commands MUST NOT be default CLI commands
`docnav adapter install`, `docnav adapter register`, `docnav adapter update`, and `docnav adapter remove` MUST NOT be valid default CLI commands.

#### Scenario: Dynamic management command is rejected
- **WHEN** a caller executes `docnav adapter install markdown`
- **THEN** the command fails as an unsupported adapter command
- **THEN** no adapter installation registry, artifact record, executable path, or fingerprint is created
