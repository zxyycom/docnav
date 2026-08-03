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

Runtime validation MUST preserve owner semantics when it uses typed-field metadata for reusable field-level checks on protocol, manifest, and other current JSON surfaces. Semantic validation stays with the owning contract. Because probe result is removed from the protocol/adapter contract, contract-validation MUST delete its probe runtime validation entry point, probe decoder hook, consumer-local probe field-definition sets, nested reason validation, and schema-name registration rather than retain an unreachable validator.

#### Scenario: Manifest field type is invalid

- **WHEN** a manifest JSON field has the wrong type
- **THEN** typed-field validation can report the field failure
- **THEN** adapter-contract maps it into the appropriate boundary diagnostic

#### Scenario: Manifest filename hint is invalid

- **WHEN** `formats[].filenames[]` contains an empty value, a path separator, `.` or `..`
- **THEN** manifest runtime validation rejects the descriptor
- **THEN** no routing index is constructed from the invalid hint

#### Scenario: Manifest suffix hint is invalid

- **WHEN** `formats[].extensions[]` contains an empty value, lacks a leading dot, consists only of `.`, or contains a path separator
- **THEN** manifest runtime validation rejects the descriptor
- **THEN** no normalized-suffix lookup view is constructed from the invalid hint

#### Scenario: Manifest compound suffix is valid

- **WHEN** `formats[].extensions[]` contains `.schema.json`
- **THEN** manifest runtime validation accepts the dotted compound suffix
- **THEN** schema and runtime validation preserve it for complete-basename longest-suffix matching

#### Scenario: Removed probe value is presented

- **WHEN** a caller or test presents a former probe-result JSON value after migration
- **THEN** no probe-specific runtime contract validator or decoder exists
- **THEN** contract-validation does not treat the removed value as a current public surface

### Requirement: Runtime validator removal is gated by parity evidence
Removing or replacing a runtime schema validator MUST be gated by parity evidence for the field and semantic constraints the previous validator enforced.

#### Scenario: Validator dependency is removed
- **WHEN** implementation stops using a runtime JSON Schema validator
- **THEN** tests or fixtures prove equivalent rejection classes
- **THEN** schema files remain available as public verification material

### Requirement: Schema and examples sync with owner changes

Changes to machine-readable fields, examples, protocol shapes, diagnostic detail shapes, adapter metadata, output payloads, or config schemas MUST update the corresponding schema and example validation material in the same change. The removed probe surface MUST have no standalone schema, examples, fixtures, schema index entries, runtime validator, or validation tests. Every former probe consumer MUST be absent or migrated to its current owner; removal MUST NOT retain a compatibility validator or inspection-only path.

#### Scenario: Protocol field changes

- **WHEN** a protocol result field is renamed, added, or removed
- **THEN** protocol schema and examples are updated
- **THEN** validation catches stale examples

#### Scenario: Probe surface is removed

- **WHEN** the adapter/protocol owner removes probe result and probe candidate details
- **THEN** probe-result schema, examples, fixtures, runtime validator, typed-field consumer definitions, and conformance references are absent from Current validation materials
- **THEN** manifest schema/examples add `formats[].filenames[]` and express `formats[].extensions[]` as leading-dot basename suffixes that may contain multiple dots
- **THEN** protocol-response schema/examples adopt the exact `FORMAT_UNKNOWN`, registry-conflict, and selected `DOCUMENT_CONTENT_INVALID` details from diagnostics-contract
- **THEN** routing-only `NO_SUPPORTED_ADAPTER`, `FORMAT_MATCH`, and `FORMAT_AMBIGUOUS` candidate examples are absent
- **THEN** no compatibility validator or inspection-only path is retained
- **THEN** every former consumer is absent or migrated to its current owner

### Requirement: Validation failures preserve owner attribution
Validation tools MUST report enough path, schema/example, and owner context for maintainers to route a failure to the correct contract owner.

#### Scenario: Example drift
- **WHEN** an example no longer validates
- **THEN** the validation report identifies the example and failing field
- **THEN** the maintainer can determine whether the owner spec or validation material should change
