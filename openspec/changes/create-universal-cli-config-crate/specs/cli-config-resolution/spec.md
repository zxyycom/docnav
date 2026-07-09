本 spec delta 是 `cli-config-resolution` 的 change-local 新增能力规范：定义通用 Rust CLI/config resolution crate 必须提供的字段契约、来源投影、合并、追踪和子仓库化边界；当前文档只存在于 `openspec/changes/create-universal-cli-config-crate/`，不影响主规范及现有其它文档。

## ADDED Requirements

### Requirement: Field Contract Projection

The CLI/config resolution library MUST define reusable field contracts with stable identity, value kind, constraints, default metadata, projection metadata, and validation failure facts without owning application-specific CLI commands, config file layout, protocol envelopes, adapter semantics, and diagnostic code identity.

#### Scenario: Build reusable field contract

- **WHEN** a consumer registers a field for resolution
- **THEN** the library records the field identity, value kind, constraints, default metadata, and projection metadata
- **THEN** the consumer remains responsible for mapping that field into application-specific public behavior

#### Scenario: Reject duplicate field identity

- **WHEN** two registered fields use the same stable identity in one field set
- **THEN** field set construction fails deterministically
- **THEN** no resolver run receives ambiguous field facts

### Requirement: Source Projection Model

The CLI/config resolution library MUST model CLI flags, environment variables, config document paths, static defaults, dynamic defaults, and custom inputs as source projections that map source-local locators to stable field identities.

#### Scenario: Extract from multiple source kinds

- **WHEN** a field declares projections for a CLI flag, an environment variable, and a config path
- **THEN** each source extractor can produce a candidate value for the same field identity
- **THEN** resolver behavior is based on field identity and source policy rather than source-local names

#### Scenario: Preserve source locator

- **WHEN** a source extractor produces a candidate value
- **THEN** the candidate records the source id, source kind, and locator such as `--limit`, `APP_LIMIT`, and `read.limit`
- **THEN** diagnostics and explain output can identify where the candidate came from

### Requirement: Ordered Source Resolution

The CLI/config resolution library MUST resolve values from an ordered collection of source specs instead of hard-coding application-specific source slots.

#### Scenario: Highest priority source wins for replace strategy

- **WHEN** the same scalar field appears in multiple sources using replace strategy
- **THEN** the resolver selects the candidate from the highest-priority applicable source
- **THEN** the result records lower-priority candidates as overridden

#### Scenario: Custom source participates in resolution

- **WHEN** a consumer registers a custom source with a deterministic priority
- **THEN** that source participates in resolution according to the same ordering rules as built-in CLI, env, config, and default sources
- **THEN** the resolver does not require code changes for the new source kind

### Requirement: Field-Level Merge Strategy

The CLI/config resolution library MUST allow field-level merge strategy so scalar, list, map, optional, and conflict-sensitive fields can resolve according to their declared semantics.

#### Scenario: Append list values across sources

- **WHEN** a list field declares append merge strategy and multiple sources provide list candidates
- **THEN** the resolver combines candidates in deterministic source order
- **THEN** the trace records every source that contributed to the final list

#### Scenario: Deny conflicting values

- **WHEN** a field declares deny-conflict strategy and more than one applicable source provides incompatible values
- **THEN** resolution reports a diagnostic instead of silently selecting one value
- **THEN** the diagnostic identifies the conflicting source locators

### Requirement: Defaults Remain Fallback Sources

The CLI/config resolution library MUST treat static and dynamic defaults as fallback candidates that apply only when explicit higher-priority sources do not provide an applicable value, unless a field's merge strategy explicitly includes defaults.

#### Scenario: Default fills absent value

- **WHEN** no non-default source provides an optional field value and a static default is declared
- **THEN** the resolver materializes the default value
- **THEN** the trace identifies the selected source as default

#### Scenario: Explicit source overrides default

- **WHEN** a non-default source provides a valid value for a field with a static default
- **THEN** the explicit source value is selected according to source priority
- **THEN** the default candidate is recorded as fallback rather than as an equal explicit source

### Requirement: Provenance Trace

