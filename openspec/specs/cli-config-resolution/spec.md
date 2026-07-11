# cli-config-resolution Specification

## Purpose

Define the reusable Rust CLI/config resolution contract that maps canonical `FieldDefSet`
declarations across CLI, environment, structured config, defaults, and custom sources into
deterministic, validated typed values with provenance, while keeping framework adapters and
Docnav-specific behavior outside the framework-independent core.

## Requirements

### Requirement: Canonical Standard Parameter Model

The CLI/config resolution library MUST use the existing `docnav-typed-fields` `FieldDef` and `FieldDefSet` model as the canonical standard parameter definition, including identity, value kind, constraints, defaults, merge strategy, validation, set construction checks, and typed materialization support. Each `FieldDef` MUST directly own its `MergeStrategy`, defaulting to `Replace`. The library MAY re-export these types or provide thin `Parameter` / `ParameterSet` aliases, but it MUST NOT require a second field or merge-policy model that copies or associates the same facts by field identity.

#### Scenario: Register canonical parameter definitions once

- **WHEN** a consumer builds a `FieldDefSet` with field types, constraints, defaults, merge strategies, and processing metadata
- **THEN** the same set can be passed to source extractors and resolution without translating it into another field set
- **THEN** field identity, validation rules, default metadata, and merge strategy have one canonical owner

#### Scenario: Expose only resolution-required metadata

- **WHEN** an extractor or resolver inspects a canonical field
- **THEN** it uses public immutable metadata views or validation entry points provided by typed-fields
- **THEN** the resolution crate does not recreate value-kind, constraint, default, merge-strategy, or validation types

#### Scenario: Default merge strategy is replace

- **WHEN** a canonical scalar, list, or map field does not explicitly declare a merge strategy
- **THEN** its `FieldDef` metadata reports `Replace`
- **THEN** no resolution-owned default policy table is required

### Requirement: Explicit Source Extraction Strategies

The canonical parameter model MUST support explicit extraction strategies for CLI flags, environment variables, and structured config paths through its processing metadata. Each strategy MUST map a source-local locator to a stable canonical field identity without owning source priority or application behavior.

#### Scenario: Declare CLI, env, and config locators

- **WHEN** one canonical field declares a CLI flag locator, an environment variable locator, and a config path locator
- **THEN** each source-specific extractor can locate input for the same field identity
- **THEN** the consumer does not repeat the field type, constraints, or default in adapter-specific declarations

#### Scenario: Preserve source locator facts

- **WHEN** an extractor produces a candidate from `--limit`, `APP_LIMIT`, or `read.limit`
- **THEN** the candidate records the canonical field identity, source identity and kind, and the matching source locator
- **THEN** diagnostics and provenance can identify the original input location

### Requirement: Declared Input Extraction Behavior

Source extractors MUST inspect only locators declared by the canonical parameter set. A missing declared input MUST not override another source, and undeclared env or config entries MUST be ignored by default without requiring a general unknown-input policy.

#### Scenario: Missing declared input produces no effective candidate

- **WHEN** a declared CLI flag, env var, or config path is absent from one source
- **THEN** that absence does not override a value from another source
- **THEN** resolution proceeds without requiring an explicit missing candidate for every field and source pair

#### Scenario: Ignore undeclared env and config entries

- **WHEN** an environment or config document contains keys that have no declared extraction strategy
- **THEN** the extractor ignores those keys
- **THEN** the consumer is not required to configure `UnknownPolicy`, unused-key diagnostics, or a full input scan

#### Scenario: Preserve clap unknown-argument behavior

- **WHEN** a consumer builds or augments a clap command from declared CLI strategies and the invocation contains an unregistered flag
- **THEN** clap applies its native unknown-argument rejection
- **THEN** the resolution core does not implement a second unknown-flag policy

### Requirement: Ordered Source Resolution

The CLI/config resolution library MUST resolve candidates from an ordered collection of sources instead of hard-coding application-specific source slots. A larger numeric priority MUST mean higher priority, and for equal priority a source registered later MUST have higher precedence.

#### Scenario: Highest-priority source wins for replace strategy

- **WHEN** the same field has valid candidates from multiple sources using replace strategy
- **THEN** the resolver selects the candidate with the largest source priority
- **THEN** the result records lower-priority candidates as overridden

#### Scenario: Resolve equal-priority sources deterministically

- **WHEN** multiple sources with equal priority provide candidates for one field
- **THEN** the candidate from the source registered later has higher precedence
- **THEN** repeated resolution with the same declarations produces the same value and trace

#### Scenario: Add a custom source

- **WHEN** a consumer registers a custom source with an identity and deterministic priority
- **THEN** its candidates participate in the same ordering rules as CLI, env, config, and default sources
- **THEN** the resolver does not require a new hard-coded source slot

### Requirement: Field-Level Merge Strategy

