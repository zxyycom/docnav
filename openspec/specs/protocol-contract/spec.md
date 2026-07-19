# protocol-contract Specification

## Purpose
Define Docnav's raw machine protocol: request and response envelopes, operation identity, operation/result pairing, compact result shapes, page/continuation facts, protocol failure envelopes, protocol error fields, and structured machine facts. `output-contract` owns readable rendering; `adapter-contract` owns linked adapter interfaces.

## Requirements
### Requirement: Protocol envelopes are self-describing
Protocol request and response envelopes MUST carry stable operation identity and enough structured context for machine validation, replay, and failure attribution.

#### Scenario: Successful operation response
- **WHEN** a document operation succeeds in protocol-json mode
- **THEN** stdout contains one protocol response envelope
- **THEN** the envelope identifies the operation
- **THEN** the result shape matches that operation

#### Scenario: Failed operation response
- **WHEN** a document operation fails in protocol-json mode
- **THEN** stdout contains one protocol failure envelope
- **THEN** the failure envelope contains the primary diagnostic projection

### Requirement: Operations bind to success result types
Each protocol operation MUST bind to its valid success result shape. A response is valid only when its operation identity and result type match.

#### Scenario: Outline result pairing
- **WHEN** the response operation is `outline`
- **THEN** the success result is an outline result
- **THEN** read, find, or info result fields are not substituted for outline

### Requirement: Protocol facts are structured before display
Protocol result fields MUST expose machine-readable facts instead of relying on display strings for semantics. Readable output MUST derive any display text from those facts or from adapter-owned presentation hooks.

#### Scenario: Cost facts
- **WHEN** an operation reports cost
- **THEN** protocol output exposes structured cost measurements
- **THEN** readable output may render a compact cost summary from those measurements

#### Scenario: Navigation item facts
- **WHEN** outline, find, or info returns items
- **THEN** protocol output includes stable item facts owned by the operation
- **THEN** display text remains an output-layer convenience

### Requirement: Page and continuation are bounded protocol facts
Paginated protocol results MUST expose bounded content and a stable continuation value or null. Callers continue through protocol fields rather than readable text parsing.

#### Scenario: More content remains
- **WHEN** a result is truncated by the active budget
- **THEN** the protocol result includes the next page value
- **THEN** the caller can request that page explicitly

#### Scenario: No content remains
- **WHEN** the returned content is complete for the request
- **THEN** the protocol result page continuation is null

### Requirement: Protocol failures use diagnostic records
Protocol failures MUST project diagnostic identity, message, owner, source, and canonical details through `diagnostics-contract`. Legacy error sources must be normalized before they reach the public protocol surface.

#### Scenario: Ref not found
- **WHEN** an adapter reports that a valid ref cannot be matched
- **THEN** protocol failure uses the stable diagnostic code for that condition
- **THEN** canonical details describe the ref boundary without changing the ref contract

### Requirement: Outline supports structured and unstructured result kinds
Outline protocol results MUST distinguish normal structured outline entries from declared unstructured full-read content. Unstructured full-read results MUST remain typed as content rather than paginated outline entries.

#### Scenario: Structured outline
- **WHEN** normal outline policy applies
- **THEN** the result kind is structured
- **THEN** entries contain refs that can be read

#### Scenario: Unstructured full-read outline
- **WHEN** opt-in pre-dispatch policy returns full content
- **THEN** the result kind is unstructured
- **THEN** the content block is not treated as an outline entry page

### Requirement: outline and find expose a success-only auto-read object

When unique-ref auto-read successfully reads the one distinct ref in the current returned result, the outline or find result MUST include a closed `auto_read` object with `reason: "unique_ref"` and a complete existing `ReadResult`. In every other outcome, `auto_read` MUST be absent.

#### Scenario: successful auto-read contains its trigger and read result
- **WHEN** nested read returns a validated success
- **THEN** `auto_read.reason` is `unique_ref`
- **AND** `auto_read.read` is the complete existing `ReadResult`
- **AND** the object contains no `mode`, `status`, sibling `ref` or `error`

#### Scenario: no successful auto-read adds no field
- **WHEN** auto-read is disabled, current returned refs are not unique, or nested read does not succeed
- **THEN** the base result contains no `auto_read` field
- **AND** no skipped reason or nested diagnostic is added elsewhere in the public result

#### Scenario: base fields remain present
- **WHEN** an outline or find result contains `auto_read`
- **THEN** the existing `kind`/`entries`/`page` or `matches`/`page` fields retain their documented shape and meaning
- **AND** no base item is removed, reordered or rewritten by composition

### Requirement: composed success retains the base operation envelope

A composed response MUST use one public `ProtocolResponse::Success` whose operation remains the requested base operation. Nested read MUST NOT create a second public envelope.

#### Scenario: outline composition retains outline operation
- **WHEN** unique-ref outline successfully adds a read result
- **THEN** the outer response operation is `outline`
- **AND** the result validates as an outline result with `auto_read`

#### Scenario: find composition retains find operation
- **WHEN** unique-ref find successfully adds a read result
- **THEN** the outer response operation is `find`
- **AND** the result validates as a find result with `auto_read`

#### Scenario: base failure remains a failure envelope
- **WHEN** the requested outline or find operation fails before a base success result exists
- **THEN** the response remains the existing `ProtocolResponse::Failure`
- **AND** no `auto_read` result is constructed

#### Scenario: nested read non-success retains base success
- **WHEN** the base operation succeeds
- **AND** nested read does not produce a validated success
- **THEN** the response remains the existing base `ProtocolResponse::Success`
- **AND** no `auto_read` field is present

### Requirement: existing page fields retain their operation meaning

Unique-ref auto-read MUST reuse the existing base result and `ReadResult` page fields. It MUST NOT add a generic composition continuation field.

#### Scenario: base continuation remains on the base result
- **WHEN** a base result with non-null `page` successfully triggers auto-read
- **THEN** the base `page` retains the documented next page number for outline or find
- **AND** it does not prevent current-result unique-ref orchestration

#### Scenario: read continuation remains nested
- **WHEN** nested read succeeds with a non-null page
- **THEN** `auto_read.read.page` retains the documented next read page number
- **AND** the caller can continue normal read using the nested read ref and page
