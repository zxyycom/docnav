本 delta spec 定义 `replace-probe-traversal-with-inferred-routing` 对 `core-cli` 尚未应用的 Target：automatic pathname routing 发生在目标文档 filesystem I/O 之前，route 命中后的 normalized document path 继续交给既有下游契约；它不表示 Current 主规范或实现已经迁移。

## MODIFIED Requirements

### Requirement: Core CLI normalizes document and project paths

Core CLI MUST derive an invocation-private routing pathname lexically from the caller document path and command cwd before handing automatic-routing facts to navigation. This derivation MUST NOT inspect target-document metadata, open or read the target, or canonicalize it through the filesystem. If automatic routing selects a registered adapter, or caller intent explicitly selects one, core and navigation MUST then perform the filesystem-backed document path/access normalization required by the operation before constructing closed standard input or dispatching the selected strategy. Downstream operation owners MUST receive the stable normalized document path rather than raw argv text. If automatic routing finds no pathname hint, the invocation MUST fail with the routing pathname and MUST NOT perform target-document filesystem I/O merely to normalize or validate a document that will not be dispatched.

#### Scenario: File outside project root

- **WHEN** a caller references a routable document outside the project root
- **THEN** core first derives its routing pathname without target-document filesystem I/O
- **THEN** navigation selects the adapter from the lexical basename
- **THEN** core normalizes the document path after selection
- **THEN** downstream operation owners receive a stable path fact rather than raw argv text

#### Scenario: Unknown basename stops before document I/O

- **WHEN** automatic routing matches neither an exact filename nor a manifest suffix for the lexical basename
- **THEN** the invocation returns the pathname no-match diagnostic using the routing pathname
- **THEN** core does not inspect target metadata, open or read the target, or canonicalize it through the filesystem

#### Scenario: Routable pathname has a filesystem failure

- **WHEN** automatic or explicit selection identifies one registered adapter
- **AND** post-selection document path/access normalization fails
- **THEN** core returns the existing path or access diagnostic for that filesystem-backed failure
- **THEN** no adapter operation is dispatched
- **THEN** routing does not try another adapter

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

- **WHEN** protocol, manifest, result, ref, or adapter-private state requires typed validation
- **THEN** the owning contract or validation boundary may construct a dedicated `FieldDefSet`
- **THEN** that field does not become a caller-configurable document-operation parameter merely because it uses typed-fields

#### Scenario: Catalog binding is invalid

- **WHEN** an entry references an unknown adapter id or has a missing or incompatible closed consumer binding
- **THEN** core catalog construction fails deterministically
- **THEN** the invalid release definition cannot reach CLI parsing or navigation dispatch