Each canonical `FieldDef` MUST directly declare one of four public merge strategies: `Replace`, `Append`, `MapMerge`, or `DenyConflict`. `Replace` MUST apply to scalar, list, and map fields and MUST be the default. The resolver MUST only execute the strategy stored in canonical field metadata; it MUST NOT own a separate merge declaration keyed by field identity.

#### Scenario: Replace a scalar, list, or map value

- **WHEN** a scalar, list, or map field uses `Replace` and more than one source provides a candidate
- **THEN** only the candidate with the largest priority, or the later registration at equal priority, is selected
- **THEN** provenance identifies the overridden candidates

#### Scenario: Append list values across sources

- **WHEN** a canonical list field uses `Append` and multiple sources provide list candidates
- **THEN** the resolver appends candidates from lower priority to higher priority
- **THEN** equal-priority candidates are appended in source registration order
- **THEN** provenance records every contributing source

#### Scenario: Merge map values across sources

- **WHEN** a canonical map field uses `MapMerge` and multiple sources provide map candidates
- **THEN** the resolver applies candidates from lower priority to higher priority and equal-priority candidates in source registration order
- **THEN** a value applied later replaces an earlier value for the same map key
- **THEN** provenance records the contributing sources

#### Scenario: Deny conflicting values

- **WHEN** a field uses `DenyConflict` and multiple applicable sources provide incompatible values
- **THEN** resolution reports a conflict diagnostic instead of silently selecting one value
- **THEN** the diagnostic identifies the conflicting source locators

### Requirement: Canonical Validation and Typed Materialization

Candidate decoding, final merged-value validation, and typed materialization MUST reuse canonical `FieldDef` metadata and validation behavior. A decode failure MUST block resolution when the candidate is selected by `Replace` or is required as a contributor by `Append`, `MapMerge`, or `DenyConflict`. A decode failure in a candidate overridden by `Replace` MUST be retained in trace but MUST NOT by itself block a valid selected candidate. Every merged or selected final value MUST be validated again by its canonical `FieldDef` before materialization.

#### Scenario: Selected candidate decode failure blocks replace

- **WHEN** the candidate selected by `Replace` cannot be decoded for its canonical field
- **THEN** resolution for that field fails with source and locator facts
- **THEN** the resolver does not fall back to a lower-precedence candidate

#### Scenario: Contributing candidate decode failure blocks merge

- **WHEN** any candidate required by `Append`, `MapMerge`, or `DenyConflict` cannot be decoded for its canonical field
- **THEN** resolution for that field fails with deterministic field, source, locator, received-value, and decode-reason facts
- **THEN** no partial merge result is returned as successful

#### Scenario: Overridden invalid candidate remains trace-only

- **WHEN** a lower-precedence candidate under `Replace` cannot be decoded and a higher-precedence selected candidate is valid
- **THEN** the invalid overridden candidate and its decode failure are retained in provenance trace
- **THEN** that overridden failure does not by itself block the valid selected result

#### Scenario: Validate the final selected or merged value

- **WHEN** candidate selection or merge produces a final value for a canonical field
- **THEN** the final value is validated again by that field's canonical type and constraints
- **THEN** only a successful canonical validation can proceed to existing `FieldDefSet` typed materialization

#### Scenario: Block materialization on invalid resolution

- **WHEN** required fields are missing or final values fail canonical validation
- **THEN** typed materialization fails with deterministic diagnostics
- **THEN** no partially invalid application object is returned as a successful result

### Requirement: Defaults Remain Canonical Fallbacks

Static defaults declared by canonical fields MUST participate as lowest-priority fallback values without requiring consumers to manually construct default source candidates. Dynamic defaults MAY be supplied through an explicit default source when required by a consumer.

#### Scenario: Static default fills an absent value

- **WHEN** no explicit source provides a field value and the canonical field declares a static default
- **THEN** the resolver materializes that default
- **THEN** provenance identifies the selected value as a default fallback

#### Scenario: Explicit source overrides a static default

- **WHEN** an explicit source provides a valid field value and the canonical field also declares a static default
- **THEN** the explicit source value is selected according to source priority
- **THEN** the default remains a fallback fact rather than an equal explicit source

### Requirement: Provenance Trace

The CLI/config resolution library MUST retain enough provenance to explain selected values, overridden candidates, merge contributors, default fallbacks, invalid source inputs, and missing required fields.

#### Scenario: Explain a selected value

- **WHEN** a consumer asks why a resolved value was selected
- **THEN** the trace reports its canonical field identity, selected source locator, and relevant overridden or contributing candidates
- **THEN** readable explain output is derived from stored trace facts rather than reconstructed from a typed object

#### Scenario: Keep trace model minimal

