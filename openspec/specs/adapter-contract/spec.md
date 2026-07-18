# adapter-contract Specification

## Purpose
Define linked adapter interface boundaries: static descriptors, manifest/probe metadata, adapter-owned native option declarations, operation handler inputs, structured operation results, adapter diagnostics, and optional full-read support hooks. `protocol-contract` owns raw envelopes; `output-contract` owns public output rendering.
## Requirements
### Requirement: Linked adapter handlers receive prepared operation input

Linked adapter strategy functions MUST receive one core-prepared, operation-specific closed typed input after adapter selection, source resolution, merge/default handling, standard type materialization, request binding, and configured core validation have completed. The existing shared operation contract MUST define its Rust types so navigation and adapters can share the boundary, while core-owned bindings MUST populate every strategy-visible value through compile-time fields, typed accessors, or closed enum variants. Shared placement MUST NOT transfer product parameter ownership away from core. “Prepared” means the strategy does not process raw sources or parameter declarations; it MUST NOT imply that every adapter-specific semantic precondition has already been checked. Protocol envelopes, serialized options, generic parameter lookup, raw source material, parameter declarations, and source-priority metadata MUST remain outside the strategy data boundary.

#### Scenario: Strategy receives outline input

- **WHEN** navigation dispatches an outline operation to a selected adapter
- **THEN** the strategy receives the normalized document path and typed outline arguments
- **THEN** applicable core-defined adapter-scoped values are already present in prepared operation input
- **THEN** raw source parsing, source resolution, default handling, and standard type materialization are complete
- **THEN** the strategy does not query a generic parameter bag or protocol request for those values
- **THEN** the strategy may still validate or revalidate adapter-specific semantics

#### Scenario: Input cannot be standardized

- **WHEN** caller input cannot be decoded, merged, defaulted, or materialized as the standard operation input type
- **THEN** navigation or the owning input boundary reports the diagnostic
- **THEN** the linked adapter strategy is not invoked with the malformed raw value

#### Scenario: Strategy consumes a core-defined format parameter

- **WHEN** core defines `max_heading_level` for Markdown outline/find and resolution succeeds
- **THEN** the Markdown strategy receives the prepared integer through a compile-time operation-input field or typed accessor
- **THEN** source priority, merge, default, and binding work are already complete
- **THEN** the strategy may validate or repeat the range check before using the integer

#### Scenario: Strategy rejects a semantic failure

- **WHEN** a standard typed value satisfies core materialization but violates an adapter algorithm precondition not guaranteed by core validation
- **THEN** the selected strategy validates the value before using it
- **THEN** it returns a standard diagnostic through the adapter contract

### Requirement: Adapter definition owns registry-facing adapter facts

Adapter definition, manifest, probe, and descriptor metadata MUST describe adapter identity, supported format facts, capability declarations, and the linked strategy implementation. The adapter definition MUST be the registry-facing aggregation point for those adapter behavior facts. The fixed adapter strategy interface MUST provide outline, read, find, and info functions. Adapter-private helpers MAY construct manifest or capability values, but shared layers MUST consume adapter behavior facts through the exported definition/factory. Adapter implementation source MUST remain a core static-registry fact. Caller-configurable document-operation parameter facts MUST come from the separate core catalog.

#### Scenario: Core lists built-in adapters

- **WHEN** `docnav adapter list` inspects adapters
- **THEN** implementation source comes from the core static registry
- **THEN** manifest/probe metadata describes adapter capability and format support
- **THEN** document-operation parameter facts remain in the separate core catalog

#### Scenario: Registry consumes one adapter definition

- **WHEN** a built-in adapter is registered with core
- **THEN** the registry receives one adapter definition containing identity, format descriptors, a linked strategy implementation, and optional capabilities
- **THEN** the fixed strategy interface provides the required operations
- **THEN** caller-configurable parameter facts come from the core catalog

#### Scenario: Adapter implementation uses private helpers

- **WHEN** an adapter implementation splits definition construction across private helper functions or modules
- **THEN** it exports one registry-facing definition or definition factory
- **THEN** registry, navigation, and dispatch consume adapter-owned behavior facts through that definition
- **THEN** core catalog remains the only parameter-definition input

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

### Requirement: Adapter handlers remain downstream of typed validation

Value decoding, nullability required for materialization, default, source-precedence, merge-strategy handling, and standard type materialization for caller-configurable document-operation parameters MUST complete before adapter dispatch. Core catalog MAY also require context-independent enum, range, shape, or other validation before dispatch. Adapter strategy functions MUST receive only the closed operation-specific standard typed input as caller data and MAY validate or repeat validation of adapter-specific semantics. Accepted parameters, source locators, defaults, merge rules, standard-input bindings, and declaration metadata MUST remain owned by the core catalog.

#### Scenario: Invalid config value cannot be materialized

- **WHEN** a config source provides a value that cannot be materialized as the core-defined standard type
- **THEN** navigation or the consuming input boundary reports the diagnostic before adapter dispatch
- **THEN** the adapter strategy is not invoked with the malformed raw value

#### Scenario: Core defers adapter semantics

- **WHEN** core performs only structural or minimal validation for an adapter-scoped value
- **THEN** the selected strategy receives the well-typed standard value
- **THEN** the strategy validates every additional precondition required by its algorithm

#### Scenario: Core and adapter repeat a rule

- **WHEN** both core and the selected strategy check the same adapter-scoped constraint
- **THEN** both checks accept the same value domain
- **THEN** either rejection maps to a compatible observable diagnostic

