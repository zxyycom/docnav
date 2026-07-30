本 delta spec 是 `replace-probe-traversal-with-inferred-routing` 的临时 typed-field 工件：它删除 probe consumer-local field projections，并禁止 private inference/registry outcomes 进入 caller field catalog。

## MODIFIED Requirements

### Requirement: Consumers keep owner semantics

Typed fields MUST provide reusable field identity, processing locator, value-kind, constraint, default, merge-strategy, validation, and typed-value facts. Docnav core MUST own caller-configurable document-operation parameter declarations, standard-input bindings, and the validation rules selected for pre-dispatch execution. Navigation MUST own source loading, resolution orchestration, private format inference/normalization, standard-input construction, and dispatch. Private inference outcomes, normalized routing identities, registry candidates, library confidence, and library errors MUST NOT become typed fields, caller parameters, processing locators, schema metadata, or adapter inputs. Adapters MUST own selected format strategies and MAY validate or repeat validation of standard typed values when algorithmic correctness requires it. Adapter validation MUST NOT contribute field declarations or source-resolution facts. Protocol envelopes, contract-validation gates, output rendering, and diagnostic code identity MUST remain with their dedicated consumer capabilities. When a consumer surface is removed, typed-fields MUST NOT retain that consumer's unreachable field-definition set or processing projection; removing probe therefore removes probe result/reason field definitions without changing the reusable typed-fields core API.

#### Scenario: Navigation consumes the core catalog

- **WHEN** navigation validates candidates for common or adapter-scoped document-operation parameter fields
- **THEN** typed fields provide canonical facts and attributed validation failures
- **THEN** the declarations come from the core-owned catalog
- **THEN** adapters do not inject or override those field facts

#### Scenario: Private routing outcome is produced

- **WHEN** navigation infers and normalizes a document format for registry lookup
- **THEN** the outcome remains private routing state
- **THEN** typed-fields does not declare or project the format identity, library evidence, or registry candidate as caller data

#### Scenario: Probe validation consumer is removed

- **WHEN** protocol/contract-validation deletes the probe result surface
- **THEN** probe result and nested reason `FieldDefSet` consumers are deleted
- **THEN** no dead probe processing path remains in typed-field projections
- **THEN** reusable typed-field construction and validation APIs remain unchanged

#### Scenario: Adapter consumes a prepared value

- **WHEN** core-defined resolution materializes an adapter-scoped value
- **THEN** the selected adapter may use that value in its format algorithm
- **THEN** the selected adapter may validate an additional or repeated semantic precondition before use
- **THEN** typed-fields does not transfer parameter authoring ownership to the adapter

#### Scenario: Core uses minimal reusable validation

- **WHEN** a core-owned field definition performs standard type materialization but leaves a context-dependent rule to the selected strategy
- **THEN** typed-fields returns the standard typed value and provenance
- **THEN** the adapter strategy owns the runtime semantic decision without owning the parameter declaration
