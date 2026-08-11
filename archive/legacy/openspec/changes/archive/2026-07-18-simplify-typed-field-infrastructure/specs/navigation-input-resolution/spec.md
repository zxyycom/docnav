## ADDED Requirements

### Requirement: Core catalog owns adapter-scoped parameter facts

Navigation MUST obtain common and adapter-scoped parameter identity, extraction metadata, standard value kind, defaults, merge strategy, core validation facts when present, operation binding, an optional exact static adapter-id marker, and closed compile-time consumer binding from the core-owned closed catalog. Every entry MUST have one compatible closed consumer target. Only strategy-visible values MUST target the shared `StandardInputBinding`; pagination/output controls MUST target navigation/core-owned closed variants and MUST NOT appear in adapter input. An entry without an adapter-id marker is common. An entry with a marker MUST participate in selected-operation resolution only when that marker equals the selected adapter id. Full config validation MUST use the complete catalog projection without making every catalog entry part of the selected operation; navigation MUST apply the exact-id filter and then the operation binding before candidate extraction, resolution, and request construction for that operation. Adapter definitions provide strategy and behavior facts rather than catalog entries. Adapter-side semantic checks MAY consume standard values but MUST NOT contribute parameter facts.

#### Scenario: Selected Markdown adapter

- **WHEN** Markdown is selected for outline
- **THEN** navigation includes untagged outline parameters and core entries tagged exactly `docnav-markdown`
- **THEN** core-defined parameters for other adapters or operations remain outside that operation field set
- **THEN** resolved values are bound to standard Markdown operation input

#### Scenario: Adapter tag does not match

- **WHEN** a catalog entry is tagged for an adapter id different from the selected adapter
- **THEN** navigation may recognize and validate its config path through the full catalog projection
- **THEN** navigation excludes that entry from selected candidate extraction and resolution
- **THEN** the selected strategy cannot observe the field or its source values

#### Scenario: Unknown adapter-scoped parameter

- **WHEN** caller input contains an adapter-scoped path absent from the full catalog, or selected direct input is not bound to the selected adapter and operation
- **THEN** navigation reports a strict caller-input diagnostic
- **THEN** dispatch stops before the value reaches the adapter

#### Scenario: Adapter definition contributes behavior facts

- **WHEN** a selected adapter definition is loaded
- **THEN** navigation reads strategy and capability facts from that definition
- **THEN** navigation reads all caller-configurable parameter facts from the core catalog

#### Scenario: Adapter validates a standard value

- **WHEN** navigation dispatches a materialized adapter-scoped value whose remaining semantics were not guaranteed by core validation
- **THEN** the selected strategy may validate the standard value
- **THEN** the strategy returns a diagnostic or result without changing catalog membership or source behavior

## MODIFIED Requirements

### Requirement: Core hands raw navigation inputs to navigation

Core CLI MUST hand document operation command facts, normalized path facts, config source descriptors/paths, the static adapter registry, and the core-owned closed parameter catalog to navigation without resolving operation arguments. The selected adapter definition contributes capability and linked strategy facts to this handoff; parameter facts come from the catalog.

#### Scenario: Outline handoff

- **WHEN** core parses `docnav outline <path>`
- **THEN** it identifies the operation and path facts
- **THEN** navigation receives the raw navigation input package and uses the core catalog for resolution
- **THEN** the selected adapter definition supplies behavior facts only

### Requirement: Navigation selects adapter before adapter parameter extraction

Navigation MUST select the adapter using routing inputs and registry facts before filtering adapter-scoped entries for selected-operation candidate extraction and resolution. Full catalog config validation is a separate projection and MUST NOT be treated as adapter parameter extraction. The selected registry entry MUST expose an adapter definition for capability and linked strategy facts. Document-operation parameter declarations MUST come from the core catalog rather than that definition.

#### Scenario: Multiple adapters exist

- **WHEN** registry contains multiple candidate adapters
- **THEN** navigation selects the adapter according to selection rules
- **THEN** only core catalog entries applicable to the selected adapter and operation participate in resolution
- **THEN** entries scoped to unselected adapters remain outside the operation field set

#### Scenario: Selected definition provides capability facts

- **WHEN** navigation has selected an adapter
- **THEN** it reads optional capability declarations and the linked strategy from the selected adapter definition
- **THEN** it reads parameter facts from the core catalog

### Requirement: Config sources are inputs, not semantic owners

Project and user config files MUST be treated as input sources. Core catalog MUST own caller-configurable document-operation parameter identity, paths, standard types, defaults, merge strategy, optional exact adapter-id marker, operation/closed-consumer bindings, and the static validation selected for core execution. Navigation MUST own source resolution and consumer-specific projection construction. An adapter MAY own how a standard adapter-scoped value affects format behavior and MAY validate algorithmic semantics before use.

