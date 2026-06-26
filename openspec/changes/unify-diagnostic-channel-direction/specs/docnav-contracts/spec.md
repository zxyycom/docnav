本 spec delta 记录统一 diagnostic channel 的目标 contract；当前内容只是 `openspec/changes/unify-diagnostic-channel-direction/` 下的 change-local docnav-contracts delta，不改变现有主规范、schema、示例或实现行为。

## ADDED Requirements

### Requirement: Diagnostic handoff uses a unified internal channel

Docnav document execution, adapter direct document execution, adapter invoke request handling, standard parameter resolution, and document output orchestration MUST be able to hand off recoverable diagnostics and fatal errors through a unified internal diagnostic collection before surface-specific output is written.

#### Scenario: Recoverable diagnostic continues operation

- **WHEN** an operation encounters a recoverable condition such as ignored CLI argv, skipped adapter candidate, or skipped config source
- **THEN** execution continues when the remaining operation input is valid
- **THEN** the recoverable diagnostic remains available in the internal diagnostic collection until the output owner renders or flushes it

#### Scenario: Fatal diagnostic preserves stable error mapping

- **WHEN** an operation encounters a fatal request, document, adapter boundary, or internal failure
- **THEN** the internal diagnostic collection can retain diagnostic context for that failure
- **THEN** the observable failure still maps to the owning `StableError` code, details, guidance, and exit-code category

#### Scenario: Caller can inspect accumulated diagnostics before output

- **WHEN** execution crosses a core, SDK, adapter, or output boundary before final stdout or stderr is written
- **THEN** the caller can inspect the diagnostics accumulated so far
- **THEN** inspecting diagnostics does not consume or drop them before final output policy runs

### Requirement: Surface output policy remains owner-specific during compatible migration

The unified internal diagnostic channel MUST NOT by itself change protocol response, manifest, probe, readable output, stderr, or exit-code behavior. Each observable surface MUST continue to use its owning output policy until a later explicit contract change modifies that surface.

#### Scenario: Protocol JSON stdout remains pure

- **WHEN** a document operation is rendered as `protocol-json`
- **THEN** stdout contains only the protocol response envelope allowed by the protocol response schema
- **THEN** recoverable diagnostics are not added to stdout unless a later explicit breaking contract change updates the protocol schema and examples

#### Scenario: Readable output keeps warning projection

- **WHEN** a document operation is rendered as `readable-view` or `readable-json`
- **THEN** recoverable diagnostics that have a readable warning projection are rendered using the existing stable warning envelope
- **THEN** the readable output wrapper remains separate from the protocol response envelope

#### Scenario: Adapter machine command stdout remains schema-owned

- **WHEN** an adapter direct `manifest`, `probe`, or `invoke` command writes machine output
- **THEN** stdout remains limited to the owning manifest, probe, or protocol response payload
- **THEN** diagnostics are written through the owning stderr policy unless a later explicit contract change updates that machine payload schema

### Requirement: Direct stderr diagnostics are replaceable by collected events

Rust entry points that currently write direct diagnostic text to stderr MUST migrate toward creating diagnostic events first and flushing them through the owning output policy when this change enters implementation, while preserving existing user-visible text and stdout boundaries during compatible migration.

#### Scenario: Direct CLI input error records diagnostic before stderr

- **WHEN** adapter direct CLI rejects command shape before a document operation can run
- **THEN** the entry point records a diagnostic event with the rejected field or command context
- **THEN** the direct CLI stderr output can be produced from that event without changing the current stdout payload boundary

#### Scenario: Process-boundary decode failure records diagnostic context

- **WHEN** adapter invoke request reading, JSON parsing, schema validation, typed deserialization, or semantic validation fails
- **THEN** the failure records diagnostic context before output is written
- **THEN** the observable response still follows the protocol failure envelope required by the invoke surface

### Requirement: Diagnostic-only warning effect is resolved before implementation

Before implementing the unified diagnostic channel, the project MUST resolve whether `diagnostic_only` is a supported warning effect in Rust and public readable schemas.

#### Scenario: Diagnostic-only effect is adopted

- **WHEN** the project decides `diagnostic_only` is a supported warning effect
- **THEN** Rust warning or diagnostic event enums include that effect
- **THEN** readable schema, examples, formatter tests, and smoke assertions agree on when the effect is emitted

#### Scenario: Diagnostic-only effect is rejected

- **WHEN** the project decides `diagnostic_only` is not part of the current contract
- **THEN** readable schema and examples remove or avoid that effect
- **THEN** Rust warning enums and smoke assertions remain limited to supported effects
