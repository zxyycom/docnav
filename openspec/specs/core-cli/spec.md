# core-cli Specification

## Purpose
Define the `docnav` core CLI process surface: executable delivery, command parsing, document operation command entry, non-navigation management commands, project and config path handling, static adapter registry ownership, output mode selection, and process exit behavior. Navigation input resolution, adapter semantics, protocol envelopes, readable rendering, refs, diagnostics, and validation materials are separate owner capabilities consumed by the CLI.
## Requirements
### Requirement: Core CLI is the standalone `docnav` executable
Docnav MUST ship a standalone `docnav` executable that is the default user-facing entry for document operations and core management commands.

#### Scenario: User invokes docnav
- **WHEN** a user runs `docnav version`
- **THEN** the executable responds without requiring an adapter-specific binary

### Requirement: Core CLI exposes document operation commands
Core CLI MUST expose outline, read, find, and info command entry points and route them into the shared document operation pipeline.

#### Scenario: Outline command
- **WHEN** a caller runs `docnav outline <path>`
- **THEN** core parses the command shape
- **THEN** it hands the document operation input to navigation

### Requirement: Core CLI strictly handles public argv
Core CLI MUST reject unknown, duplicate, unsupported, or unused public arguments according to the strict argv policy owned by CLI.

#### Scenario: Unknown flag
- **WHEN** a caller passes an unknown flag to a document command
- **THEN** core reports an invalid request diagnostic
- **THEN** the invalid flag is not forwarded to navigation or adapters

### Requirement: Core CLI normalizes document and project paths
Core CLI MUST normalize document paths, project context, and command cwd behavior before handing path facts to downstream owners.

#### Scenario: File outside project root
- **WHEN** a caller references a document outside the project root
- **THEN** core normalizes the document path
- **THEN** downstream owners receive a stable path fact rather than raw argv text

### Requirement: Core CLI resolves config file paths
Core CLI MUST resolve user and project config file paths from explicit flags and defaults, then pass config source descriptors and paths to navigation or config commands.

#### Scenario: Explicit project config
- **WHEN** a caller passes `--project-config <path>`
- **THEN** core resolves that path as the project config source
- **THEN** navigation receives the descriptor and path with explicit-source attribution

#### Scenario: Default user config
- **WHEN** a caller omits `--user-config`
- **THEN** core uses the configured platform default user config location
- **THEN** absence of that default source remains distinguishable from an invalid present source

### Requirement: Core management commands have bounded behavior
Core CLI MUST provide bounded management commands such as config, init, doctor, adapter inspection, and version without expanding them into dynamic adapter package management by default.

#### Scenario: Adapter list
- **WHEN** a caller runs adapter inspection
- **THEN** core reports adapters from the static registry
- **THEN** inspection is complete from release-local registry facts

### Requirement: Core release owns static adapter implementation registry

Core CLI MUST provide the core release static registry that binds adapter identity, descriptor metadata, capability declarations, implementation source, and a linked implementation of the fixed strategy interface. Public document-operation parameter facts MUST come from the separate core-owned catalog.

#### Scenario: Document operation selects an adapter

- **WHEN** a document operation needs adapter candidates
- **THEN** core supplies the static registry to navigation
- **THEN** implementation source comes from the current release
- **THEN** parameter definitions come from the separate core catalog rather than the selected registry entry

### Requirement: Core CLI selects output mode and process exit behavior

Core CLI MUST accept exactly the document output modes `readable-view` and `protocol-json`, construct the corresponding output plan, and map failures to existing process exit behavior. Omitted output or `readable-view` MUST construct `Rendered` with the built-in renderer. `protocol-json` MUST construct `ProtocolJson`. A document failure that occurs before navigation returns a response MUST be projected into `ProtocolResponse::Failure` before the selected output plan executes.

#### Scenario: Omitted output uses the built-in renderer

- **WHEN** a caller omits output mode for a document operation
- **THEN** core constructs `Rendered` with the built-in `readable-view` renderer

#### Scenario: Explicit readable-view uses the built-in renderer

- **WHEN** a caller requests `--output readable-view`
- **THEN** core constructs `Rendered` with the built-in renderer

#### Scenario: Protocol JSON bypasses rendering

- **WHEN** a caller requests `--output protocol-json`
- **THEN** core constructs `ProtocolJson`
- **THEN** the protocol response is emitted without invoking a renderer

#### Scenario: Early document failure follows the recognized output mode

- **WHEN** a document failure occurs before navigation returns a response
- **THEN** core constructs `ProtocolResponse::Failure` through the existing protocol error projection
- **THEN** `ProtocolJson` serializes that response or `Rendered` passes it to the built-in renderer

#### Scenario: Removed readable-json value is rejected

- **WHEN** CLI or config input supplies `readable-json`
- **THEN** core reports the normal invalid-value diagnostic
- **THEN** no readable-json plan、alias or fallback is constructed

#### Scenario: Non-document output remains unchanged

- **WHEN** help、version or another non-document command succeeds
- **THEN** core keeps that command's existing owner-specific output behavior

### Requirement: Core CLI preserves adapter business semantics
Core CLI MUST preserve adapter-owned document facts such as refs, content type, item facts, and adapter diagnostics when routing results to output.