- **WHEN** source load, explicitness, missing-input, and candidate state can be represented by source presence, candidate presence, or a diagnostic
- **THEN** the public trace uses those existing facts
- **THEN** it does not require parallel public state enums that do not affect resolution behavior

### Requirement: Framework Adapter Boundary

Framework-specific extraction MUST remain outside the framework-independent resolution core. The Cargo workspace MUST provide a clap companion for supported CLI strategies and a structured-config companion for config-path strategies; environment extraction MAY remain in the core because it requires no external framework. The clap companion MUST support string, integer, finite-number, `SetTrue` boolean, repeated-string array, and repeated `key=value` object projections. It MUST reject `ValueKind::Json` CLI projections with `ClapProjectionError::UnsupportedValueKind` and MUST NOT decode a raw CLI string as arbitrary JSON.

#### Scenario: Use clap companion with canonical parameters

- **WHEN** a consumer passes a canonical `FieldDefSet` containing supported CLI strategies to `cli-config-resolution-clap`
- **THEN** the companion can register or read the declared clap arguments and return candidates for canonical field identities
- **THEN** it does not require `FieldContract`, `FieldSet`, or duplicated validation metadata

#### Scenario: Reject arbitrary JSON CLI decoding

- **WHEN** a canonical field combines a CLI flag processing locator with `ValueKind::Json`
- **THEN** `cli-config-resolution-clap` rejects the projection with `ClapProjectionError::UnsupportedValueKind`
- **THEN** it does not interpret the raw CLI string as an arbitrary JSON candidate

#### Scenario: Use structured config companion with canonical parameters

- **WHEN** a consumer passes a structured config document and a canonical `FieldDefSet` containing config-path strategies to the config companion
- **THEN** the companion returns candidates for declared canonical field identities
- **THEN** the resolution core receives them through the same candidate/source model used by CLI and env extraction

#### Scenario: Extract environment variables without ambient global state

- **WHEN** a consumer passes an iterable collection of environment key/value pairs and a canonical parameter set to the env extractor
- **THEN** the extractor reads only declared environment locators
- **THEN** tests and consumers can supply input without requiring the extractor to own process-global environment access

### Requirement: Docnav Hard Cutover Boundary

Docnav MUST consume the canonical parameter set through the same extraction and resolution API intended for external consumers, while preserving existing Docnav CLI, config, adapter, protocol, diagnostic, and output behavior without a runtime fallback to the old resolver.

#### Scenario: Remove parallel field-set conversion

- **WHEN** Docnav resolves navigation fields and selected adapter native options
- **THEN** it passes existing canonical typed-fields metadata directly to extractors and resolution
- **THEN** the runtime path does not build a parallel generic field set or copy field type and constraint metadata

#### Scenario: Preserve Docnav source priority

- **WHEN** Docnav maps explicit input, project config, user config, and built-in defaults into ordered sources
- **THEN** resolution matches Docnav's documented priority behavior
- **THEN** Docnav remains the owner of adapter applicability, request construction, diagnostic-code mapping, and output projection

#### Scenario: Map pre-parsed input through a consumer-owned source adapter

- **WHEN** Docnav already holds parsed direct or native CLI input before resolution
- **THEN** a Docnav-private source adapter may traverse canonical processing metadata and emit public `Source` / `SourceCandidate` values
- **THEN** the adapter does not maintain a parallel locator table or copy field type, constraint, default, validation, or merge metadata

#### Scenario: Complete cutover removes compatibility paths

- **WHEN** the canonical resolution path passes Docnav equivalence tests
- **THEN** old resolver paths, runtime feature flags, fallback switches, and field-model compatibility wrappers are absent from the runtime command path
- **THEN** rollback requires reverting the code change rather than toggling a runtime fallback

### Requirement: Independent Cargo Workspace Repository

The reusable library MUST be organized as an independently checkoutable Cargo workspace repository. The workspace MAY contain typed-fields, typed-fields macros, resolution core, clap companion, and structured-config companion packages, and `cli-config-resolution` MUST be the primary consumer entry that re-exports the canonical parameter types.

#### Scenario: Build the independent workspace

- **WHEN** the sub-repository is checked out without the Docnav workspace
- **THEN** its package metadata, libraries, tests, documentation, and end-to-end example build and run from its own workspace root
- **THEN** no package requires Docnav protocol, adapter contracts, navigation, output, or Markdown adapter crates

#### Scenario: Consume the workspace from Docnav

- **WHEN** Docnav depends on the independent workspace packages
- **THEN** Docnav resolves canonical parameters through `cli-config-resolution` and its companions
- **THEN** repository placement or dependency-source changes do not alter runtime parameter semantics

#### Scenario: Keep external publication separately approved

- **WHEN** the independent Cargo workspace is complete but no external release has been approved
- **THEN** local or approved dependency-source consumption can still be verified
- **THEN** crates.io publication, license selection, version, and release order remain separate release decisions
