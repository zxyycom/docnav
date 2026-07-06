# navigation-input-resolution Specification

## Purpose
Define document navigation input resolution: core-supplied command facts, config source descriptors and paths, registry handoff, config source loading, routing input extraction, adapter selection, selected adapter typed-field declarations, source priority, typed validation/extraction, operation argument binding, request construction, pre-dispatch policy, and linked adapter dispatch.

## Requirements
### Requirement: Core hands raw navigation inputs to navigation
Core CLI MUST hand document operation command facts, normalized path facts, config source descriptors/paths, and the static adapter registry to navigation without pre-owning adapter options or operation arguments.

#### Scenario: Outline handoff
- **WHEN** core parses `docnav outline <path>`
- **THEN** it identifies the operation and path facts
- **THEN** navigation receives the raw navigation input package for resolution

### Requirement: Navigation loads config sources with origin-aware absence
Navigation MUST load config sources from core-provided descriptor paths and preserve whether each source is explicit, default, absent, present invalid, or present valid.

#### Scenario: Default config is absent
- **WHEN** a default config path does not exist
- **THEN** navigation records the source as absent
- **THEN** resolution may continue to lower-priority sources

#### Scenario: Explicit config is invalid
- **WHEN** an explicit config path exists but cannot be parsed as valid config
- **THEN** navigation reports a present-source failure
- **THEN** fallback behavior remains available only for absent default sources

### Requirement: Navigation selects adapter before adapter parameter extraction
Navigation MUST select the adapter using routing inputs and registry facts before extracting selected adapter native options.

#### Scenario: Multiple adapters exist
- **WHEN** registry contains multiple candidate adapters
- **THEN** navigation selects the adapter according to selection rules
- **THEN** only the selected adapter's native declarations are used for extraction

### Requirement: Selected adapter declarations own parameter facts
Selected adapter typed-field declarations MUST provide adapter-owned option identity, extraction metadata, defaults, and validation facts used during navigation resolution.

#### Scenario: Selected Markdown adapter
- **WHEN** Markdown is selected
- **THEN** Markdown native option declarations are registered for extraction
- **THEN** non-Markdown option declarations remain outside the selected declaration set

### Requirement: Source priority is explicit over project over user over built-in
Navigation MUST merge input sources in the priority order explicit, project config, user config, then built-in defaults. A narrower source boundary is valid only when its owner capability states the exception.

#### Scenario: Explicit value overrides config
- **WHEN** the same field appears in explicit input and project config
- **THEN** the explicit value wins
- **THEN** source attribution records that explicit source

### Requirement: Config sources are inputs, not semantic owners
Project and user config files MUST be treated as input sources. The semantic owner of each field remains core, navigation, adapter, output, or another declared capability.

#### Scenario: Adapter option in config
- **WHEN** a config file provides a Markdown native option
- **THEN** navigation attributes the source to that config
- **THEN** Markdown remains the owner of the option semantics

### Requirement: Adapter native options are owner-scoped
Navigation MUST validate and extract native options only when they are declared by the selected adapter. Undeclared owner-scoped options MUST fail strictly.

#### Scenario: Unknown native option
- **WHEN** a caller provides an option not declared by the selected adapter
- **THEN** navigation reports a strict input diagnostic
- **THEN** dispatch stops before that option reaches an adapter handler

### Requirement: Request construction consumes typed resolution results
Navigation MUST construct operation arguments and request envelopes from typed resolution results. Raw argv strings, raw config JSON, and display output are inputs to earlier owners, not request-construction sources.

#### Scenario: Read request
- **WHEN** typed resolution produces document path, ref, page, and limit
- **THEN** navigation constructs read operation arguments
- **THEN** adapter dispatch receives typed operation input

### Requirement: Diagnostics preserve owner and source
Navigation diagnostics MUST preserve the owner boundary, input source, field identity, and reason needed for protocol/readable projection.

#### Scenario: Config field fails validation
- **WHEN** a project config value violates a typed field constraint
- **THEN** navigation reports the field and project source
- **THEN** diagnostics/output can project a primary failure

### Requirement: Navigation resolves standard outline mode
Navigation MUST resolve standard outline mode inputs and policy before dispatch, including built-in defaults and owner-scoped overrides.

#### Scenario: Outline mode configured
- **WHEN** config selects an outline mode
- **THEN** navigation resolves the effective mode
- **THEN** the selected policy is available before adapter dispatch

### Requirement: Outline mode rules use deterministic path patterns
Navigation MUST apply deterministic matching and precedence whenever path-pattern rules participate in outline mode selection.

#### Scenario: Multiple rules match
- **WHEN** multiple configured outline mode rules match a document path
- **THEN** navigation applies the deterministic precedence rule
- **THEN** the selected policy is recorded in source attribution

### Requirement: Adapter-scoped cost threshold can trigger unstructured full-read outline
Navigation MUST run the unstructured full-read pre-dispatch check before normal adapter outline whenever the effective policy, selected adapter declaration, and cost threshold all permit that result.

#### Scenario: Threshold permits full read
- **WHEN** the selected adapter declares full-read support
- **AND** navigation determines the full read cost is below the effective threshold
- **THEN** navigation returns the declared unstructured outline result
- **THEN** normal structured outline dispatch is skipped for that request

#### Scenario: Threshold does not permit full read
- **WHEN** the cost threshold is exceeded or support is undeclared
- **THEN** navigation dispatches normal adapter outline

### Requirement: Navigation dispatches linked adapter handlers
After successful input resolution and pre-dispatch checks, navigation MUST dispatch to the selected linked adapter handler and return structured result or diagnostic facts to the owning output/protocol layer.

#### Scenario: Dispatch succeeds
- **WHEN** navigation has prepared typed operation input
- **THEN** it calls the selected adapter handler
- **THEN** it preserves the returned structured result facts for projection
