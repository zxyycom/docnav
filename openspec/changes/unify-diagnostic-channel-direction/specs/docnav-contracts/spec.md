本 spec delta 记录强制迁移到统一 `DiagnosticStack` 的目标 contract；当前内容只是 `openspec/changes/unify-diagnostic-channel-direction/` 下的 change-local docnav-contracts delta，本文件本身不立即修改主规范、schema、示例或实现行为。

## ADDED Requirements

### Requirement: Diagnostic handoff uses a unified internal stack

Docnav document execution, adapter direct document execution, adapter invoke request handling, standard parameter resolution, non-document commands, and document output orchestration MUST hand off recoverable diagnostics and fatal errors through a unified internal `DiagnosticStack` before surface-specific output is written.

#### Scenario: Recoverable diagnostic continues operation

- **WHEN** an operation encounters a recoverable condition such as ignored CLI argv, skipped adapter candidate, or skipped config source
- **THEN** the condition is pushed as a diagnostic event before the caller decides how to proceed
- **THEN** execution continues when the remaining operation input is valid
- **THEN** the recoverable diagnostic remains available in the internal diagnostic stack until the output owner renders or flushes it

#### Scenario: Fatal diagnostic preserves mechanical diagnostic code

- **WHEN** an operation encounters a fatal request, document, adapter boundary, or internal failure
- **THEN** the internal diagnostic stack can retain diagnostic context for that failure before the fatal outcome is returned
- **THEN** the event carries a diagnostic code that can be projected to the target surface error code, message, details, guidance, and exit-code category

#### Scenario: Caller can inspect accumulated diagnostics before output

- **WHEN** execution crosses a core, SDK, adapter, or output boundary before final stdout or stderr is written
- **THEN** the caller can inspect the diagnostics accumulated so far
- **THEN** inspecting diagnostics does not consume or drop them before final output policy runs

### Requirement: Diagnostic stack entries have internal identities and checkpoints

The internal diagnostic stack MUST provide scoped identities and checkpoints so callers can inspect specific pushed events and drain or flush diagnostics created after a known point without exposing stack implementation details as public output contract.

#### Scenario: Pushed diagnostic can be retrieved by id

- **WHEN** a caller pushes a diagnostic event onto the stack
- **THEN** the stack returns an opaque `DiagnosticId` scoped to that stack lifetime
- **THEN** a caller holding that id can retrieve the same event without popping or consuming it
- **THEN** callers cannot provide their own `DiagnosticId` value when pushing

#### Scenario: Caller drains diagnostics after a checkpoint

- **WHEN** a caller creates a `DiagnosticMark` before trying a candidate path
- **AND** that candidate path pushes one or more diagnostics
- **THEN** the caller can drain diagnostics pushed after the mark as a batch
- **THEN** diagnostics that existed before the mark remain in the stack

#### Scenario: Caller drains diagnostics after an event id

- **WHEN** a caller holds the `DiagnosticId` for an earlier stack event
- **THEN** the caller can choose whether draining starts strictly after that event id or includes the event referenced by that id
- **THEN** the event referenced by the id remains available unless the caller explicitly removes it

#### Scenario: Stack retrieval is LIFO by default

- **WHEN** a caller pops, drains, snapshots, renders, or flushes stack entries without requesting another order
- **THEN** the stack returns entries in last-in-first-out order
- **THEN** a caller that needs insertion order or grouped output must explicitly reverse or regroup the returned entries outside the stack

#### Scenario: Stack lifetime does not cross process boundary

- **WHEN** a top-level command, adapter direct command, or invoke request creates a diagnostic stack
- **THEN** stack ids and marks are valid only for that in-process stack lifetime
- **THEN** serialized protocol/readable output does not expose stack ids, marks, or indexes as durable refs

### Requirement: Diagnostic code owns stable error identity

Diagnostic events MUST carry the mechanical code used for both warning and failure identity. Legacy stable error objects MUST NOT be embedded in the diagnostic stack or remain the owning model for error identity.

#### Scenario: Fatal surface error is projected from diagnostic code

- **WHEN** a surface owner renders or serializes a fatal diagnostic event
- **THEN** the target surface error code is derived from the diagnostic event code
- **THEN** the surface owner projects that code into the target surface without requiring `StableError` as an intermediate owner

#### Scenario: Warning surface output is projected from diagnostic code

- **WHEN** a surface owner renders or serializes a recoverable diagnostic event as a warning
- **THEN** the warning id is derived from the diagnostic event code
- **THEN** the warning projection uses event effect and details from the diagnostic stack

### Requirement: Surface output policy migrates to DiagnosticStack

The unified internal diagnostic stack is a breaking migration target. Each observable surface MUST switch to a DiagnosticStack-based output policy, and affected docs, schemas, examples, fixtures, and consumer tests MUST be updated in the same implementation work.

#### Scenario: Protocol JSON uses updated diagnostic projection

- **WHEN** a document operation is rendered as `protocol-json`
- **THEN** stdout follows the updated protocol response schema for the DiagnosticStack migration
- **THEN** any protocol-visible diagnostic fields or errors are derived from stack entries

#### Scenario: Readable output derives diagnostics from stack entries

- **WHEN** a document operation is rendered as `readable-view` or `readable-json`
- **THEN** recoverable diagnostics that have a readable warning projection are rendered from stack entries
- **THEN** the readable output wrapper remains separate from the protocol response envelope

#### Scenario: Adapter machine command output uses updated schema

- **WHEN** an adapter direct `manifest`, `probe`, or `invoke` command writes machine output
- **THEN** stdout follows the updated owning manifest, probe, or protocol response schema
- **THEN** diagnostic output is derived from stack entries according to that updated surface policy

### Requirement: Direct stderr diagnostics are replaced by stack events

Rust entry points that currently write direct diagnostic text to stderr MUST migrate to creating diagnostic events first and flushing them through the owning output policy when this change enters implementation.

#### Scenario: Direct CLI input error records diagnostic before stderr

- **WHEN** adapter direct CLI rejects command shape before a document operation can run
- **THEN** the entry point records a diagnostic event with the rejected field or command context
- **THEN** the direct CLI stderr output can be produced from that event according to the new DiagnosticStack-based surface policy

#### Scenario: Process-boundary decode failure records diagnostic context

- **WHEN** adapter invoke request reading, JSON parsing, schema validation, typed deserialization, or semantic validation fails
- **THEN** the failure records diagnostic context before output is written
- **THEN** the observable response follows the updated DiagnosticStack-based protocol failure policy

### Requirement: Diagnostic-only warning effect is deferred

The unified diagnostic channel migration MUST NOT add `diagnostic_only` unless a concrete behavior requires it.

#### Scenario: Diagnostic-only effect is not required

- **WHEN** no current behavior requires `diagnostic_only`
- **THEN** Rust warning or diagnostic event enums omit that effect
- **THEN** readable schema and examples remove or avoid that effect

#### Scenario: Diagnostic-only effect is added later

- **WHEN** a later behavior requires `diagnostic_only`
- **THEN** that work adds enum, schema, examples, formatter tests, and smoke assertions together
