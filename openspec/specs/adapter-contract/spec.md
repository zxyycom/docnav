# adapter-contract Specification

## Purpose
Define linked adapter interface boundaries: static descriptors, manifest/probe metadata, adapter-owned native option declarations, operation handler inputs, structured operation results, adapter diagnostics, and optional full-read support hooks. `protocol-contract` owns raw envelopes; `output-contract` owns public output rendering.
## Requirements
### Requirement: Linked adapter handlers receive prepared operation input
Linked adapter handlers MUST receive operation-specific typed input after core CLI parsing, config source loading, adapter selection, native option extraction, default resolution, request construction, and adapter-specific internal typed native option handoff have completed. Handler inputs MUST be derivable from the selected adapter definition's operation binding and native option declarations. Raw CLI argv, raw config JSON, and untyped native option source values MUST remain inputs to earlier owner boundaries for basic type, requiredness, allowed-value, and range validation.

#### Scenario: Handler receives outline input
- **WHEN** navigation dispatches an outline operation to a selected adapter
- **THEN** the handler receives the normalized document path
- **THEN** it receives typed outline arguments and selected adapter native option values through the internal dispatch boundary
- **THEN** raw CLI argv and raw config file parsing are already complete
- **THEN** basic native option type and range validation has already completed

#### Scenario: Handler receives invalid caller intent
- **WHEN** caller input is invalid before adapter dispatch
- **THEN** navigation or the owning input boundary reports the diagnostic
- **THEN** the linked adapter handler is not invoked for that invalid request

#### Scenario: Handler consumes adapter-specific option accessor
- **WHEN** a selected adapter declares a native option with a typed accessor or typed handoff binding
- **THEN** navigation resolves and validates the declared option before dispatch
- **THEN** the handler consumes the typed value through the adapter-owned accessor or handoff structure
- **THEN** basic JSON type and range validation remains proven by navigation resolution for that option

#### Scenario: Handler receives typed native option binding
- **WHEN** a selected adapter declaration binds a native option such as `max_heading_level` to handler input
- **THEN** the binding is prepared from the selected adapter definition before handler dispatch
- **THEN** the handler receives a typed value or typed accessor for that option
- **THEN** handler correctness is proven through that typed value or typed accessor

### Requirement: Adapter definition owns registry-facing adapter facts
Adapter definition, manifest, probe, and descriptor metadata MUST describe adapter identity, supported format facts, native option declarations, capability declarations, and operation support. The adapter definition MUST be the registry-facing aggregation point and adapter authoring surface for metadata, declarations, full-read capability group, and operation handler handles. Adapter authors MUST expose each adapter-owned fact through one registry-facing definition or definition factory. Adapter-private helpers or modules may construct portions of that definition, but shared layers MUST consume adapter-owned facts only through the exported definition/factory. Adapter implementation source MUST remain a core static-registry fact.

#### Scenario: Core lists built-in adapters
- **WHEN** `docnav adapter list` inspects adapters
- **THEN** implementation source comes from the core static registry
- **THEN** manifest/probe metadata describes adapter capability and format support only
- **THEN** registry-facing adapter metadata is derived from the selected adapter definition

#### Scenario: Registry consumes a single adapter definition
- **WHEN** a built-in adapter is registered with core
- **THEN** the registry receives one adapter definition for that adapter
- **THEN** identity, format descriptors, native option declarations, operation handlers, and optional capability groups are reachable from that definition
- **THEN** the registry uses definition-provided adapter-owned native option and capability semantics

#### Scenario: Adapter author uses one registry-facing definition
- **WHEN** a linked adapter declares identity, format descriptors, native options, required operation handlers, and full-read support/content/cost/facts
- **THEN** those declarations are authored in one adapter definition or definition factory
- **THEN** static registry, adapter inspection, CLI native option discovery, navigation declaration registration, and dispatch consume facts derived from that definition
- **THEN** any transition adapter layer is owned by contract/registry/navigation code and derives from that definition

#### Scenario: Adapter implementation uses private helpers
- **WHEN** an adapter implementation splits definition construction across private helper functions or modules
- **THEN** the adapter exports one registry-facing definition or definition factory
- **THEN** registry, core, CLI, navigation, and dispatch consume adapter-owned facts through that exported definition or factory
- **THEN** private helper boundaries do not become shared-layer declaration inputs

### Requirement: Native options are adapter-owned declarations
Format-native options MUST be declared by the owning adapter in the adapter definition and consumed by navigation input resolution as owner-scoped input sources. Shared layers MUST accept native option input only through selected-adapter declarations. The same declaration MUST provide typed metadata for CLI/config extraction, default resolution, source validation, operation applicability, and adapter-specific internal typed native option handoff or accessor values before handler dispatch while external protocol JSON shape remains owned by the protocol contract.

#### Scenario: Adapter declares a native option
- **WHEN** a Markdown adapter option is registered in the static registry through its adapter definition
- **THEN** navigation can extract and validate that option for Markdown operations
- **THEN** the option applies only to the declaring adapter
- **THEN** the internal dispatch boundary can provide the Markdown handler with a typed option value

#### Scenario: Caller supplies an undeclared option
- **WHEN** caller input contains a native option not declared for the selected adapter
- **THEN** input resolution reports a strict caller-input diagnostic
- **THEN** dispatch stops before forwarding the unknown option

