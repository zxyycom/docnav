# contract-validation Specification

## Purpose
Define Docnav contract validation materials and gates: public JSON Schema and examples, runtime JSON field validation, typed-field parity evidence, schema/example drift checks, and validation synchronization when protocol, adapter metadata, diagnostics, output, or field shapes change.

## Requirements
### Requirement: JSON Schema remains public contract and verification material
Public JSON Schema files MUST remain authoritative verification material for documented machine-readable fields. Product semantics stay with the owner specs that define those fields.

#### Scenario: Schema validates an example
- **WHEN** an example is part of the public validation set
- **THEN** schema validation can check its machine-readable shape
- **THEN** semantic ownership still points to the corresponding owner capability

### Requirement: Runtime JSON validation uses typed field metadata where appropriate
Runtime validation MUST preserve owner semantics when it uses typed-field metadata for reusable field-level checks on protocol, manifest, probe, and other JSON surfaces. Semantic validation stays with the owning contract.

#### Scenario: Manifest field type is invalid
- **WHEN** a manifest JSON field has the wrong type
- **THEN** typed-field validation can report the field failure
- **THEN** adapter-contract maps it into the appropriate boundary diagnostic

### Requirement: Runtime validator removal is gated by parity evidence
Removing or replacing a runtime schema validator MUST be gated by parity evidence for the field and semantic constraints the previous validator enforced.

#### Scenario: Validator dependency is removed
- **WHEN** implementation stops using a runtime JSON Schema validator
- **THEN** tests or fixtures prove equivalent rejection classes
- **THEN** schema files remain available as public verification material

### Requirement: Schema and examples sync with owner changes
Changes to machine-readable fields, examples, protocol shapes, diagnostic detail shapes, adapter metadata, output payloads, or config schemas MUST update the corresponding schema and example validation material in the same change.

#### Scenario: Protocol field changes
- **WHEN** a protocol result field is renamed, added, or removed
- **THEN** protocol schema and examples are updated
- **THEN** validation catches stale examples

### Requirement: Validation failures preserve owner attribution
Validation tools MUST report enough path, schema/example, and owner context for maintainers to route a failure to the correct contract owner.

#### Scenario: Example drift
- **WHEN** an example no longer validates
- **THEN** the validation report identifies the example and failing field
- **THEN** the maintainer can determine whether the owner spec or validation material should change
