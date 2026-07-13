## MODIFIED Requirements

### Requirement: Native options are adapter-owned declarations

Format-native options MUST be declared by the owning adapter definition and consumed by navigation as owner-scoped fields. Each declaration MUST provide canonical field identity、value kind、constraints、default、processing locators、operation applicability and handler binding.

When an option is exposed on the public CLI, the adapter's project-specific field builder MUST attach Docnav CLI extension metadata to the underlying canonical field declaration. That extension MUST contain only presentation facts absent from canonical metadata: help prose、value name、display order and Boolean encoding when applicable. Accepted values and default display MUST derive from canonical metadata; the builder MUST NOT create a second semantic option model.

#### Scenario: Declaration drives the native option lifecycle

- **WHEN** an adapter registers a native option for a document operation through the project field builder
- **THEN** CLI presentation、candidate identity、selected validation and handler binding derive from that declaration
- **THEN** downstream consumers receive those facts through field projections

#### Scenario: Help reuses canonical semantics

- **WHEN** an option extension declares help/value name and the canonical field declares enum/default metadata
- **THEN** generated help combines the project extension with canonical accepted values and default display
- **THEN** the adapter does not repeat those facts in CLI-only metadata

#### Scenario: Boolean encoding is explicit

- **WHEN** an adapter exposes a Boolean native option on the CLI
- **THEN** its project extension selects a valueless switch or declared value tokens
- **THEN** candidate extraction produces a Boolean value or field-local invalid fact without arbitrary JSON guessing
