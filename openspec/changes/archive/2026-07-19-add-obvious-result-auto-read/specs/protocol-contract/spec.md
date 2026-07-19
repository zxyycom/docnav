本 spec delta 定义 default-on unique-ref composition 的 machine-readable success result。Base operation envelope 和 base fields 保持不变；`auto_read` 只在 nested read 成功时出现。

## ADDED Requirements

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
