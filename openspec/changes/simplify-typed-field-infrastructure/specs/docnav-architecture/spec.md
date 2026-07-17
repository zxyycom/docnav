## MODIFIED Requirements

### Requirement: Component ownership is single-sourced

Architecture MUST assign each durable rule to one owner. Core owns command/process behavior, the closed catalog of caller-configurable document-operation parameters including adapter-scoped parameters, and the closed standard operation-input contract. The standard input Rust types MAY live in the existing shared operation-contract layer required by navigation and adapters, but that dependency placement MUST NOT transfer accepted-input or binding ownership away from core. Navigation owns source loading, full-catalog config validation, adapter selection, selected-operation filtering, typed-field resolution orchestration, standard-input construction, and dispatch. Adapters own the fixed strategy interface, format detection, parsing, navigation algorithms, algorithmic semantic validation, refs, and result facts. Caller-configurable parameter facts MUST remain in the core catalog even when an adapter validates or revalidates a standard value. Protocol owns machine envelopes; contract-validation owns schema and runtime validation gates while preserving field-owner semantics; output owns readable projections; diagnostics own stable error identity; refs own cross-layer ref opacity.

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