#### Scenario: Adapter returns content type
- **WHEN** an adapter returns `content_type`
- **THEN** core forwards the adapter-owned fact unchanged
- **THEN** output surfaces can project the same fact

### Requirement: Config surface is read-only inspect

Core CLI MUST expose `docnav config inspect` as the only long-term config subcommand. The command MUST NOT mutate config files, accept key/value edits, delete fields, or preserve single-key get/list editor semantics. Legacy `docnav config get`, `docnav config set`, `docnav config unset`, and `docnav config list` MUST be removed as accepted subcommands in this breaking change.

#### Scenario: Config inspect reports selected sources

- **WHEN** a caller runs `docnav config inspect`
- **THEN** core CLI obtains the selected project and user config source facts through the shared config source selection/loading primitives
- **THEN** the output includes each source's scope, path, origin, existence/load state, and source-attributed validation diagnostics or a bounded diagnostic summary when present
- **THEN** no config file is modified

#### Scenario: Legacy config mutators are not accepted

- **WHEN** a caller runs `docnav config set defaults.output readable-json`
- **THEN** core CLI rejects the subcommand through the normal CLI parse/error boundary
- **THEN** no config file is modified

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

### Requirement: Config inspect remains source-scoped

Core CLI config inspection MUST remain a source inspection command. The command MUST report selected config sources, source summaries, load states, source-attributed validation diagnostics, and currently resolvable parameter facts. Adapter-id namespaced fields MAY appear as source fields validated through parameter aggregation metadata; selected adapter/operation dispatch remains owned by navigation input resolution.

#### Scenario: Adapter-id option appears as source field

- **WHEN** a selected config source contains `options.docnav-markdown.max_heading_level`
- **AND** a caller runs `docnav config inspect`
- **THEN** the output reports that path as a source field with validation facts and currently resolvable parameter facts when metadata is available
- **THEN** no config file is modified

#### Scenario: Inspect does not preview dispatch

- **WHEN** a selected config source contains `options.docnav-markdown.max_heading_level`
- **AND** a caller runs `docnav config inspect` without invoking a document operation
- **THEN** the output reports source validation facts and any parameter facts currently resolvable from the selected sources
- **THEN** the output does not claim that the value was dispatched to an adapter

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

### Requirement: unique-ref auto-read is enabled by default

Core CLI MUST expose `--auto-read disabled|unique-ref` for `outline` and `find`. Project and user config MUST accept `defaults.auto_read` with the same exact values. The built-in default MUST be `unique-ref`.

#### Scenario: outline and find expose the exact mode
- **WHEN** a caller requests help for `outline` or `find`
- **THEN** help includes `--auto-read <disabled|unique-ref>`
- **AND** help identifies `unique-ref` as the built-in default
- **AND** no other auto-read token is advertised

#### Scenario: omitted mode enables unique-ref orchestration
- **WHEN** a caller omits `--auto-read` for `outline` or `find`
- **THEN** core resolves the mode as `unique-ref`
- **AND** projects it to document orchestration

#### Scenario: disabled mode preserves the base command
- **WHEN** a caller passes `--auto-read disabled`
- **THEN** core executes only the existing base operation
- **AND** the success result contains no `auto_read` field

#### Scenario: explicit unique-ref supports both document output modes
- **WHEN** a caller invokes `outline` or `find` with `--auto-read unique-ref`
- **AND** selects either `readable-view` or `protocol-json`
- **THEN** core accepts the invocation and projects the resolved mode to document orchestration

#### Scenario: config inspect recognizes the auto-read field
- **WHEN** selected project or user config contains `defaults.auto_read`
- **THEN** `docnav config inspect` reports the canonical auto-read field and source candidate through its existing config-source projection
- **AND** inspection does not construct a document operation or trigger auto-read

#### Scenario: unsupported command rejects the mode before dispatch
- **WHEN** a caller passes `--auto-read` to `read`, `info` or a non-document command
- **THEN** core returns the existing strict input diagnostic
- **AND** no adapter operation is dispatched

#### Scenario: invalid mode rejects the invocation before dispatch
- **WHEN** a caller passes an auto-read value other than `disabled` or `unique-ref`
- **THEN** core returns `INVALID_REQUEST`
- **AND** no adapter operation is dispatched

### Requirement: silent auto-read preserves root exit and logging ownership

Core CLI MUST keep the root invocation operation as `outline` or `find`. When base success does not produce a successful composed auto-read result, the root response MUST remain the validated base success and the process exit code MUST remain `0`.

#### Scenario: nested read diagnostic is silent
- **WHEN** the base outline or find succeeds
- **AND** the unique-ref nested read does not return a validated success
- **THEN** the root response is the unchanged base success
- **AND** the result contains no `auto_read`
- **AND** the process exits with `0`

#### Scenario: invocation logging keeps one root operation
- **WHEN** invocation logging is enabled for auto-read outline or find
- **THEN** the main operation event remains owned by the root outline or find invocation
- **AND** core does not emit a second top-level read invocation event

#### Scenario: explicit content capture handles successful auto-read
- **WHEN** unique-ref auto-read successfully adds read content
- **AND** invocation content capture is explicitly enabled
- **THEN** the added read content uses the existing hashed content-capture path and event shape
- **AND** the main JSONL event does not inline the content