#### Scenario: Native option declaration stays the semantic owner
- **WHEN** project config, user config, or explicit CLI input provides an adapter native option
- **THEN** navigation uses the selected adapter declaration to resolve and validate the value
- **THEN** the adapter declaration remains the source of value kind, default, range, and operation applicability semantics
- **THEN** core and navigation use selected declaration metadata for adapter-owned semantics

#### Scenario: Native option declaration drives every consumer
- **WHEN** a native option declaration exposes CLI, config, default, validation, and handler binding metadata
- **THEN** CLI option discovery, config validation, navigation extraction, and dispatch handoff use that declaration
- **THEN** that declaration is the complete shared source for adapter-owned option semantics

### Requirement: Adapter results preserve format semantics
Adapters MUST return structured operation results or adapter diagnostics that preserve format-owned facts such as refs, content type, parse boundaries, cost facts, and operation-specific item metadata. Core and output layers MUST project those facts without replacing adapter semantics.

#### Scenario: Adapter returns read content
- **WHEN** a linked adapter returns read content with `content_type`
- **THEN** core and output surfaces preserve that content type
- **THEN** display rendering may summarize the content without changing its machine facts

### Requirement: Adapter operation support is explicit
Adapter definitions MUST declare supported document operations and capability groups, including unstructured full-read support, content, cost measurement, and result facts used by navigation pre-dispatch policy. Required operation handler handles and capability group handles MUST be reachable from the same adapter definition. Navigation uses declared support facts when selecting adapter-level capabilities. Capability groups MUST aggregate related hooks under one declared owner boundary.

#### Scenario: Adapter supports unstructured full read
- **WHEN** an adapter declares a full-read capability group
- **THEN** navigation may use that declaration for opt-in full-read pre-dispatch
- **THEN** the adapter still owns the content and cost facts it returns
- **THEN** support, content, cost measurement, and result facts are interpreted within the declared full-read capability boundary

#### Scenario: Capability boundary is unavailable
- **WHEN** policy requires a capability outside the selected adapter definition
- **THEN** navigation reports the unsupported boundary
- **THEN** fallback behavior must come from a declared owner rather than inference

#### Scenario: Full-read capability complements operation handlers
- **WHEN** an adapter declares an optional full-read capability group
- **THEN** the adapter still declares the required `outline`, `read`, `find`, and `info` operation handlers
- **THEN** navigation uses the optional capability only for the policy path that explicitly permits it

### Requirement: Native option declarations provide config validation facts

Adapter native option declarations MUST provide the typed-field facts needed by the parameter aggregation layer to validate config source values for that adapter option, including owner identity, option key, operation applicability, value kind, constraints, defaults when declared, source processing metadata, and operation binding metadata. The registry/navigation aggregation layer combines those declaration facts with the existing adapter registry id to produce adapter-id namespaced persistent config processing paths.

#### Scenario: Adapter option is used for config validation

- **WHEN** a registered adapter declares native option key `max_heading_level` for an operation
- **THEN** parameter aggregation can combine that declaration with the adapter registry id and project the persistent config path to `options.<adapter-id>.max_heading_level`
- **THEN** config source validation can use that declaration to validate a config source value for that adapter-id path
- **THEN** the adapter handler receives only the typed option value after navigation resolution succeeds

### Requirement: Shared config projections keep adapter option keys distinct

Parameter aggregation MUST project adapter-owned persistent config paths with the existing adapter registry id segment and keep `options.<adapter-a>.<key>` distinct from `options.<adapter-b>.<key>`. Equal option keys from different adapters remain separate adapter-owned fields. This migration changes persistent config-source paths only; adapter handler handoff remains operation-specific typed input. When a single adapter exposes incompatible declarations for the same adapter-id config path, shared layers MUST report adapter-local declaration conflict through the consuming CLI/config boundary or require the adapter to expose distinct config keys.

#### Scenario: Same key in different adapters remains distinct

- **WHEN** two adapters declare option key `mode` with different value kind or constraints
- **AND** config inspection or source validation evaluates `options.docnav-markdown.mode`
- **THEN** parameter aggregation uses only the declaration for adapter id `docnav-markdown`
- **THEN** declarations from other adapter ids do not affect that validation

#### Scenario: Adapter-local conflict is rejected

- **WHEN** one adapter declares the same config path `options.docnav-markdown.mode` with incompatible metadata for different operations
- **THEN** parameter aggregation does not choose one declaration implicitly
- **THEN** the consuming config or navigation boundary reports an adapter-local declaration conflict before producing source validation results or dispatching the adapter

### Requirement: Adapter handlers remain downstream of typed validation

Adapter handlers MUST continue to receive operation-specific typed input after config source validation, selected adapter option extraction, and request construction. Adapter handlers MUST NOT be responsible for basic type/range/nullability validation of raw config values that can be expressed in their native option declarations.

#### Scenario: Invalid config option blocks dispatch

- **WHEN** a config source provides an invalid value for a selected adapter native option
- **THEN** navigation or the consuming input boundary reports the validation diagnostic before adapter dispatch
- **THEN** the adapter handler is not invoked with the invalid raw config value

