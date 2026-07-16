## MODIFIED Requirements

### Requirement: Native options are adapter-owned declarations

Format-native options MUST be declared by the owning adapter in the adapter definition and consumed by navigation input resolution as owner-scoped input sources. Shared layers MUST accept native option input only through adapter declarations and MUST resolve request effects only through the selected adapter/current-operation declarations.

The same declaration MUST provide canonical field identity、value kind、constraints、default、processing locators、operation applicability and adapter-specific internal typed handoff or accessor binding. When an option is exposed as a public document CLI flag, that declaration MUST also provide owner-authored CLI help、value name and Boolean input encoding. Accepted values、defaults and validation semantics MUST be projected from canonical field facts. External protocol JSON shape remains owned by the protocol contract.

#### Scenario: Adapter declares a native option

- **WHEN** a Markdown adapter option is registered in the static registry through its adapter definition
- **THEN** navigation can extract and validate that option for its declared Markdown operations
- **THEN** the option applies only to the declaring adapter and declared operations
- **THEN** the internal dispatch boundary can provide the Markdown handler with a typed option value

#### Scenario: Public CLI option derives from the adapter declaration

- **WHEN** an adapter declaration exposes a native option on a document CLI operation
- **THEN** operation help、flag registration、candidate identity and selected validation derive from that declaration
- **THEN** core consumes the operation-scoped projection supplied from registered declarations

#### Scenario: Caller supplies an option not declared for the selected adapter

- **WHEN** explicit caller input contains a native option not declared for the selected adapter/current operation
- **THEN** input resolution reports a strict caller-input diagnostic
- **THEN** dispatch stops before forwarding the option

#### Scenario: Native option declaration stays the semantic owner

- **WHEN** project config、user config or explicit CLI input provides an adapter native option
- **THEN** navigation uses the selected adapter declaration to resolve and validate the value
- **THEN** the adapter declaration remains the source of value kind、default、range and operation applicability semantics
- **THEN** core and navigation consume those facts through canonical projections

#### Scenario: Config-only native option omits public CLI metadata

- **WHEN** an adapter native option is accepted only through config or another declared non-CLI source
- **THEN** its declaration can omit CLI metadata
- **THEN** it is absent from the public CLI projection

#### Scenario: Native option declaration drives every declared consumer

- **WHEN** a native option declaration exposes CLI、config、default、validation and handler binding metadata
- **THEN** CLI option discovery、config validation、navigation extraction and dispatch handoff use that declaration
- **THEN** that declaration is the complete shared authoring source for adapter-owned option facts
