## ADDED Requirements

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

## MODIFIED Requirements

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
