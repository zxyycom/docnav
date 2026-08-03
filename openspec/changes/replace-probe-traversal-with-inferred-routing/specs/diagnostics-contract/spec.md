本 delta spec 定义 `replace-probe-traversal-with-inferred-routing` 对 `diagnostics-contract` 尚未应用的 Target：固定 pathname no-match 与 registry-conflict 的 exact canonical details，并删除 probe/routing-only candidate vocabulary；它不表示 Current 主规范或实现已经迁移。

## MODIFIED Requirements

### Requirement: DiagnosticCode owns identity and canonical details

Every stable diagnostic code MUST have a single identity owner and canonical detail shape. Pathname routing MUST reuse existing `FORMAT_UNKNOWN`, `ADAPTER_UNAVAILABLE`, document diagnostics, and `INTERNAL_ERROR`; it MUST NOT add a code named after the routing mechanism. Removing JSON probe MUST add `DOCUMENT_CONTENT_INVALID` for selected content that violates the JSON-owned parse/safety contract instead of retaining a probe-stage or internal error. Other layers can add context only when the context preserves the code identity and detail semantics. The canonical affected mapping MUST be:

| Routing outcome | Code | Exact canonical `details` |
| --- | --- | --- |
| routing basename matches no manifest pathname hint | `FORMAT_UNKNOWN` | `{"path":"<routing-pathname>","reason":"FORMAT_NOT_RECOGNIZED","candidates":[]}` |
| registry duplicate format identity escapes release validation | `INTERNAL_ERROR` | `{"error_id":"registry-format-identity-conflict"}` |
| registry duplicate same-kind exact-filename/normalized-suffix hint escapes release validation | `INTERNAL_ERROR` | `{"error_id":"registry-path-hint-conflict"}` |
| explicit adapter id is absent from registry | `ADAPTER_UNAVAILABLE` | `{"adapter_id":"<declared-id>","reason":"ADAPTER_NOT_FOUND","selection_source":"<resolved-source>","stage":"resolve"}` |
| selected JSON syntax is invalid | `DOCUMENT_CONTENT_INVALID` | `{"path":"<normalized-path>","reason":"JSON_SYNTAX_INVALID"}` |
| selected JSON has trailing non-whitespace input | `DOCUMENT_CONTENT_INVALID` | `{"path":"<normalized-path>","reason":"JSON_TRAILING_INPUT"}` |
| selected JSON has duplicate decoded member | `DOCUMENT_CONTENT_INVALID` | `{"path":"<normalized-path>","reason":"JSON_DUPLICATE_MEMBER"}` |
| selected JSON exceeds maximum depth | `DOCUMENT_CONTENT_INVALID` | `{"path":"<normalized-path>","reason":"JSON_MAXIMUM_DEPTH_EXCEEDED"}` |
| selected adapter has another document/ref/semantic/operation failure | adapter-owned code | adapter owner's canonical details, unchanged by routing |

Manifest-derived automatic routing MUST NOT emit `NO_SUPPORTED_ADAPTER`, `FORMAT_AMBIGUOUS`, `FORMAT_MATCH`, `format`, or `candidate_failures`: every recognized hint and format identity comes from the same validated linked registry, and duplicate mappings are registry invariants rather than document ambiguity. The old `probe` stage, `PROBE_*` reasons, and `json-document-changed-after-probe` error id are removed. Matched filenames/suffixes, matched format identities, derived-index internals, parser types/messages, unstable offsets, member names, and dependency traces MUST NOT appear in diagnostic identity or canonical details. Selected adapter read/parse/semantic/operation failures MUST retain their owner-owned code and canonical details and MUST NOT be remapped as routing failures.

#### Scenario: Pathname hint is unknown

- **WHEN** automatic routing finds no exact filename or complete-basename suffix hint
- **THEN** the primary code is `FORMAT_UNKNOWN`
- **THEN** details use the lexical routing pathname and `FORMAT_NOT_RECOGNIZED` without a `format` field
- **THEN** candidates are empty and no matched hint is exposed
- **THEN** no target-document filesystem I/O was needed to form the diagnostic

#### Scenario: Registry identity conflict is internal

- **WHEN** runtime defensively observes a duplicate normalized format identity that release validation should have blocked
- **THEN** the primary code is `INTERNAL_ERROR`
- **THEN** details equal `{"error_id":"registry-format-identity-conflict"}`
- **THEN** the failure is not presented as a document-specific format ambiguity

#### Scenario: Registry pathname-hint conflict is internal

- **WHEN** runtime defensively observes a duplicate same-kind exact filename or ASCII-normalized suffix hint that release validation should have blocked
- **THEN** the primary code is `INTERNAL_ERROR`
- **THEN** details equal `{"error_id":"registry-path-hint-conflict"}`
- **THEN** the failure is not presented as a document-specific format ambiguity

#### Scenario: Code appears at multiple surfaces

- **WHEN** the same routing diagnostic is projected to protocol-json and readable-view
- **THEN** both projections use the same diagnostic code
- **THEN** canonical details keep the same structured meaning

#### Scenario: Selected JSON content is invalid

- **WHEN** pathname or explicit selection chooses `docnav-json`
- **AND** the actual document has invalid syntax, trailing input, a duplicate decoded member, or excessive depth
- **THEN** the primary code is `DOCUMENT_CONTENT_INVALID`
- **THEN** details contain the normalized path and the exact JSON reason from the canonical mapping
- **THEN** no probe-stage/internal error or parser-library detail is exposed

### Requirement: Public failures expose one primary diagnostic

Failure outputs MUST expose a single primary diagnostic record for the failed operation. Additional context must remain secondary, stable, and subordinate to the primary cause. Automatic routing MUST NOT synthesize sibling or nested candidate-failure lists after probe traversal is removed.

#### Scenario: Multiple candidate adapter failures

- **WHEN** automatic routing finds no pathname hint or the one selected adapter subsequently fails
- **THEN** the operation failure has one primary diagnostic from the owning boundary
- **THEN** no multiple-candidate failure set or candidate evidence is produced

#### Scenario: Automatic routing has no hint match

- **WHEN** routing produces pathname unknown or registry-invariant failure
- **THEN** the operation exposes exactly one primary diagnostic from the owning row of the canonical mapping
- **THEN** no registry-order candidate failure list is attached

#### Scenario: Selected adapter parsing fails

- **WHEN** routing selected one adapter and its actual document parse fails
- **THEN** the adapter-owned diagnostic remains the one primary failure
- **THEN** routing does not emit a second format diagnostic or try another adapter
