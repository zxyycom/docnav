本 delta spec 是 `replace-probe-traversal-with-inferred-routing` 的临时契约工件：它把 registry-facing format descriptors 与 selected adapter operations 保留为固定边界，同时从 routing contract 移除 adapter probe。

## MODIFIED Requirements

### Requirement: Adapter definition owns registry-facing adapter facts

Adapter definition, manifest, and descriptor metadata MUST describe adapter identity, supported format facts, capability declarations, and the linked strategy implementation. The adapter definition MUST be the registry-facing aggregation point for those adapter behavior facts. Format descriptors MUST expose project-owned normalized format identities that navigation can match by exact equality without executing adapter code. Across the core static registry, one normalized format identity MUST resolve to at most one adapter definition; registry construction, doctor, and release validation MUST reject duplicates before document routing. The fixed adapter strategy interface MUST provide outline, read, find, and info functions and MUST NOT define a probe function, probe result, probe reason, probe version, or selection-detection hook. Adapter-private helpers MAY construct manifest or capability values, but shared layers MUST consume adapter behavior facts through the exported definition/factory. Adapter implementation source MUST remain a core static-registry fact. Caller-configurable document-operation parameter facts MUST come from the separate core catalog. Format inference MUST remain navigation-private routing mechanics and MUST NOT transfer document decode, parse, ref, or operation semantics away from the selected adapter.

#### Scenario: Core lists built-in adapters

- **WHEN** `docnav adapter list` inspects adapters
- **THEN** implementation source comes from the core static registry
- **THEN** manifest metadata describes adapter capability and normalized format support
- **THEN** listing does not execute a selection probe
- **THEN** document-operation parameter facts remain in the separate core catalog

#### Scenario: Registry consumes one adapter definition

- **WHEN** a built-in adapter is registered with core
- **THEN** the registry receives one adapter definition containing identity, format descriptors, a linked strategy implementation, and optional capabilities
- **THEN** the fixed strategy interface provides the required operations and exposes no probe surface
- **THEN** caller-configurable parameter facts come from the core catalog

#### Scenario: Automatic routing uses format descriptors

- **WHEN** navigation has normalized one inferred format identity
- **THEN** registry matching compares that identity exactly with definition format descriptors
- **THEN** an adapter definition is not executed merely to decide whether it is a candidate
- **THEN** registry order does not choose between equal format identities

#### Scenario: Registry declares duplicate format identity

- **WHEN** two built-in adapter definitions declare the same normalized format identity
- **THEN** core registry construction, doctor, and release validation reject the registry
- **THEN** no document invocation uses registry order to choose one definition

#### Scenario: Adapter implementation uses private helpers

- **WHEN** an adapter implementation splits definition construction across private helper functions or modules
- **THEN** it exports one registry-facing definition or definition factory
- **THEN** registry, navigation, and dispatch consume adapter-owned behavior facts through that definition
- **THEN** core catalog remains the only parameter-definition input
