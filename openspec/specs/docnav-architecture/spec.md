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
Architecture MUST assign each durable rule to one owner. Core CLI owns command entry and process behavior; navigation owns request preparation and dispatch; adapters own format semantics; protocol owns machine envelopes; output owns readable projections; diagnostics own stable error identity; refs own cross-layer ref opacity.

#### Scenario: Cross-layer behavior changes
- **WHEN** a change affects multiple layers
- **THEN** each changed rule is recorded in its owning capability
- **THEN** architecture records only the boundary or dependency between those owners

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
