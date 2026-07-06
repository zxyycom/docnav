# typed-fields Specification

## Purpose
Define the typed field definition core: reusable field identity, extraction metadata, value kind and constraint metadata, static default metadata, field-level decode/validation attribution, schema metadata projections, and duplicate identity checks. Consumers own CLI/config mapping, navigation input resolution, adapter metadata, protocol envelopes, readable output, and full JSON Schema documents.

## Requirements
### Requirement: Typed fields define reusable field identity and extraction metadata
Typed field definitions MUST provide stable field identity, source path metadata, value kind, extraction strategy, and default metadata without owning the consumer's public behavior.

#### Scenario: Consumer registers fields
- **WHEN** a consumer builds a definition set
- **THEN** each field has stable identity and extraction metadata
- **THEN** the consumer still owns how the extracted value affects its contract

### Requirement: Field validation groups type and field constraints
Typed field validation MUST handle reusable field-level constraints such as presence, type, enum, range, length, and pattern where those constraints are declared by the field owner.

#### Scenario: Invalid field value
- **WHEN** a value violates a declared field constraint
- **THEN** validation reports the field identity
- **THEN** the consumer maps that failure into its diagnostic boundary

### Requirement: Typed field validation reports attributed failures
Validation failures MUST preserve field identity, source path, source owner, and constraint information needed for downstream diagnostics.

#### Scenario: Config value fails validation
- **WHEN** a config source provides an invalid value
- **THEN** validation reports which field and source failed
- **THEN** navigation or the owning consumer projects the appropriate diagnostic

### Requirement: Schema metadata remains a field-level view
Schema metadata generated from typed fields MUST remain a field-level projection. Complete public JSON Schema documents remain owned by contract validation and schema materials.

#### Scenario: Schema metadata requested
- **WHEN** a consumer asks for field schema metadata
- **THEN** typed fields provide the field-level metadata
- **THEN** complete schema document ownership remains with contract-validation and schema materials

### Requirement: Definition sets reject duplicate field identities
Definition sets MUST reject duplicate field identities before a consumer uses them for extraction, validation, or schema projection.

#### Scenario: Duplicate identity
- **WHEN** two registered fields use the same identity
- **THEN** definition-set construction fails
- **THEN** consumers receive unambiguous metadata or a build failure

### Requirement: Consumers keep owner semantics
Typed fields MUST provide reusable field facts while consumer capabilities decide CLI flags, config source priority, adapter native option semantics, protocol envelope shape, output rendering, and diagnostic code identity.

#### Scenario: Navigation consumes typed fields
- **WHEN** navigation uses typed fields to validate input
- **THEN** typed fields provide validation facts
- **THEN** navigation owns source priority, operation binding, and dispatch behavior

### Requirement: Definition sets provide typed-field projections
Definition sets MUST keep consumer ownership intact when they provide projections for extraction, validation, defaults, docs, and schema metadata.

#### Scenario: Consumer builds a projection
- **WHEN** a consumer requests a projection from a definition set
- **THEN** the projection is derived from the same field facts
- **THEN** consumer-specific policy remains outside the typed field core
