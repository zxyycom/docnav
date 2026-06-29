## ADDED Requirements

### Requirement: Runtime JSON contract field validation uses typed-field metadata
Manifest, probe, protocol request, and protocol response runtime validation MUST use `docnav-typed-fields` for field-level extraction and validation while preserving contract-owned semantic validation.

#### Scenario: Runtime decode uses typed-field validation
- **WHEN** manifest, probe, protocol request, or protocol response JSON is decoded through typed JSON contract validation
- **THEN** field-level path, presence, type, enum, range, length, and pattern constraints are validated through typed-field definitions
- **THEN** cross-field rules, protocol envelope rules, operation/result pairing, and diagnostic details are validated through semantic validation
- **THEN** runtime decode does not invoke public JSON Schema files through a generic schema validator

### Requirement: Runtime schema validator removal is gated by parity evidence
Removing or moving generic JSON Schema validation out of the production decode path MUST require project-owned tests that prove representative field-level and surface-level schema-backed failures.

#### Scenario: Runtime validator removal requires project-owned proof
- **WHEN** generic JSON Schema runtime validation is removed or moved out of the production decode path
- **THEN** typed-field core tests cover representative field-level validation equivalence classes
- **THEN** manifest, probe, protocol request, and protocol response parity tests cover representative schema-backed failures
- **THEN** the parity evidence covers failure stage, error category, field path, stdout/stderr placement, and protocol envelope behavior where those are observable

### Requirement: JSON Schema remains contract and verification material
Public JSON Schema files MUST remain stable contract material for examples, fixtures, CI drift checks, and third-party alignment.

#### Scenario: JSON Schema remains contract and verification material
- **WHEN** examples, fixtures, CI drift checks, or third-party contract alignment are validated
- **THEN** public JSON Schema files remain the stable validation material
- **THEN** any schema validator used for those checks is outside the production runtime decode dependency path
