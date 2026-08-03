# docnav-architecture Specification

## Purpose
Define Docnav's durable component boundaries and cross-layer invariants. This capability owns the component map, default operation flow, shared-helper boundary, and integration entry-point rules; detailed CLI flags, protocol fields, adapter behavior, output rendering, diagnostics, refs, and validation rules stay in their dedicated owner capabilities.
## Requirements
### Requirement: Document navigation follows bounded operation flow
Docnav document navigation MUST keep the primary flow as `outline -> ref -> read`. Any exception to that flow MUST name its owning capability and MUST keep the observable result bounded, typed, and auditable.

#### Scenario: Standard structured navigation
- **WHEN** a caller needs to inspect a structured document
- **THEN** the caller can obtain entries through outline
- **THEN** the caller can pass a returned ref unchanged to read
- **THEN** read returns the bounded region identified by that ref

#### Scenario: Declared exception
- **WHEN** an operation bypasses the normal outline/ref/read chain
- **THEN** the owning capability names the exception
- **THEN** the protocol and output capabilities still define its result shape

### Requirement: Component ownership is single-sourced

Architecture MUST assign each durable rule to one owner. Core owns command/process behavior, static registry membership, manifest-derived unique format/pathname-hint validation, lexical derivation of the invocation-private routing pathname, post-selection filesystem-backed document path/access normalization, the closed catalog of caller-configurable document-operation parameters including adapter-scoped parameters, and the closed standard operation-input contract. The standard input Rust types MAY live in the existing shared operation-contract layer required by navigation and adapters, but that dependency placement MUST NOT transfer accepted-input or binding ownership away from core. Adapters own declarative manifest format identities, `extensions[]` basename suffixes, `filenames[]`, capability facts, and the fixed selected strategy interface. Navigation owns source loading, full-catalog config validation, one invocation-private complete-basename lookup over registry-derived exact-filename and ASCII-normalized suffix views, adapter selection, the sequencing boundary that permits filesystem-backed document processing only after selection, selected-operation filtering, typed-field resolution orchestration, standard-input construction, and dispatch. The derived indexes, matched hint, and matched format identity MUST remain private implementation facts rather than a public or adapter extension surface, and routing MUST add no external format-inference or regex dependency. After selection, adapters own document acquisition required by the operation, decoding/parsing, format semantics, navigation algorithms, algorithmic semantic validation, refs, and result facts. Adapters MUST NOT execute automatic-selection detection. Caller-configurable parameter facts MUST remain in the core catalog even when an adapter validates or revalidates a standard value. Protocol owns machine envelopes and routing diagnostic projection; contract-validation owns schema and runtime validation gates while preserving field-owner semantics; output owns readable projections; diagnostics own stable error identity and canonical routing details; refs own cross-layer ref opacity.

#### Scenario: Cross-layer behavior changes

- **WHEN** a change affects format routing, registry validation, selected parsing, and protocol diagnostics
- **THEN** core owns registry format/pathname-hint integrity
- **THEN** adapters own the manifest pathname facts
- **THEN** core/navigation own private route-before-I/O sequencing, complete-basename matching, selection, and no-fallback dispatch
- **THEN** the selected adapter owns actual document parse and format semantics
- **THEN** diagnostics/protocol owners define the public failure identity and exact details

#### Scenario: Automatic routing selects one adapter

- **WHEN** navigation privately maps one routing basename to a manifest-owned format identity
- **AND** core's validated registry maps that identity to one definition
- **THEN** navigation selects that definition without executing adapter detection code
- **THEN** core/navigation do not inspect target metadata, open, canonicalize, read, or parse the document during selection
- **THEN** filesystem-backed document path/access normalization starts only after selection
- **THEN** the selected strategy still performs the actual document processing required by the operation

#### Scenario: Format-specific parameter exists

- **WHEN** a parameter applies only to one adapter or operation
- **THEN** core declares its accepted input facts, standard value kind, optional exact adapter-id marker, operation/standard-input bindings, and pre-dispatch validation policy in the product catalog
- **THEN** navigation resolves it and passes a closed standard operation input
- **THEN** the adapter owns how that value affects format behavior and any semantic check required by the strategy
- **THEN** adapter-side validation does not create a parameter declaration surface

### Requirement: Default document operations use linked adapter libraries
The default document operation implementation source MUST be the current core release's static linked adapter set. Future runtime adapter models require their own capability and MUST leave this default path explicit while they are not the selected architecture.

#### Scenario: Core dispatches a document operation
- **WHEN** `docnav` dispatches outline, read, find, or info
- **THEN** implementation candidates come from the core release static registry
- **THEN** the selected linked adapter library receives prepared operation input

### Requirement: Shared helpers preserve policy ownership
Shared crates and helpers MUST centralize only reusable mechanics. CLI behavior, adapter semantics, protocol envelopes, output projections, diagnostics, refs, and validation material keep their owning capabilities even when they share helper code.

#### Scenario: Helper is reused across layers
- **WHEN** multiple components consume a shared helper
- **THEN** each component keeps its observable policy in its own capability
- **THEN** the helper exposes mechanics without redefining that component's public contract

### Requirement: Integration entry points share Docnav contracts
Integration surfaces such as MCP bridges or local service modes MUST delegate document semantics to the Docnav document operation contracts instead of re-parsing documents, reinterpreting refs, or inventing incompatible output semantics.

#### Scenario: Bridge invokes Docnav
- **WHEN** an integration surface exposes a document tool
- **THEN** it maps caller input to Docnav document operations
- **THEN** it preserves Docnav success and failure semantics at its own transport boundary
