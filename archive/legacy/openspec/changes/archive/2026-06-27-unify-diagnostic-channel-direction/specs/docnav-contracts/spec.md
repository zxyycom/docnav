本 spec delta 根据 `docs/diagnostics.md` 的目标形态记录错误通道 contract：运行时问题进入请求内栈，`DiagnosticCode` 是唯一机械身份来源，每个 code 拥有 canonical details，边界 surface 读取记录并投影。该 delta 归属 `docnav-contracts`，随 change 应用后进入主 spec。

## ADDED Requirements

### Requirement: Runtime problems flow through a request-local diagnostic stack

Docnav runtime and public surface code MUST record runtime problems in a request-local diagnostic stack before the owning boundary decides whether to continue, fail, exit, or write surface-specific output.

#### Scenario: Recoverable problem is recorded before continuation

- **WHEN** an operation encounters a recoverable condition such as ignored CLI argv, skipped adapter candidate, skipped config source, or recoverable adapter evidence
- **THEN** the condition is pushed as a diagnostic record before the caller decides how to proceed
- **THEN** execution continues when the remaining operation input is valid
- **THEN** the record remains available until the boundary owner reads, renders, or flushes it

#### Scenario: Fatal problem records context before failure surface

- **WHEN** an operation encounters a fatal request, document, adapter boundary, or internal failure
- **THEN** the diagnostic stack records the fatal context before the fatal outcome is returned or propagated
- **THEN** the record carries a diagnostic code that can be projected to the target surface error code, message, details, guidance, and exit-code category

#### Scenario: Diagnostic stack stores facts only

- **WHEN** a diagnostic record is pushed
- **THEN** the stack stores the record without deciding whether the operation succeeds or fails
- **THEN** the caller or surface owner decides continuation, failure, output format, output channel, and exit behavior

### Requirement: DiagnosticCode owns identity and canonical details

`docnav-diagnostics` MUST own `DiagnosticCode`, grouped code families, each code's canonical details object, and projection metadata. Other Docnav crates MUST use those diagnostics-owned identities and MUST NOT redefine protocol, readable, adapter, or standard-parameter diagnostic code identities.

#### Scenario: Diagnostic code aggregates grouped enums

- **WHEN** implementation groups diagnostic codes by purpose, producer, or projection family
- **THEN** each group can use its own manually maintained enum
- **THEN** the top-level `DiagnosticCode` aggregates those group enums into one mechanical identity domain
- **THEN** callers outside `docnav-diagnostics` use the top-level `DiagnosticCode` or its explicit family conversions

#### Scenario: Diagnostic code owns warning and error identity

- **WHEN** a diagnostic record is rendered as a warning, fatal error, readable warning id, protocol error code, stderr line, or other surface field
- **THEN** the mechanical identity is derived from the record's `DiagnosticCode`
- **THEN** the surface field does not become the source of identity for the internal channel

#### Scenario: Diagnostic code owns canonical details

- **WHEN** a caller creates a diagnostic record for a specific `DiagnosticCode`
- **THEN** the record details conform to the canonical details object structure for that code
- **THEN** missing required fields, wrong field types, or disallowed extra fields are rejected by the diagnostics-owned constructor or checker
- **THEN** surface projection maps from that canonical details object

#### Scenario: Diagnostic code carries projection rules

- **WHEN** implementation defines whether a diagnostic can project to an error surface, warning surface, stderr line, or exit behavior
- **THEN** the rule is derived from the `DiagnosticCode` rule set
- **THEN** protocol schema, readable schema, examples, and fixtures consume the projection but do not own the rule source

### Requirement: Diagnostic stack provides scoped checkpoints and LIFO retrieval

The diagnostic stack MUST provide request-scoped ids, checkpoints, and default LIFO retrieval so callers can inspect or drain diagnostics created after a known point without exposing stack implementation details as public output contract.

#### Scenario: Pushed record can be retrieved by id

- **WHEN** a caller pushes a diagnostic record onto the stack
- **THEN** the stack returns an opaque `DiagnosticId` scoped to that stack lifetime
- **THEN** a caller holding that id can retrieve the same record without popping or consuming it
- **THEN** callers cannot provide their own `DiagnosticId` value when pushing

#### Scenario: Caller drains records after a checkpoint

- **WHEN** a caller creates a mark before trying a candidate path
- **AND** that candidate path pushes one or more diagnostic records
- **THEN** the caller can drain records pushed after the mark as a batch
- **THEN** records that existed before the mark remain in the stack

#### Scenario: Caller drains records after a record id

- **WHEN** a caller holds the `DiagnosticId` for an earlier stack record
- **THEN** the caller can choose whether draining starts strictly after that record id or includes the record referenced by that id
- **THEN** the record referenced by the id remains available unless the caller explicitly removes it

#### Scenario: Stack retrieval is LIFO by default

- **WHEN** a caller pops, drains, snapshots, renders, or flushes stack records without requesting another order
- **THEN** the stack returns records in last-in-first-out order
- **THEN** a caller that needs insertion order or grouped output explicitly reverses or regroups the returned records outside the stack

#### Scenario: Stack lifetime does not cross process boundary