#### Scenario: Markdown-scoped parameter in config

- **WHEN** a config file provides `options.docnav-markdown.max_heading_level`
- **THEN** navigation attributes the source to that config
- **THEN** core catalog provides the document-operation parameter facts
- **THEN** Markdown owns the value's effect on its outline/find algorithm and may defensively validate it
- **THEN** Markdown does not become the parameter declaration owner

### Requirement: Request construction consumes typed resolution results

Navigation MUST construct protocol operation arguments/request envelopes, strategy-facing standard operation input, and `PreparedNavigationRequest` / core output projection as consumer-specific projections of the same typed resolution result. Standard input MUST be the closed operation-specific Rust contract shared by navigation and adapter strategies. Core-defined bindings MUST populate only strategy-visible values through compile-time fields, typed accessors, or closed enum variants rather than a generic parameter lookup surface. `pagination.enabled` MUST combine with `limit` to normalize the effective limit before dispatch; `output` MUST populate only `PreparedNavigationRequest` / core output projection and MUST NOT enter adapter input. Standard input MUST represent completed source resolution and type materialization; it MUST NOT claim that all adapter semantic validation has completed. Protocol `Options` MUST retain its stable serialized values shape. Raw argv strings, raw config JSON, declaration metadata, display output, protocol envelopes, and serialized protocol representation MUST remain outside the strategy-input projection.

#### Scenario: Read request

- **WHEN** core/navigation has normalized document path and ref, and catalog resolution produces page and limit
- **THEN** navigation constructs read operation arguments
- **THEN** adapter dispatch receives those normalized facts through the closed typed read input

#### Scenario: Operation includes an adapter-scoped value

- **WHEN** typed resolution produces core-defined adapter-scoped values
- **THEN** request construction binds them according to core catalog operation bindings
- **THEN** the selected strategy receives those values through compile-time standard-input fields or typed accessors

#### Scenario: Protocol-stable options remain compatible

- **WHEN** protocol request construction includes `OperationArguments.options`
- **THEN** navigation constructs the existing protocol object from typed resolution results
- **THEN** it constructs standard strategy input directly from the same resolution result
- **THEN** the external protocol shape remains separate from internal parameter authoring ownership

### Requirement: Navigation dispatches linked adapter handlers

After successful input resolution, standard type materialization, and configured core pre-dispatch checks, navigation MUST dispatch the closed standard operation input to the selected linked adapter strategy and return structured result or diagnostic facts to the owning output/protocol layer. The strategy reference and capability context MUST come from the selected adapter definition; applicable operation-specific typed fields or accessors MUST be built from core-catalog resolution. The selected strategy MUST NOT require a second caller-data argument or generic parameter handoff. It MAY return semantic validation diagnostics for conditions not guaranteed by core or MAY repeat a core check defensively.

#### Scenario: Dispatch succeeds

- **WHEN** navigation has constructed standard typed operation input
- **THEN** it calls the selected adapter strategy
- **THEN** it preserves the returned structured result facts for projection

#### Scenario: Dispatch returns adapter semantic diagnostic

- **WHEN** standard input is well-typed but violates a selected strategy precondition
- **THEN** the strategy returns a diagnostic before running the unsafe or invalid algorithm path
- **THEN** navigation preserves that diagnostic for normal protocol/readable projection

#### Scenario: Dispatch combines separate core facts

- **WHEN** navigation dispatches a selected operation
- **THEN** the strategy implementation comes from the selected adapter definition
- **THEN** adapter-scoped typed values come from entries applicable to that adapter and operation in core catalog
- **THEN** routing/strategy facts and parameter facts remain owned by their separate sources

### Requirement: Navigation exposes parameter aggregation projections

Navigation MUST expose scalar parameter projections derived from the core-owned catalog for `page`, `limit`, `pagination.enabled`, `output`, and adapter-namespaced Markdown `max_heading_level`. These projections MUST preserve processing paths, field identity, the optional exact adapter-id marker, operation binding, standard value kind, core validation facts when present, defaults, merge strategy, and closed consumer binding facts. The full config-validation projection MUST combine those scalar catalog facts with the current owner-specific compound validator for `outline.mode_rules` / `outline.thresholds`; those compound algorithms MUST NOT be required to become ordinary scalar catalog fields. The selected-operation field set MUST apply exact adapter-id and operation filters before candidate extraction and resolution. Both catalog views MUST derive from the same scalar facts; strategy validation remains outside those projections.