The CLI/config resolution library MUST return provenance facts for resolved values, validation diagnostics, overridden candidates, merge contributors, and missing required fields.

#### Scenario: Explain selected value

- **WHEN** a consumer asks why a resolved value was selected
- **THEN** the library can report the selected source locator, selected value, overridden candidates, and default fallback facts
- **THEN** the explanation is derived from stored trace data rather than reconstructed from the final typed struct

#### Scenario: Invalid candidate retains source facts

- **WHEN** a source candidate violates the field's declared type/constraint rules
- **THEN** the resolver reports a validation diagnostic with field identity, source id, source locator, received value kind, and constraint reason
- **THEN** invalid raw input is not materialized into the final typed value

### Requirement: Typed Materialization

The CLI/config resolution library MUST materialize resolved values into typed output while preserving access to the resolution result and diagnostics.

#### Scenario: Materialize final struct

- **WHEN** all required fields resolve successfully and unresolved optional fields are absent
- **THEN** the consumer can materialize an application-owned typed struct from the resolved values
- **THEN** the consumer can still inspect the underlying resolution trace

#### Scenario: Block materialization on diagnostics

- **WHEN** resolution contains diagnostics for missing required fields, failed validation, and mixed failure cases
- **THEN** materialization fails with deterministic diagnostics
- **THEN** no partially invalid application struct is returned as a successful result

### Requirement: Framework Adapter Boundary

The CLI/config resolution library MUST keep framework integrations such as `clap`, env loading, and serde-compatible config parsing behind companion adapter crates, while the core resolver remains independent of any single framework. Derive macro convenience is outside the first implementation slice.

#### Scenario: Use clap companion crate without changing core resolver

- **WHEN** a consumer uses the `cli-config-resolution-clap` companion crate
- **THEN** the companion crate can generate and read CLI arguments from field projections
- **THEN** the core resolver API remains usable without `clap`

#### Scenario: Use config document adapter without changing core resolver

- **WHEN** a consumer uses the serde-compatible config companion crate
- **THEN** the companion crate maps document paths to source candidates
- **THEN** the core resolver receives source candidates through the same source model used by CLI and env sources

### Requirement: Docnav Hard Cutover Boundary

The CLI/config resolution library MUST support Docnav hard cutover through application-owned integration code that preserves existing Docnav CLI, config, adapter, protocol, diagnostic, and output behavior without retaining the old fixed source resolver as a runtime fallback.

#### Scenario: Preserve Docnav source priority

- **WHEN** Docnav maps explicit input, project config, user config, and built-in defaults into the generic source model
- **THEN** the resulting resolution behavior matches Docnav's documented priority order
- **THEN** Docnav remains the owner of navigation-specific diagnostics and protocol projection

#### Scenario: Complete cutover removes old runtime path

- **WHEN** Docnav navigation input resolution is switched to the generic resolver
- **THEN** the old fixed source resolver is not reachable from the runtime command path
- **THEN** rollback requires reverting the code change rather than toggling a runtime fallback

#### Scenario: Keep adapter semantics outside core library

- **WHEN** Docnav resolves selected adapter native options through the generic resolver
- **THEN** the core library validates field values and returns source trace facts
- **THEN** Docnav navigation remains responsible for selected-adapter semantics, operation applicability, handler binding, protocol envelope construction, and output projection

### Requirement: Sub-Repository Readiness

The CLI/config resolution library MUST be structured so its core crate can move to an independent repository without requiring Docnav-specific runtime dependencies.

#### Scenario: Verify independent crate boundary

- **WHEN** the implementation reaches the sub-repository readiness checkpoint
- **THEN** the core crate builds and tests without depending on Docnav protocol, adapter contracts, navigation, output, and Markdown adapter crates
- **THEN** Docnav-specific integration remains in Docnav-owned cutover code

#### Scenario: Preserve release validation evidence

- **WHEN** the crate is prepared for external reuse
- **THEN** package metadata, package boundaries, examples, and release validation tests demonstrate the reusable API surface
- **THEN** no external release artifact is treated as approved until the implementation audit confirms the crate boundary
