## ADDED Requirements

### Requirement: Core release owns a closed document-operation parameter catalog

Core MUST provide one closed catalog for every caller-configurable document-operation parameter accepted by the release. The catalog MUST include common and adapter-scoped fields and own canonical identity, applicable CLI/env/config locators, standard value kind, defaults, merge strategy, operation binding, an optional exact static adapter-id marker, and a closed compile-time consumer binding. Every entry MUST target one compatible closed consumer; only strategy-visible values MUST target a compile-time field, typed accessor, or closed variant through the shared `StandardInputBinding`, while core/navigation-only controls MUST target navigation/core-owned closed variants and MUST NOT appear in adapter input. The catalog inventory for this change MUST be `page`, `limit`, `pagination.enabled`, `output`, and Markdown `max_heading_level`; adapter routing, document path/ref/query, `invocation_log`, and config-path selection flags MUST remain outside it. `pagination.enabled` and `limit` MUST normalize to the effective limit, while `output` MUST populate only `PreparedNavigationRequest` / core output projection. An untagged entry MUST be common; a tagged entry MUST apply only when its marker equals the selected adapter id. An env locator MUST mean that env is enabled for that field; without one, no env candidate is accepted for the field. Adding or removing an env locator is an observable product-input change. The catalog MUST also own whichever context-independent validation rules core executes before dispatch; it is not required to encode every adapter algorithm precondition. Catalog construction MUST reject duplicate or incompatible entries, unknown adapter ids, and missing or incompatible consumer bindings. Core code is the only authoring path for catalog entries.

#### Scenario: Core declares a Markdown-scoped parameter

- **WHEN** the release supports `max_heading_level` for Markdown outline and find
- **THEN** core catalog declares `--max-heading-level`, `options.docnav-markdown.max_heading_level`, integer range `1..=6`, default `3`, outline/find bindings, and exact adapter marker `docnav-markdown`
- **THEN** CLI, config inspection, navigation resolution, and request binding consume that same entry
- **THEN** Markdown adapter source does not declare the parameter
- **THEN** Markdown may repeat the range check before applying its strategy

#### Scenario: Add a future adapter-scoped parameter

- **WHEN** a built-in adapter needs a new caller-configurable document-operation parameter
- **THEN** the release change adds the parameter to core catalog and updates the adapter consumer together
- **THEN** loading or registering the adapter alone cannot expand accepted CLI, env, config, or protocol input

#### Scenario: Enable env for one product field

- **WHEN** an owner change enables environment input for a catalog field
- **THEN** it adds the exact environment locator to that field's core catalog entry
- **THEN** fields without an environment locator remain unaffected
- **THEN** the enabled field resolves env after explicit input and before project/user config

#### Scenario: Core defers context-dependent validation

- **WHEN** an adapter-scoped parameter has semantics that depend on document content or an algorithm-specific combination
- **THEN** core catalog still defines whether the parameter exists, its source locators, standard value kind, exact adapter-id marker when scoped, operation binding, default/merge behavior, and closed consumer binding
- **THEN** core may perform only the validation needed to construct that standard value
- **THEN** the selected adapter strategy validates the remaining semantic precondition without declaring a new parameter

#### Scenario: Non-product fields remain with their owners

- **WHEN** protocol, manifest, probe, result, ref, or adapter-private state requires typed validation
- **THEN** the owning contract or validation boundary may construct a dedicated `FieldDefSet`
- **THEN** that field does not become a caller-configurable document-operation parameter merely because it uses typed-fields

#### Scenario: Catalog binding is invalid

- **WHEN** an entry references an unknown adapter id or has a missing or incompatible closed consumer binding
- **THEN** core catalog construction fails deterministically
- **THEN** the invalid release definition cannot reach CLI parsing or navigation dispatch

### Requirement: Config inspect validates core-owned adapter-scoped parameters

Core CLI config inspection MUST treat `options.<adapter-id>.<parameter-key>` as a core-owned adapter-scoped parameter namespace. The adapter id segment MUST use the existing static registry id without aliases. Equal keys for different adapter ids MUST remain distinct catalog paths. Inspection MUST validate catalog membership, standard value materialization, and the static rules exposed by the core projection; it MUST NOT claim to have executed adapter-only or document-dependent semantics. Bare `options.<parameter-key>` paths MUST receive only normal unknown/invalid path handling.

