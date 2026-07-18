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

Typed fields MUST provide reusable field identity, processing locator, value-kind, constraint, default, merge-strategy, validation, and typed-value facts. Docnav core MUST own caller-configurable document-operation parameter declarations, standard-input bindings, and the validation rules selected for pre-dispatch execution. Navigation MUST own source loading, resolution orchestration, standard-input construction, and dispatch. Adapters MUST own format strategies and MAY validate or repeat validation of standard typed values when algorithmic correctness requires it. Adapter validation MUST NOT contribute field declarations or source-resolution facts. Protocol envelopes, contract-validation gates, output rendering, and diagnostic code identity MUST remain with their dedicated consumer capabilities.

#### Scenario: Navigation consumes the core catalog

- **WHEN** navigation validates candidates for common or adapter-scoped document-operation parameter fields
- **THEN** typed fields provide canonical facts and attributed validation failures
- **THEN** the declarations come from the core-owned catalog
- **THEN** adapters do not inject or override those field facts

#### Scenario: Adapter consumes a prepared value

- **WHEN** core-defined resolution materializes an adapter-scoped value
- **THEN** the selected adapter may use that value in its format algorithm
- **THEN** the selected adapter may validate an additional or repeated semantic precondition before use
- **THEN** typed-fields does not transfer parameter authoring ownership to the adapter

#### Scenario: Core uses minimal reusable validation

- **WHEN** a core-owned field definition performs standard type materialization but leaves a context-dependent rule to the selected strategy
- **THEN** typed-fields returns the standard typed value and provenance
- **THEN** the adapter strategy owns the runtime semantic decision without owning the parameter declaration

### Requirement: Definition sets provide typed-field projections

Definition sets MUST provide extraction, validation, default, documentation, schema, and processing projections from one canonical field-facts representation. A processing projection MAY add processing id, input kind, locator, and source path context, but it MUST reference or compose the canonical identity, value kind, constraints, default, merge strategy, and validation behavior rather than store a second independently constructed copy keyed by field identity. Consumer-specific policy MUST remain outside the typed field core.

#### Scenario: Consumer builds a processing projection

- **WHEN** a consumer requests processing metadata for a canonical field
- **THEN** the projection adds only processing-specific locator and input-kind context
- **THEN** type, constraints, default, merge strategy, and validation derive from the same canonical field facts used by schema and materialization

#### Scenario: Projection cannot drift

- **WHEN** a canonical field fact changes during construction
- **THEN** schema, extraction, validation, defaults, resolution, and materialization observe the same fact
- **THEN** no parallel processing metadata record can retain a conflicting copied value

### Requirement: Definition sets expose processing-path validation metadata

Definition sets MUST expose a deterministic projection that lets consumers find field metadata by processing id and structured source path. The projection MUST include field identity, source path, value kind, constraints, nullability/presence, defaults when declared, and enough declaration metadata for the consumer to validate a candidate JSON source value for declared scalar or simple value fields. When a current config-source owner elects to express an array/object shape through typed-fields, the projection MUST also preserve the current subset's compound node kind, object member metadata, array item metadata, and nested source path facts. Duplicate or incompatible declarations for the same processing id and structured path MUST be rejected or surfaced as deterministic projection errors; consumers MUST NOT have to choose between competing field facts.

#### Scenario: Consumer finds config field metadata

- **WHEN** a consumer asks a definition set for metadata at processing id `config` and path `defaults.pagination.limit`
- **THEN** the definition set returns the matching field metadata when that path is declared
- **THEN** the consumer can validate a candidate JSON value without reconstructing the field's value kind, constraints, or containing shape

#### Scenario: Consumer finds nested config-source metadata when declared

- **WHEN** a consumer asks a definition set for metadata at processing id `config` and path `outline.mode_rules[].mode`
- **THEN** the definition set returns metadata for the nested field when the owner has declared that array item object shape through typed-fields
- **THEN** the consumer can report failures against the nested processing path