- **WHEN** a top-level `docnav` command, adapter direct command, or adapter `invoke` request creates a diagnostic stack
- **THEN** stack ids and marks are valid only for that in-process stack lifetime
- **THEN** serialized protocol/readable output does not expose stack ids, marks, or indexes as durable refs

### Requirement: Boundary surfaces project diagnostic records

Modules that discover problems MUST push diagnostic records into the channel. Boundary surfaces MUST read those records and project them according to their owner contract.

#### Scenario: Runtime module writes but does not format final output

- **WHEN** core runtime, adapter routing, standard parameter resolution, adapter direct CLI, or adapter `invoke` handling discovers a problem
- **THEN** that module records what happened, its impact, canonical details, and source in the diagnostic stack
- **THEN** that module does not own final user-visible formatting unless it is also the boundary surface owner

#### Scenario: Boundary surface projects records to its own contract

- **WHEN** CLI, protocol surface, readable output, adapter direct CLI, or adapter `invoke` handler reaches an output boundary
- **THEN** the boundary reads the diagnostic stack records relevant to that output
- **THEN** the boundary projects records according to `docs/cli.md`, `docs/protocol.md`, `docs/output.md`, or `docs/adapter-contract.md`

#### Scenario: Surface docs do not redefine internal channel semantics

- **WHEN** protocol, readable, CLI, adapter, schema, or example docs describe diagnostic output
- **THEN** they define display, filtering, mapping, stdout/stderr placement, envelope shape, or exit behavior for their surface
- **THEN** they do not redefine `DiagnosticCode`, canonical details, `DiagnosticId`, mark lifetime, or default LIFO semantics

### Requirement: Legacy diagnostic sources are fully migrated

Existing error and warning fact sources MUST fully migrate to diagnostic channel records and diagnostics-owned projections. The completed implementation MUST NOT retain a legacy compatibility layer as a parallel diagnostic fact source.

#### Scenario: Stable error projection uses diagnostic code

- **WHEN** a fatal diagnostic is rendered or serialized as a stable surface error
- **THEN** the target surface error code is derived from `DiagnosticCode`
- **THEN** no legacy stable error object remains as an owning fact model after migration completes

#### Scenario: Warning projection uses diagnostic code and details

- **WHEN** a recoverable diagnostic is rendered as a warning
- **THEN** the warning id is derived from `DiagnosticCode`
- **THEN** the warning effect and family-specific details are derived from the diagnostic record
- **THEN** no independent warning id registry remains as the warning identity source

#### Scenario: Direct stderr path records before flushing

- **WHEN** a Rust entry point rejects command shape, fails manifest/probe/invoke decode, fails schema validation, or hits output write failure before normal document output
- **THEN** the entry point records diagnostic context in the channel before writing stderr or protocol/readable failure output
- **THEN** the observable output follows the owning surface projection policy

### Requirement: Protocol error rules JSON is removed

`docs/protocol/error-rules.json` MUST be deleted as a rule source. Protocol error code and details validation MUST consume `DiagnosticCode` protocol projections from `docnav-diagnostics`, while protocol docs, schema, examples, and tests remain validation and presentation materials.

#### Scenario: Protocol crate uses diagnostics code directly

- **WHEN** `docnav-protocol` needs to render, validate, or categorize a protocol-visible diagnostic
- **THEN** it depends on `docnav-diagnostics` and uses `DiagnosticCode` or an explicit diagnostics-owned protocol projection
- **THEN** it does not maintain a separate `StableErrorCode` or protocol-local required-details rule source

#### Scenario: Protocol schema consumes projection

- **WHEN** `protocol-response.schema.json` validates an error response
- **THEN** its code enum and details constraints match the diagnostics-owned protocol projection
- **THEN** the schema does not define new diagnostic code or details rules that are absent from `DiagnosticCode`

#### Scenario: Error rules generator no longer reads protocol JSON

- **WHEN** repository validation checks generated error rules
- **THEN** no script reads `docs/protocol/error-rules.json`
- **THEN** generated Rust or TypeScript constants, if any remain, are derived from diagnostics-owned rules or are replaced by direct `docnav-diagnostics` usage

### Requirement: Diagnostic channel changes update validation materials

Changes to diagnostic channel semantics or surface projection MUST update the relevant owner docs, JSON Schema, examples, fixtures, and tests in the same implementation work.

#### Scenario: Protocol JSON projection is validated

- **WHEN** a document operation is rendered as `protocol-json`
- **THEN** stdout follows the protocol response schema owned by the protocol docs
- **THEN** any protocol-visible diagnostic fields or errors are derived from diagnostic channel records

#### Scenario: Readable output projection is validated

- **WHEN** a document operation is rendered as `readable-view` or `readable-json`
- **THEN** recoverable diagnostics that have a readable warning projection are rendered from diagnostic channel records
- **THEN** readable output remains separate from the protocol response envelope

#### Scenario: Adapter machine command projection is validated

- **WHEN** an adapter direct `manifest`, `probe`, or `invoke` command writes machine output
- **THEN** stdout follows the owning manifest, probe, or protocol response schema
- **THEN** diagnostic output is derived from diagnostic channel records according to that surface policy

#### Scenario: Diagnostic-only effect requires its own behavior owner

- **WHEN** implementation updates warning effects while applying this change
- **THEN** it adds only effects required by a concrete behavior owner and matching validation material