#### Scenario: Inspect a Markdown-scoped value

- **WHEN** a selected config file contains `options.docnav-markdown.max_heading_level`
- **THEN** inspection resolves the exact adapter id and path against the core catalog
- **THEN** it validates the value using the core-owned field facts

#### Scenario: Inspect cannot execute adapter-only semantics

- **WHEN** a known adapter-scoped config value is well-typed but requires selected-document context for semantic validation
- **THEN** inspection reports it as a known materializable catalog value
- **THEN** it does not invent an adapter declaration or claim that runtime semantic validation has completed

#### Scenario: Same key for different adapters remains distinct

- **WHEN** selected config sources contain `options.docnav-markdown.mode` and `options.docnav-other.mode`
- **THEN** config inspection keeps both paths distinct
- **THEN** one core catalog entry does not validate or rewrite the other adapter namespace

#### Scenario: Bare adapter-scoped path is unknown

- **WHEN** a selected config source contains `options.max_heading_level`
- **THEN** inspection treats that path through normal unknown/invalid config handling
- **THEN** it does not infer an adapter id or apply migration behavior

## MODIFIED Requirements

### Requirement: Core release owns static adapter implementation registry

Core CLI MUST provide the core release static registry that binds adapter identity, descriptor metadata, capability declarations, implementation source, and a linked implementation of the fixed strategy interface. Public document-operation parameter facts MUST come from the separate core-owned catalog.

#### Scenario: Document operation selects an adapter

- **WHEN** a document operation needs adapter candidates
- **THEN** core supplies the static registry to navigation
- **THEN** implementation source comes from the current release
- **THEN** parameter definitions come from the separate core catalog rather than the selected registry entry

### Requirement: Config inspect validates through parameter aggregation metadata

Core CLI config inspection MUST validate config source keys and values through the config-source projection produced from the core-owned parameter catalog where that projection expresses the field. The inspection output MUST report invalid value kind, nullability, unknown catalog path, and any enum, range, shape, or other constraints selected for core validation without reimplementing those field semantics in core CLI. It MAY leave adapter-only or document-dependent semantic validation to runtime strategy dispatch. Object/array shape diagnostics for current config arrays MAY remain owner-specific when existing owner validation already preserves source path and parity with navigation resolution.

#### Scenario: Inspect reports invalid typed value

- **WHEN** a selected config file contains `defaults.pagination.limit` with value `0`
- **THEN** `docnav config inspect` validates the value through the config-source projection produced from the core catalog
- **THEN** the output identifies `defaults.pagination.limit`, the selected source path, and the typed validation reason

#### Scenario: Inspect reports invalid adapter-scoped value

- **WHEN** a selected config file contains `options.docnav-markdown.max_heading_level` outside `1..=6`
- **THEN** config inspect validates that path through the same core catalog used by navigation
- **THEN** the output identifies the catalog path, selected source, and range reason without consulting an adapter declaration

#### Scenario: Inspect reports nested shape failure

- **WHEN** a selected config file contains an invalid `outline.mode_rules[]` or `outline.thresholds` compound shape
- **THEN** `docnav config inspect` combines the full scalar catalog projection with the current owner-specific compound validator
- **THEN** the compound algorithm is not required to become ordinary scalar catalog fields
- **THEN** the output identifies the nested config path and source path

#### Scenario: Inspect reports only its validation coverage

- **WHEN** a value passes catalog materialization and configured core checks but has adapter-specific runtime semantics
- **THEN** config inspect may report the value as structurally valid
- **THEN** the selected adapter strategy remains responsible for any semantic check not guaranteed by the core projection

## REMOVED Requirements

### Requirement: Config inspect preserves adapter option ownership

**Reason**: Product parameter authoring moves to the core-owned closed catalog. Adapter definitions no longer own declarations that config inspection can discover or validate.

**Migration**: Preserve `options.<adapter-id>.<key>` path isolation and strict unknown-path behavior through the new core-owned adapter-scoped catalog requirement. Remove registry-backed adapter declaration lookup. Adapter strategy diagnostics for runtime semantic failures remain valid and do not act as config declarations.
