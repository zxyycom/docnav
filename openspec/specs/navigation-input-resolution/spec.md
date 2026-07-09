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
Navigation MUST select the adapter using routing inputs and registry facts before extracting selected adapter native options. The selected registry entry MUST expose an adapter definition, and navigation MUST consume native option declarations and capability declarations from that selected definition.

#### Scenario: Multiple adapters exist
- **WHEN** registry contains multiple candidate adapters
- **THEN** navigation selects the adapter according to selection rules
- **THEN** only the selected adapter's native declarations are used for extraction
- **THEN** declarations from unselected adapter definitions remain outside the operation field set

#### Scenario: Selected definition provides capability facts
- **WHEN** navigation has selected an adapter
- **THEN** navigation reads optional capability declarations from the selected adapter definition
- **THEN** pre-dispatch policy uses only those declared support facts

#### Scenario: Navigation receives selected definition as the fact source
- **WHEN** core registry returns a selected adapter entry
- **THEN** the selected entry provides the adapter definition used for declaration registration, full-read pre-dispatch, and dispatch
- **THEN** navigation uses definition-provided adapter-owned native option and capability semantics

### Requirement: Selected adapter declarations own parameter facts
Selected adapter typed-field declarations MUST provide adapter-owned option identity, extraction metadata, defaults, validation facts, operation applicability, and internal typed handoff/accessor binding metadata used during navigation resolution. These declarations MUST come from the selected adapter definition. The same declaration MUST drive extraction and handler binding for request construction and dispatch.

#### Scenario: Selected Markdown adapter
- **WHEN** Markdown is selected
- **THEN** Markdown native option declarations are registered for extraction from the selected adapter definition
- **THEN** non-Markdown option declarations remain outside the selected declaration set
- **THEN** resolved Markdown option values can be handed to the Markdown handler as typed native option values through the internal dispatch boundary

#### Scenario: Selected adapter declaration binds typed handoff
- **WHEN** a selected adapter declaration includes typed handoff or accessor binding metadata
- **THEN** navigation uses that metadata after validation and extraction
- **THEN** request construction or dispatch prepares the adapter-specific typed option value for the handler

#### Scenario: Selected declaration binds dispatch
- **WHEN** navigation has validated a selected adapter native option
- **THEN** the handler binding comes from the same selected declaration used for extraction
- **THEN** request construction uses that declaration as the adapter-owned mapping for that option

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
Navigation MUST validate and extract native options only when they are declared by the selected adapter definition. Undeclared owner-scoped options MUST fail strictly. Declared native options MUST be resolved into typed values before dispatch, and handlers receive those typed values or accessors for native-option consumption.

#### Scenario: Unknown native option
- **WHEN** a caller provides an option not declared by the selected adapter
- **THEN** navigation reports a strict input diagnostic
- **THEN** dispatch stops before that option reaches an adapter handler

#### Scenario: Declared native option becomes typed handoff
- **WHEN** a caller provides a declared native option value
- **THEN** navigation validates the value through the selected adapter declaration
- **THEN** navigation records source attribution for diagnostics and logging
- **THEN** the selected handler receives the typed native option value or accessor result

### Requirement: Request construction consumes typed resolution results
Navigation MUST construct operation arguments, request envelopes, and handler-facing adapter input from typed resolution results. Raw argv strings, raw config JSON, and display output are inputs to earlier owners. Request construction or dispatch MUST preserve an internal typed selected-adapter native option handoff/accessor for the operation while protocol output wrappers and external JSON shapes remain under their existing owners.

#### Scenario: Read request
- **WHEN** typed resolution produces document path, ref, page, and limit
- **THEN** navigation constructs read operation arguments
- **THEN** adapter dispatch receives typed operation input

#### Scenario: Operation includes selected adapter options
- **WHEN** typed resolution produces selected adapter native option values
- **THEN** request construction or dispatch binds those values to the selected adapter input through an internal handoff
- **THEN** the protocol/readable output wrapper remains owned by the output and protocol capabilities
- **THEN** raw config JSON is not forwarded as handler input

#### Scenario: Protocol-stable options remain separate from handler input
- **WHEN** protocol request construction includes `OperationArguments.options`
- **THEN** navigation constructs that protocol-stable object from typed resolution results
- **THEN** handler-facing typed native option handoff/accessor remains the dispatch contract for declared typed bindings
- **THEN** protocol output wrapper shape remains separate from adapter handler input typing

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
Navigation MUST run the unstructured full-read pre-dispatch check before normal adapter outline whenever the effective policy, selected adapter full-read capability declaration, and cost threshold all permit that result. Navigation MUST treat support, content, cost measurement, and result facts as parts of the selected adapter's declared full-read capability group.

#### Scenario: Threshold permits full read
- **WHEN** the selected adapter declares full-read capability support
- **AND** navigation determines the full read cost is below the effective threshold
- **THEN** navigation returns the declared unstructured outline result
- **THEN** normal structured outline dispatch is skipped for that request

