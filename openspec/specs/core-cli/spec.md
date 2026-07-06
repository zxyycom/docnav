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
