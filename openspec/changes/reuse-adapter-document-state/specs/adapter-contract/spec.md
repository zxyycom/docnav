**Interpretation:** This mechanism-neutral Target delta requires an
invocation-private reuse boundary but does not select a handle, prepared-state,
session, shared source, or Rust type. `proposal.md` owns the change status;
`design.md` leaves “approved invocation lifecycle” and “approved document view”
open; tasks 1.7–1.8 must approve and define them before applying this delta.

## MODIFIED Requirements

### Requirement: Linked adapter handlers receive prepared operation input

Linked adapter strategy functions MUST receive one core-prepared, operation-specific closed typed input after adapter selection, source resolution, merge/default handling, standard type materialization, request binding, and configured core validation have completed. The existing shared operation contract MUST define its Rust types so navigation and adapters can share the boundary, while core-owned bindings MUST populate every strategy-visible caller value through compile-time fields, typed accessors, or closed enum variants. Shared placement MUST NOT transfer product parameter ownership away from core. “Prepared” means the strategy does not process raw caller sources or parameter declarations; it MUST NOT imply that every adapter-specific semantic precondition has already been checked or that adapter-private document preparation belongs in the standard input. Protocol envelopes, serialized options, generic parameter lookup, raw caller source material, parameter declarations, source-priority metadata, parser trees, resolved nodes, and reusable-state identifiers MUST remain outside the strategy data boundary. The approved adapter lifecycle MUST allow eligible calls in one navigation invocation to reuse adapter-private document preparation without turning that preparation into a second caller-data argument.

#### Scenario: Strategy receives outline input

- **WHEN** navigation dispatches an outline operation to a selected adapter
- **THEN** the strategy receives the normalized document path and typed outline arguments
- **THEN** applicable core-defined adapter-scoped values are already present in prepared operation input
- **THEN** raw caller source parsing, source resolution, default handling, and standard type materialization are complete
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

#### Scenario: Private preparation is reused

- **WHEN** navigation dispatches an eligible later stage over the same approved document view
- **THEN** the adapter may consume the compatible source view and its existing private decoded/parser/index/source-region facts through the approved lifecycle
- **THEN** the closed operation input still contains only operation-specific caller and strategy-visible values
- **THEN** no generic state lookup, parser value, or caller-visible state identifier enters that input

### Requirement: Adapter definition owns registry-facing adapter facts

Adapter definition, manifest, probe, and descriptor metadata MUST describe adapter identity, supported format facts, capability declarations, and the linked strategy implementation. The adapter definition MUST remain the registry-facing aggregation point for those adapter behavior facts. Any approved adapter-private lifecycle mechanics MUST stay behind the registry-selected adapter boundary and MUST NOT become public manifest or probe metadata. A separately approved core-owned immutable document source primitive, if selected, MUST remain acquisition mechanics rather than adapter behavior metadata and MUST NOT transfer format semantics away from the adapter. This requirement does not prescribe whether the total mechanism uses a returned value, handle, receiver, core source primitive, local representation, or a bounded combination. The fixed adapter strategy interface MUST provide outline, read, find, and info functions. Adapter-private helpers MAY construct manifest or capability values and private document state, but shared layers MUST consume adapter behavior facts through the exported definition/factory. Adapter implementation source MUST remain a core static-registry fact. Caller-configurable document-operation parameter facts MUST come from the separate core catalog.

#### Scenario: Core lists built-in adapters

- **WHEN** `docnav adapter list` inspects adapters
- **THEN** implementation source comes from the core static registry
- **THEN** manifest/probe metadata describes adapter capability and format support
- **THEN** document-operation parameter facts remain in the separate core catalog
- **THEN** invocation-private lifecycle and parser facts are absent from listed metadata

#### Scenario: Registry consumes one adapter definition

- **WHEN** a built-in adapter is registered with core
- **THEN** the registry receives one adapter definition containing identity, format descriptors, a linked strategy implementation, and optional capabilities
- **THEN** the fixed strategy interface provides the required operations
- **THEN** caller-configurable parameter facts come from the core catalog
- **THEN** any approved private lifecycle representation remains internal to the registry-selected adapter boundary

#### Scenario: Adapter implementation uses private helpers

- **WHEN** an adapter implementation splits definition construction across private helper functions or modules
- **THEN** it exports one registry-facing definition or definition factory
- **THEN** registry, navigation, and dispatch consume adapter-owned behavior facts through that definition
- **THEN** core catalog remains the only parameter-definition input

#### Scenario: Unsupported candidate is released

- **WHEN** an adapter probe is unsupported or invalid during automatic discovery
- **THEN** any candidate-private decoded, parser, index, or source-region state becomes unreachable under the approved bounded cleanup policy
- **THEN** a separately approved shared source view follows its own invocation bound
- **THEN** that state is not transferred to a different adapter
- **THEN** public probe evidence and discovery continuation retain their existing meaning

### Requirement: Adapter operation support is explicit

Adapter definitions MUST declare supported document operations and capability groups, including unstructured full-read support, content, cost measurement, and result facts used by navigation pre-dispatch policy. Required operation handler handles and capability group handles MUST be reachable from the same adapter definition. Navigation uses declared support facts when selecting adapter-level capabilities. Capability groups MUST aggregate related hooks under one declared owner boundary. When navigation calls multiple declared hooks or falls back to a normal operation over the same approved document view in one invocation, the adapter contract MUST allow the selected adapter to reuse compatible private preparation without changing the declared public capability facts.

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

#### Scenario: Full-read stages share private preparation

- **WHEN** navigation measures full-read cost and then invokes content/facts hooks or structured outline over the same approved document view
- **THEN** the selected adapter can reuse its compatible private preparation across those calls
- **THEN** result facts and diagnostics remain those of the existing declared hooks and operation
- **THEN** lifecycle reuse does not create a new public capability group