#### Scenario: Threshold selects normal outline
- **WHEN** the cost threshold is exceeded or support is undeclared
- **THEN** navigation dispatches normal adapter outline

#### Scenario: Full-read capability is partially unsupported
- **WHEN** pre-dispatch policy requires a full-read capability fact outside the selected adapter definition
- **THEN** navigation follows the documented fallback or reports the unsupported boundary according to the owning policy
- **THEN** navigation bases that decision on the selected adapter definition's full-read capability group

### Requirement: Navigation dispatches linked adapter handlers
After successful input resolution and pre-dispatch checks, navigation MUST dispatch to the selected linked adapter handler and return structured result or diagnostic facts to the owning output/protocol layer. Dispatch MUST use the selected adapter definition's operation handler and prepared internal typed native option handoff/accessor for the selected operation.

#### Scenario: Dispatch succeeds
- **WHEN** navigation has prepared typed operation input
- **THEN** it calls the selected adapter handler
- **THEN** it preserves the returned structured result facts for projection

#### Scenario: Dispatch uses selected definition handler
- **WHEN** navigation dispatches a selected operation
- **THEN** the operation handler comes from the selected adapter definition
- **THEN** typed native option values correspond to declarations from the same selected adapter definition

#### Scenario: Dispatch preserves single-definition ownership
- **WHEN** navigation calls a selected operation handler
- **THEN** the handler handle, capability context, and native option handoff all originate from the selected adapter definition
- **THEN** navigation dispatch uses a coherent selected definition for handler handles and native option declarations

### Requirement: Navigation exposes parameter aggregation projections

Navigation MUST participate in a parameter aggregation boundary derived from common navigation typed fields, outline mode config fields, and adapter-id namespaced typed-field declarations. The aggregation MUST preserve processing paths, field identity, owner, adapter id when applicable, value kind, constraints, defaults, current owner-specific shape validation handoff when applicable, and source binding facts, and MUST be able to produce CLI/input and config-source projections without taking ownership of adapter-native option semantics or actual source attribution policy.

#### Scenario: Config-source projection includes common fields

- **WHEN** navigation builds the config-source projection for document operation inputs
- **THEN** metadata for `defaults.pagination.enabled`, `defaults.pagination.limit`, `defaults.output`, and declared outline mode config fields is derived from the same field facts used by navigation resolution
- **THEN** consumers can validate config source values without redefining those field facts

#### Scenario: Config-source projection includes adapter-id options

- **WHEN** navigation builds config-source metadata from the adapter registry
- **THEN** native option declarations are projected under `options.<adapter-id>.<option-key>`
- **THEN** equal option keys from different adapter ids remain distinct config paths

### Requirement: Config source validation uses the config-source projection

Navigation MUST validate config source keys and declared values through the config-source projection before constructing operation arguments when those fields are consumed for the selected operation. Unknown fields, unknown adapter ids, selected-adapter `options.<adapter-id>.*` fields not declared for the selected operation, owner-specific object/array shape failures in the supported subset, and typed validation failures MUST produce blocking diagnostics with config source attribution. Known adapter-id namespaces for adapters other than the selected adapter MAY remain valid source facts but MUST NOT affect selected operation argument construction.

#### Scenario: Config native option fails selected declaration

- **WHEN** a project config file contains `options.docnav-markdown.max_heading_level` with a value outside the Markdown adapter declaration range
- **THEN** navigation reports a blocking typed validation diagnostic for that config source and field
- **THEN** adapter dispatch does not occur

#### Scenario: Config option unsupported by selected adapter

- **WHEN** a user config file contains an `options.docnav-markdown.*` key not declared by the selected Markdown adapter operation
- **THEN** navigation reports an unsupported native option diagnostic for that config source
- **THEN** the raw option value is not forwarded to the adapter handler

#### Scenario: Unknown adapter namespace is blocking

- **WHEN** a config source contains `options.unknown_adapter.max_heading_level`
- **THEN** navigation reports an unknown adapter id diagnostic with config source attribution
- **THEN** adapter dispatch does not occur

### Requirement: Navigation consumes selected adapter namespace

When navigation constructs operation arguments for a selected adapter and operation, it MUST consume native option values from that selected adapter's `options.<adapter-id>.*` namespace and validate them against the selected operation field set. Values stored under other known adapter ids MUST remain separate source facts and MUST NOT be forwarded to the selected adapter handler.

#### Scenario: Selected adapter reads its own namespace

- **WHEN** a config file contains `options.docnav-markdown.max_heading_level` and `options.docnav-other.max_heading_level`
- **AND** navigation selects adapter id `docnav-markdown` for an outline operation
- **THEN** navigation validates and consumes the value under `options.docnav-markdown.max_heading_level`
- **THEN** the value under `options.docnav-other.max_heading_level` is not forwarded to the Markdown adapter handler or used to validate the Markdown selected operation

