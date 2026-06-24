## ADDED Requirements

### Requirement: Typed field definitions expose reusable field metadata
Typed field definitions MUST describe typed field identity, JSON path, value constraint metadata, and validation error attribution without owning full JSON Schema file generation.

#### Scenario: Metadata is available without schema generation ownership
- **WHEN** a consumer registers a typed field definition
- **THEN** the definition exposes schema metadata for type, path, requiredness, defaults, enum or range constraints where applicable
- **THEN** the typed field layer does not claim ownership of writing complete public schema files
