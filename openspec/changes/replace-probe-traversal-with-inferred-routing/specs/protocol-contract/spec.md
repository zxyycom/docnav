本 delta spec 定义 `replace-probe-traversal-with-inferred-routing` 对 `protocol-contract` 尚未应用的 Target：用既有 failure envelope 投影 exact routing diagnostics，并从 shared protocol contract 完整删除 probe result surface；它不表示 Current 主规范或实现已经迁移。

## ADDED Requirements

### Requirement: Probe result is not a protocol surface

The shared protocol contract MUST NOT define, export, decode, validate, serialize, or schema-check a probe result, probe reason, probe version, or probe-stage candidate fact. Adapter list MUST continue to consume manifest facts, including `formats[].extensions[]` and `formats[].filenames[]`; document operations MUST expose only their existing request/success envelopes or the canonical failure envelope.

#### Scenario: Automatic routing selects an adapter

- **WHEN** navigation performs private manifest pathname lookup and exact registry lookup
- **THEN** no probe request or result enters the protocol boundary
- **THEN** successful operation output retains the existing operation result envelope

#### Scenario: Protocol library public surface is inspected

- **WHEN** consumers inspect shared protocol exports, decoders, constants, schemas, and validation entry points
- **THEN** no `ProbeResult`, probe decoder, probe validator, probe version, or probe schema surface remains

## MODIFIED Requirements

### Requirement: Protocol failures use diagnostic records

Protocol failures MUST project diagnostic identity, message, owner, source, and canonical details through `diagnostics-contract`. Legacy error sources must be normalized before they reach the public protocol surface. An automatic pathname miss MUST project `FORMAT_UNKNOWN` with exact details `{"path":"<routing-pathname>","reason":"FORMAT_NOT_RECOGNIZED","candidates":[]}`; this path is lexical because target-document filesystem normalization has not run. A registry format identity conflict MUST project `INTERNAL_ERROR` with exact details `{"error_id":"registry-format-identity-conflict"}`. A registry same-kind normalized-suffix or exact-filename conflict MUST project `INTERNAL_ERROR` with exact details `{"error_id":"registry-path-hint-conflict"}`. Selected JSON syntax, trailing-input, duplicate-member, and depth failures MUST project `DOCUMENT_CONTENT_INVALID` with exact `{"path":"<normalized-path>","reason":"<stable-json-reason>"}` details from diagnostics-contract. Target automatic routing MUST NOT project `NO_SUPPORTED_ADAPTER`, `FORMAT_AMBIGUOUS`, `FORMAT_MATCH`, `candidate_failures`, probe stages/reasons, matched hints/formats, derived pathname/index facts, or external routing evidence. A selected adapter document/parse/semantic/operation failure MUST preserve its owner-compatible diagnostic and MUST NOT be replaced by a routing failure.

#### Scenario: Ref not found

- **WHEN** an adapter reports that a valid ref cannot be matched
- **THEN** protocol failure uses the stable diagnostic code for that condition
- **THEN** canonical details describe the ref boundary without changing the ref contract

#### Scenario: Unknown pathname has one exact projection

- **WHEN** automatic routing matches no manifest filename or complete-basename suffix hint
- **THEN** protocol failure uses `FORMAT_UNKNOWN` with `FORMAT_NOT_RECOGNIZED` and no `format`
- **THEN** details carry the lexical routing pathname rather than a filesystem-normalized document path

#### Scenario: Registry identity conflict is fatal

- **WHEN** a duplicate registry format identity reaches runtime
- **THEN** protocol failure uses `INTERNAL_ERROR`
- **THEN** details equal `{"error_id":"registry-format-identity-conflict"}`
- **THEN** no document candidate or private routing evidence is serialized

#### Scenario: Registry pathname-hint conflict is fatal

- **WHEN** a duplicate same-kind normalized-suffix or exact-filename hint reaches runtime
- **THEN** protocol failure uses `INTERNAL_ERROR`
- **THEN** details equal `{"error_id":"registry-path-hint-conflict"}`
- **THEN** no document candidate or private routing evidence is serialized

#### Scenario: Selected adapter failure is preserved

- **WHEN** one adapter has been selected and its actual parse or operation fails
- **THEN** the protocol failure projects that adapter-owned diagnostic
- **THEN** protocol projection does not synthesize a routing diagnostic

#### Scenario: Selected invalid JSON has stable protocol details

- **WHEN** selected `docnav-json` rejects syntax, trailing input, a duplicate decoded member, or maximum depth
- **THEN** protocol failure uses `DOCUMENT_CONTENT_INVALID`
- **THEN** details contain only the normalized path and corresponding stable JSON reason
- **THEN** protocol output omits `json-document-changed-after-probe` and parser implementation evidence

#### Scenario: Explicit adapter is unavailable

- **WHEN** a declared adapter id is absent from registry
- **THEN** protocol failure uses `ADAPTER_UNAVAILABLE`
- **THEN** details contain the declared `adapter_id`, reason `ADAPTER_NOT_FOUND`, resolved `selection_source`, and stage `resolve`
- **THEN** automatic pathname routing is not invoked
