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
Core CLI MUST provide the core release static registry that binds adapter identity, linked implementation source, descriptor metadata, native option declarations, and operation handlers.

#### Scenario: Document operation selects an adapter
- **WHEN** a document operation needs adapter candidates
- **THEN** core supplies the static registry to navigation
- **THEN** implementation source comes from the current release

### Requirement: Core CLI selects output mode and process exit behavior
Core CLI MUST parse requested output mode, call the output contract for projection, and map diagnostics to process exit behavior without redefining protocol or readable payload semantics.

#### Scenario: Protocol-json requested
- **WHEN** a caller requests `--output protocol-json`
- **THEN** core asks the protocol/output pipeline for protocol stdout
- **THEN** readable-view framing is not emitted to stdout

#### Scenario: Failure exit
- **WHEN** a document operation fails
- **THEN** core exits according to the CLI exit mapping for the diagnostic class
- **THEN** the failure payload remains owned by diagnostics and output/protocol contracts

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

Core CLI config inspection MUST validate config source keys and values through the config-source projection produced by owner-provided parameter aggregation metadata where that projection expresses the field. The inspection output MUST report invalid value kind, enum, range, nullability, adapter declaration, and owner-specific config constraint failures without reimplementing field semantics in core CLI. Object/array shape diagnostics for current config arrays MAY remain owner-specific when existing owner validation already preserves source path and parity with navigation resolution.

#### Scenario: Inspect reports invalid typed value

- **WHEN** a selected config file contains `defaults.pagination.limit` with value `0`
- **THEN** `docnav config inspect` validates the value through the config-source projection produced by parameter aggregation
- **THEN** the output identifies `defaults.pagination.limit`, the selected source path, and the typed validation reason

#### Scenario: Inspect reports nested shape failure

- **WHEN** a selected config file contains an invalid `outline.mode_rules[]` item shape
- **THEN** `docnav config inspect` validates the nested config source shape through the current owner validation path or config-source projection for that supported subset
- **THEN** the output identifies the nested config path and source path

### Requirement: Config inspect preserves adapter option ownership

Core CLI config inspection MUST treat `options.<adapter-id>.<option-key>` keys as adapter-owned native option sources. The adapter id segment MUST be resolved using the existing adapter registry id without aliases. Equal option keys from different adapter ids MUST remain distinct config paths. Bare `options.<option-key>` paths MUST NOT receive migration, compatibility, or special diagnostic behavior beyond the normal unknown/invalid config path handling.

#### Scenario: Adapter-id native option is inspected

- **WHEN** a selected config file contains `options.docnav-markdown.max_heading_level`
- **THEN** inspection resolves `docnav-markdown` through the adapter registry-backed metadata projection
- **THEN** inspection validates that value through the Markdown adapter option declaration when metadata is available

#### Scenario: Same option key in different adapters is deterministic

- **WHEN** selected config sources contain `options.docnav-markdown.mode` and `options.docnav-other.mode`
- **THEN** config inspection keeps both paths distinct
- **THEN** declarations from one adapter id do not validate or rewrite the other adapter id namespace

#### Scenario: Bare native option path is a normal unknown path

- **WHEN** a selected config source contains `options.max_heading_level`
- **THEN** config inspection treats that path through the normal unknown/invalid config path handling
- **THEN** inspection does not infer an adapter id, rewrite the path, or apply migration behavior

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