#### Scenario: Config-source projection includes common fields

- **WHEN** navigation builds the config-source projection for document operation inputs
- **THEN** metadata for `page`, `defaults.pagination.enabled`, `defaults.pagination.limit`, and `defaults.output` comes from core catalog
- **THEN** consumers validate config source values without redefining those field facts

#### Scenario: Config-source projection includes adapter-scoped fields

- **WHEN** navigation builds config-source metadata
- **THEN** core catalog entries are projected under `options.<adapter-id>.<parameter-key>`
- **THEN** equal keys for different adapter ids remain distinct config paths

### Requirement: Config source validation uses the config-source projection

Navigation MUST validate config source keys and declared static values through the full core-catalog config projection before constructing operation arguments. Unknown fields, unknown adapter ids, values that cannot be materialized as the catalog standard type, owner-specific object/array shape failures in the supported subset, and configured core validation failures MUST produce blocking diagnostics with config source attribution. A known valid field for another adapter MAY remain a valid source fact, but only the selected-operation field set may contribute candidates to resolution or standard input. Within the selected adapter namespace, a known field not bound to the selected operation MUST continue to produce the existing unsupported-parameter diagnostic. A well-typed selected value whose remaining semantics are deliberately deferred MAY reach the selected strategy.

#### Scenario: Config adapter-scoped value fails validation

- **WHEN** a project config file contains `options.docnav-markdown.max_heading_level` outside `1..=6`
- **THEN** navigation reports a blocking typed validation diagnostic using core catalog facts
- **THEN** its public code, owner labels, field/source details, expected/received facts, and guidance remain compatible with the existing adapter-scoped diagnostic contract
- **THEN** adapter dispatch does not occur

#### Scenario: Config value requires adapter semantics

- **WHEN** a known adapter-scoped config value can be materialized and passes configured core checks but requires document or algorithm context
- **THEN** navigation constructs standard input with source attribution
- **THEN** the selected strategy performs the remaining semantic validation
- **THEN** a semantic failure is returned through the normal diagnostic contract

#### Scenario: Config parameter is not applicable to selected operation

- **WHEN** a user config file contains a known path in the selected adapter namespace that is not bound to the selected operation
- **THEN** navigation reports an unsupported parameter diagnostic for that config source
- **THEN** adapter dispatch does not occur for that invalid operation input

#### Scenario: Config contains a known field for another adapter

- **WHEN** a config source contains a well-typed catalog path for an adapter other than the selected adapter
- **THEN** full config validation accepts the known source fact
- **THEN** selected-operation resolution does not extract, merge, validate as selected input, or forward that value

#### Scenario: Unknown adapter namespace is blocking

- **WHEN** a config source contains `options.unknown_adapter.max_heading_level`
- **THEN** navigation reports an unknown adapter id or unknown catalog path diagnostic with config source attribution
- **THEN** adapter dispatch does not occur

### Requirement: Navigation consumes selected adapter namespace

When navigation constructs operation arguments for a selected adapter and operation, it MUST consume adapter-scoped values only from untagged core catalog entries or entries whose exact adapter-id marker matches the selected adapter, and only when their operation binding matches. Values stored under other known adapter ids MUST remain separate source facts and MUST NOT be forwarded to the selected adapter strategy.

#### Scenario: Selected adapter reads its own namespace

- **WHEN** config contains `options.docnav-markdown.max_heading_level` and `options.docnav-other.max_heading_level`
- **AND** navigation selects `docnav-markdown` for outline
- **THEN** full config validation recognizes both catalog paths
- **THEN** selected-operation resolution applies the core-defined validation policy and consumes only the Markdown path
- **THEN** the other adapter path is not forwarded to Markdown or used as Markdown operation input

## REMOVED Requirements

### Requirement: Selected adapter declarations own parameter facts

**Reason**: Product parameters are now authored once in core catalog. Reading declarations from selected adapter definitions recreates a dynamic injection boundary that the static release model does not use.

**Migration**: Filter core catalog by selected adapter/operation and bind resolved typed values to standard operation input. Keep selected adapter definition as the owner of capability and strategy facts only. Strategy-side semantic validation remains allowed.

### Requirement: Adapter native options are owner-scoped

**Reason**: Applicability remains adapter-scoped, but caller-configurable document-operation parameter ownership moves to core. Validation no longer requires adapter-provided declarations; runtime strategy validation may still consume standard values.

**Migration**: Validate unknown, unsupported, standard-type, and configured core rules against core catalog. Preserve source attribution and standard adapter consumption, and map any deferred adapter semantic failure through the normal diagnostic contract.
