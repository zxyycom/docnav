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
