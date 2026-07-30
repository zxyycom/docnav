**Interpretation:** This mechanism-neutral Target delta fixes the
navigation/adapter/public responsibility boundaries but does not select storage
ownership, a source snapshot, a handle, a session, or a Rust type.
`proposal.md` owns the change status; `design.md` leaves “approved invocation
lifecycle” and “approved document view” open; tasks 1.7–1.8 must approve and
define them before applying this delta.

## MODIFIED Requirements

### Requirement: Component ownership is single-sourced

Architecture MUST assign each durable rule to one owner. Core owns command/process behavior, the closed catalog of caller-configurable document-operation parameters including adapter-scoped parameters, and the closed standard operation-input contract. The standard input Rust types MAY live in the existing shared operation-contract layer required by navigation and adapters, but that dependency placement MUST NOT transfer accepted-input or binding ownership away from core. Navigation owns raw navigation configuration-source loading, full-catalog config validation, adapter selection, selected-operation filtering, typed-field resolution orchestration, standard-input construction, dispatch, operation composition, and the bounded lifetime of invocation-private reusable document state. Configuration-source loading MUST NOT be interpreted as ownership of document-file acquisition or immutable document byte storage; that storage owner and source-view lifetime MUST follow the explicitly approved mechanism. Adapters own the fixed strategy interface, format detection, decode/parse semantics, adapter-private parser/index/source-region state, navigation algorithms, algorithmic semantic validation, refs, and result facts. Navigation MUST NOT inspect adapter-private state, and adapter-private state MUST NOT enter public protocol, output, ref, continuation, logging, or caller-visible identifiers. Caller-configurable parameter facts MUST remain in the core catalog even when an adapter validates or revalidates a standard value. Protocol owns machine envelopes; contract-validation owns schema and runtime validation gates while preserving field-owner semantics; output owns readable projections; diagnostics own stable error identity; refs own cross-layer ref opacity.

#### Scenario: Cross-layer behavior changes

- **WHEN** a change affects multiple layers
- **THEN** each changed rule is recorded in its owning capability
- **THEN** architecture records only the boundary or dependency between those owners

#### Scenario: Format-specific parameter exists

- **WHEN** a parameter applies only to one adapter or operation
- **THEN** core declares its accepted input facts, standard value kind, optional exact adapter-id marker, operation/standard-input bindings, and pre-dispatch validation policy in the product catalog
- **THEN** navigation resolves it and passes a closed standard operation input
- **THEN** the adapter owns how that value affects format behavior and any semantic check required by the strategy
- **THEN** adapter-side validation does not create a parameter declaration surface

#### Scenario: One invocation composes adapter work

- **WHEN** navigation combines selection, pre-dispatch policy, a base operation, or a nested operation over one approved document view
- **THEN** navigation controls which stages run and when the reusable state lifetime ends
- **THEN** the selected adapter controls the private decode, parse, index, source-region, and ref facts reused by those stages
- **THEN** document byte acquisition and storage follow the separately approved ownership rule
- **THEN** neither owner assumes the other's semantic responsibilities

#### Scenario: Private state is not observable contract

- **WHEN** an adapter reuses invocation-private document state
- **THEN** protocol, output, ref, continuation, logging, schema, and caller inputs remain free of state handles and parser facts
- **THEN** public behavior is derived only from the existing operation result or diagnostic contracts

### Requirement: Default document operations use linked adapter libraries

The default document operation implementation source MUST be the current core release's static linked adapter set. Invocation-private reusable adapter state MUST remain inside the linked execution process and MUST end with the bounded navigation invocation. Future runtime adapter models require their own capability and MUST leave this default path explicit while they are not the selected architecture.

#### Scenario: Core dispatches a document operation

- **WHEN** `docnav` dispatches outline, read, find, or info
- **THEN** implementation candidates come from the core release static registry
- **THEN** the selected linked adapter library receives prepared operation input

#### Scenario: Linked invocation reuses private state

- **WHEN** the approved same-invocation lifecycle reuses selected-adapter preparation
- **THEN** the reusable state remains private to the linked adapter execution boundary
- **THEN** it is no longer retained after the invocation completes or fails
- **THEN** no cross-invocation cache is created by this capability

### Requirement: Integration entry points share Docnav contracts

Integration surfaces such as MCP bridges or local service modes MUST delegate document semantics to the Docnav document operation contracts instead of re-parsing documents, reinterpreting refs, or inventing incompatible output semantics. They MUST NOT serialize invocation-private adapter state or extend it into a cross-request cache; a future external adapter host requires its own process-local lifecycle design without adding a public session identifier through this capability.

#### Scenario: Bridge invokes Docnav

- **WHEN** an integration surface exposes a document tool
- **THEN** it maps caller input to Docnav document operations
- **THEN** it preserves Docnav success and failure semantics at its own transport boundary

#### Scenario: Local service executes a document operation

- **WHEN** a local service delegates a document operation to the linked adapter path
- **THEN** it uses the same approved navigation invocation lifecycle
- **THEN** any adapter-private reusable state ends with that invocation
- **THEN** service caching remains limited to separately owned core facts

#### Scenario: Future execution crosses a process boundary

- **WHEN** a future capability executes an adapter in another process
- **THEN** this capability does not authorize serializing parser state or exposing a public state identifier
- **THEN** that capability must define how the adapter host keeps any reusable state host-local and bounded
