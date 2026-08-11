**本 delta 只定义 core CLI 如何用 optional find path 选择单文档或 project scope，并保持单文档 command、auto-read 和 process mapping 不变；它尚未通过实现前阻断审计。**

## MODIFIED Requirements

### Requirement: Core CLI exposes document operation commands
Core CLI MUST expose outline, read, find, and info command entry points and route them into the shared document operation pipeline. `find` MUST accept an optional document path: a supplied path selects the existing single-document pipeline, while an omitted path selects project find under the current resolved project root.

#### Scenario: Outline command
- **WHEN** a caller runs `docnav outline <path>`
- **THEN** core parses the command shape
- **THEN** it hands the document operation input to navigation

#### Scenario: Find with a path remains single-document
- **WHEN** a caller runs `docnav find <path> --query <text>`
- **THEN** core selects single-document find
- **AND** it preserves the existing path validation and document operation pipeline

#### Scenario: Find without a path selects the current project
- **WHEN** a caller runs `docnav find --query <text>`
- **THEN** core selects project find
- **AND** it hands the current resolved project root to navigation

#### Scenario: Find help distinguishes the two scopes
- **WHEN** a caller requests `docnav find --help`
- **THEN** help shows path as optional
- **AND** states that omission searches the current resolved project root
- **AND** does not advertise inferred routing or a directory-path alias

### Requirement: Core CLI normalizes document and project paths
Core CLI MUST normalize document paths, project context, and command cwd behavior before handing path facts to downstream owners. Explicit document paths MUST retain the existing normalization and file-access boundary. A pathless project find MUST hand the already resolved project root to navigation without inventing a synthetic document path.

#### Scenario: File outside project root
- **WHEN** a caller references a document outside the project root
- **THEN** core normalizes the document path
- **THEN** downstream owners receive a stable path fact rather than raw argv text

#### Scenario: Pathless find reuses project-root resolution
- **WHEN** a caller omits the find path
- **THEN** core resolves the project root by searching upward for the nearest `.docnav/` and otherwise using the invocation cwd
- **AND** navigation receives that exact project root as the discovery boundary

#### Scenario: Explicit directory is not a project alias
- **WHEN** a caller supplies a directory as the find path
- **THEN** core returns the existing document-path diagnostic
- **AND** it does not convert the explicit path into project scope

### Requirement: unique-ref auto-read is enabled by default

Core CLI MUST expose `--auto-read disabled|unique-ref` for `outline` and single-document `find`. Project and user config MUST accept `defaults.auto_read` with the same exact values. The built-in default MUST be `unique-ref` for eligible single-document operations. Pathless project find MUST NOT materialize that default or accept an explicit `--auto-read` candidate.

#### Scenario: outline and single-document find expose the exact mode
- **WHEN** a caller requests help for `outline` or `find`
- **THEN** help includes `--auto-read <disabled|unique-ref>`
- **AND** help identifies `unique-ref` as the built-in default for eligible single-document scope
- **AND** help identifies project find as auto-read ineligible
- **AND** no other auto-read token is advertised

#### Scenario: omitted mode enables unique-ref orchestration
- **WHEN** a caller omits `--auto-read` for `outline` or single-document `find`
- **THEN** core resolves the mode as `unique-ref`
- **AND** projects it to document orchestration

#### Scenario: disabled mode preserves the base command
- **WHEN** a caller passes `--auto-read disabled` to an eligible single-document operation
- **THEN** core executes only the existing base operation
- **AND** the success result contains no `auto_read` field

#### Scenario: explicit unique-ref supports both document output modes
- **WHEN** a caller invokes `outline` or single-document `find` with `--auto-read unique-ref`
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

#### Scenario: project find rejects explicit auto-read before discovery
- **WHEN** a caller omits the find path and supplies `--auto-read`
- **THEN** core returns the existing scope-inapplicable input diagnostic
- **AND** project discovery and adapter dispatch do not start

#### Scenario: project find does not inherit configured auto-read
- **WHEN** a caller omits the find path
- **AND** project or user config contains a valid `defaults.auto_read` value
- **THEN** full config validation still recognizes the field
- **AND** project-scope resolution does not materialize or dispatch auto-read

#### Scenario: invalid mode rejects the invocation before dispatch
- **WHEN** a caller passes an auto-read value other than `disabled` or `unique-ref`
- **THEN** core returns `INVALID_REQUEST`
- **AND** no adapter operation is dispatched

## ADDED Requirements

### Requirement: Project find maps local and fatal outcomes separately

Core CLI MUST map a validated project find result, including a result with document-scoped failures, to success exit code `0`. Failures that prevent project orchestration or output from producing a validated result MUST retain the existing diagnostic-to-exit mapping.

#### Scenario: Mixed project result exits successfully
- **WHEN** project find returns matches for one document and a bounded local failure for another
- **THEN** core emits one validated find success response
- **AND** the process exits with `0`

#### Scenario: Project-root failure uses the existing failure mapping
- **WHEN** the resolved project root cannot be enumerated
- **THEN** core emits the mapped top-level document failure
- **AND** the process uses the existing non-zero exit code for that diagnostic
