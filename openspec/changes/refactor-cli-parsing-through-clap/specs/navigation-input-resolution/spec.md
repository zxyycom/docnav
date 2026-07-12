本 delta 的核心目标是让 navigation 消费 typed CLI source，并把 pre-selection projection 与 selected-adapter resolution 明确分层；本文是仅位于 `openspec/changes/refactor-cli-parsing-through-clap/` 的未审核临时 spec，不影响现有主规范或其它文档。

## RENAMED Requirements

- FROM: `Core hands raw navigation inputs to navigation`
- TO: `Core hands normalized navigation inputs to navigation`
- FROM: `Navigation selects adapter before adapter parameter extraction`
- TO: `Navigation separates registry-wide CLI projection from selected adapter resolution`

## MODIFIED Requirements

### Requirement: Core hands normalized navigation inputs to navigation
Core CLI MUST hand navigation document operation facts, normalized path facts, config source descriptors/paths, the static adapter registry, and the canonical typed native CLI `Source` produced by `cli-config-resolution-clap`. Core MUST NOT pre-own adapter option semantics or operation arguments. Raw native option strings and Clap `ArgMatches` MUST NOT cross into navigation.

#### Scenario: Outline handoff
- **WHEN** core parses `docnav outline <path>` and any declared native CLI arguments
- **THEN** core produces operation/path facts and a typed native CLI source
- **THEN** navigation receives those facts with config descriptors and the registry for resolution

### Requirement: Navigation separates registry-wide CLI projection from selected adapter resolution
Navigation MUST select the adapter from routing inputs and registry facts before resolving or binding selected adapter native options. Before selection, parameter aggregation MAY expose one registry-wide, operation-scoped canonical CLI projection solely for core and the Clap companion to register and lexically decode every statically linked flag that can occur on that command. Only declarations from the selected adapter definition MAY participate in selected operation validation, default resolution, typed handoff, capability policy, or dispatch.

#### Scenario: Multiple adapters exist
- **WHEN** the registry contains multiple adapter candidates
- **THEN** the operation command registers the registry-wide set of globally unique native CLI flags
- **THEN** navigation selects the adapter according to adapter selection rules
- **THEN** only the selected adapter declarations enter semantic resolution and handler binding

#### Scenario: Selected definition provides capability facts
- **WHEN** navigation selects an adapter
- **THEN** navigation reads optional capability declarations from that selected definition
- **THEN** pre-dispatch policy uses only those declared support facts

#### Scenario: Selected definition remains the fact source
- **WHEN** navigation prepares selected operation resolution and dispatch
- **THEN** declaration registration, full-read pre-dispatch, handler selection, and native option binding use one coherent selected adapter definition

### Requirement: Adapter native options are owner-scoped
Navigation MUST validate and resolve native options only when the selected adapter definition declares them for the current operation. Explicit unknown, unselected-adapter, or operation-inapplicable options MUST fail strictly. Declared options MUST become typed values before dispatch, and handlers MUST receive those values or typed accessors.

#### Scenario: Unknown native option
- **WHEN** a caller provides an option not declared by the selected adapter
- **THEN** navigation reports a strict input diagnostic
- **THEN** dispatch stops before the option reaches an adapter handler

#### Scenario: Explicit option belongs to another adapter
- **WHEN** the registry-wide projection accepts a flag owned by one adapter but navigation selects another adapter
- **THEN** navigation reports a strict unsupported or unused native option diagnostic with CLI source attribution
- **THEN** navigation neither discards nor forwards that candidate silently

#### Scenario: Lexically invalid option fails before owner filtering
- **WHEN** a registry-declared native flag has a value that its Clap parser cannot decode
- **THEN** core returns the Clap lexical input diagnostic before navigation selection or owner filtering
- **THEN** navigation does not re-read raw input to infer a different adapter-owner failure

#### Scenario: Declared native option becomes typed handoff
- **WHEN** a caller provides a valid declared native option for the selected adapter and operation
- **THEN** navigation validates the typed candidate through the selected declaration
- **THEN** navigation records source attribution and provides the handler with the typed value or accessor

### Requirement: Navigation exposes parameter aggregation projections
Navigation MUST derive parameter aggregation from common navigation fields, outline mode config fields, and adapter-id namespaced declarations. Aggregation MUST preserve processing paths, field identity, owner, adapter id when applicable, value kind, constraints, defaults, supported owner-specific shape validation, and source binding facts. It MUST produce CLI/input and config-source projections without taking ownership of adapter-native semantics or source priority. For each operation, the registry-wide native CLI projection MUST include only applicable declarations and MUST reject duplicate CLI locators across adapter owners.

#### Scenario: Config-source projection includes common fields
- **WHEN** navigation builds the document-operation config-source projection
- **THEN** `defaults.pagination.enabled`, `defaults.pagination.limit`, `defaults.output`, and declared outline mode fields come from the same canonical facts used by resolution
- **THEN** consumers validate those values without redefining field metadata

#### Scenario: Config-source projection includes adapter-id options
- **WHEN** navigation builds config-source metadata from the adapter registry
- **THEN** native options are projected under `options.<adapter-id>.<option-key>`
- **THEN** equal option keys from different adapter ids remain distinct config paths

#### Scenario: CLI projection includes operation-applicable adapter flags
- **WHEN** navigation builds an operation's native CLI projection from the static registry
- **THEN** it includes every applicable adapter declaration's canonical CLI metadata
- **THEN** core can pass that projection to `cli-config-resolution-clap` without reconstructing ids, value kinds, or actions

#### Scenario: Duplicate adapter CLI flag is a declaration conflict
- **WHEN** two declarations applicable to the same operation use the same CLI locator
- **THEN** release-local command-model validation reports a deterministic internal declaration conflict before authoritative parsing or dispatch
- **THEN** it neither infers compatibility nor chooses an owner implicitly