#### Scenario: Missing path is distinguishable

- **WHEN** a consumer asks a definition set for metadata at an undeclared config path
- **THEN** the definition set reports that no field is declared at that path
- **THEN** the consumer remains responsible for mapping that result into an unknown-field or unsupported-option diagnostic

#### Scenario: Duplicate path is rejected deterministically

- **WHEN** a definition set contains two incompatible declarations for processing id `config` and path `defaults.output`
- **THEN** the projection reports a duplicate or incompatible declaration error
- **THEN** the consumer does not silently choose one declaration for validation

### Requirement: Typed-field validation returns canonical values

Typed-field validation MUST allow consumers to validate a candidate JSON value for a declared scalar field and obtain the canonical typed value, or a validation failure that preserves field identity, processing path, value kind, and constraint reason. When a current config-source owner expresses a compound field through typed-fields, validation MUST support that declared subset and preserve nested path and shape reason in failures. Source-specific coercion and source priority MUST remain consumer-owned before typed-field validation is called.

#### Scenario: Candidate value is canonicalized

- **WHEN** a consumer validates the candidate JSON value `true` accepted by a declared boolean field
- **THEN** typed-field validation returns the canonical boolean typed value
- **THEN** the consumer can serialize the canonical value into its own inspection, config-source, or request representation

#### Scenario: Candidate value fails constraints

- **WHEN** a consumer validates a candidate JSON value that violates a declared numeric range
- **THEN** typed-field validation returns a failure with the field identity, processing path, received value kind, and range reason
- **THEN** the consumer maps the failure into its own diagnostic boundary

#### Scenario: Compound candidate fails nested shape when typed-fields owns that shape

- **WHEN** a consumer validates a candidate JSON array whose item object is missing a required declared member
- **AND** the owner has declared that compound config-source shape through typed-fields
- **THEN** typed-field validation returns a failure with the field identity, processing path, nested item/member path, received value kind, and shape reason
- **THEN** the consumer maps the failure into its own config-source diagnostic boundary

### Requirement: Processing-path projections preserve consumer ownership

Processing-path metadata projections MUST NOT own source priority, CLI flag syntax, config command shape, adapter selection, public diagnostic code identity, protocol envelope shape, output rendering, adapter-id namespace policy, or generic JSON schema policy. Those policies MUST remain with the consuming capability.

Owner-specific array/object validation MAY remain outside typed-fields when that owner can preserve source path, diagnostics, and navigation/config parity without duplicating field semantics in core CLI. Typed-fields MUST NOT be treated as a required replacement for existing outline array validation unless the implementation audit shows the current owner path cannot satisfy the config-source parity target.

#### Scenario: Consumer maps validation result

- **WHEN** typed-fields returns a validation failure for a config processing path
- **THEN** typed-fields provides field-level failure facts
- **THEN** the consuming config or navigation owner chooses the public diagnostic code, source priority behavior, source-specific coercion, and output projection

### Requirement: Public construction surface is direct and bounded

Typed fields MUST support direct `FieldDef` / `FieldDefSet` construction, validation, defaults, merge strategy, and typed materialization. Consumers MUST be able to use typed materialization with only the reusable validation rules selected for that definition set; typed-fields MUST NOT require every downstream domain or algorithm precondition to become a field declaration. Direct builders MUST be the public construction path for consumer-owned sets; internal builder entries MAY remain private implementation details. Contributions to another consumer's set MUST go through that consumer's owned construction boundary.

#### Scenario: Core builds the product catalog directly

- **WHEN** core defines common or adapter-scoped document-operation parameter fields
- **THEN** it uses direct field and definition-set builders
- **THEN** the resulting definitions are owned by the core catalog

#### Scenario: Protocol validates direct field sets

- **WHEN** protocol contract validation builds request, response, manifest, or probe fields
- **THEN** direct builders continue to express declaration paths, expected shape, validation, defaults, and materialization
- **THEN** protocol validation remains independent from the product parameter catalog
