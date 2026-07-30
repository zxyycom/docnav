本 delta spec 是 `replace-probe-traversal-with-inferred-routing` 的临时协议工件：它用既有 failure envelope 投影 exact routing diagnostics，并从 shared protocol contract 完整删除 probe result surface。

## ADDED Requirements

### Requirement: Probe result is not a protocol surface

The shared protocol contract MUST NOT define, export, decode, validate, serialize, or schema-check a probe result, probe reason, probe version, or probe-stage candidate fact. Adapter list MUST continue to consume manifest facts and document operations MUST expose only their existing request/success envelopes or the canonical failure envelope.

#### Scenario: Automatic routing selects an adapter

- **WHEN** navigation performs private format inference and exact registry lookup
- **THEN** no probe request or result enters the protocol boundary
- **THEN** successful operation output retains the existing operation result envelope

#### Scenario: Protocol library public surface is inspected

- **WHEN** consumers inspect shared protocol exports, decoders, constants, schemas, and validation entry points
- **THEN** no `ProbeResult`, probe decoder, probe validator, probe version, or probe schema surface remains

## MODIFIED Requirements

### Requirement: Protocol failures use diagnostic records

Protocol failures MUST project diagnostic identity, message, owner, source, and canonical details through `diagnostics-contract`. Legacy error sources must be normalized before they reach the public protocol surface. Automatic unknown MUST project `FORMAT_UNKNOWN` with exact details `{"path":"<normalized-path>","reason":"FORMAT_NOT_RECOGNIZED","candidates":[]}`. Inferred-format unsupported MUST project `FORMAT_UNKNOWN` with exact details `{"path":"<normalized-path>","reason":"NO_SUPPORTED_ADAPTER","format":"<normalized-format-id>","candidates":[]}`. Multiple normalized inference identities MUST project `FORMAT_AMBIGUOUS`; its sorted/deduplicated `candidates[]` contains only exact-mapped project `adapter_id`, stage `resolve`, and reason `FORMAT_MATCH`, and MAY be empty or single-item without permitting routing to guess a winner. Registry identity conflict MUST project `INTERNAL_ERROR` with exact details `{"error_id":"registry-format-identity-conflict"}`. An inference implementation failure outside an existing document diagnostic MUST project `INTERNAL_ERROR` with exact details `{"error_id":"format-routing-failed"}`; document path/read/encoding failures retain their existing `DOCUMENT_*` exact details. These routing details MUST omit `candidate_failures`, probe stages/reasons, and third-party inference values/messages. A selected adapter parse/semantic/operation failure MUST preserve its adapter-owned diagnostic and MUST NOT be replaced by a format-routing failure.

#### Scenario: Ref not found

- **WHEN** an adapter reports that a valid ref cannot be matched
- **THEN** protocol failure uses the stable diagnostic code for that condition
- **THEN** canonical details describe the ref boundary without changing the ref contract

#### Scenario: Unknown and unsupported project differently

- **WHEN** automatic routing cannot recognize a project-owned format identity
- **THEN** protocol failure uses `FORMAT_UNKNOWN` with `FORMAT_NOT_RECOGNIZED` and no `format`
- **WHEN** routing recognizes an unsupported normalized identity
- **THEN** protocol failure uses `FORMAT_UNKNOWN` with `NO_SUPPORTED_ADAPTER` and that normalized `format`

#### Scenario: Registry identity conflict is fatal

- **WHEN** a duplicate registry format identity reaches runtime
- **THEN** protocol failure uses `INTERNAL_ERROR`
- **THEN** details equal `{"error_id":"registry-format-identity-conflict"}`
- **THEN** no document candidate or inference-library evidence is serialized

#### Scenario: Routing implementation failure is local and sanitized

- **WHEN** inference fails outside an existing document diagnostic
- **THEN** protocol failure uses `INTERNAL_ERROR`
- **THEN** details equal `{"error_id":"format-routing-failed"}`
- **THEN** third-party error text is not serialized

#### Scenario: Selected adapter failure is preserved

- **WHEN** one adapter has been selected and its actual parse or operation fails
- **THEN** the protocol failure projects that adapter-owned diagnostic
- **THEN** protocol projection does not synthesize `FORMAT_UNKNOWN` or `FORMAT_AMBIGUOUS`

#### Scenario: Explicit adapter is unavailable

- **WHEN** a declared adapter id is absent from registry
- **THEN** protocol failure uses `ADAPTER_UNAVAILABLE`
- **THEN** details contain the declared `adapter_id`, reason `ADAPTER_NOT_FOUND`, resolved `selection_source`, and stage `resolve`
- **THEN** inference is not invoked
