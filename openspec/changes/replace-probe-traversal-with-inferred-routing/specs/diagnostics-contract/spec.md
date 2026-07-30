本 delta spec 是 `replace-probe-traversal-with-inferred-routing` 的临时诊断工件：它复用既有 format/internal codes，固定 routing outcome 的 exact canonical details，并删除 probe-only candidate vocabulary。

## MODIFIED Requirements

### Requirement: DiagnosticCode owns identity and canonical details

Every stable diagnostic code MUST have a single identity owner and canonical detail shape. Format routing MUST reuse existing `FORMAT_UNKNOWN`, `FORMAT_AMBIGUOUS`, `ADAPTER_UNAVAILABLE`, document diagnostic, and `INTERNAL_ERROR` codes rather than add a code named after the inference mechanism. Other layers can add context only when the context preserves the code identity and detail semantics. The canonical routing mapping MUST be:

| Routing outcome | Code | Exact canonical `details` |
| --- | --- | --- |
| no normalized format identity | `FORMAT_UNKNOWN` | `{"path":"<normalized-path>","reason":"FORMAT_NOT_RECOGNIZED","candidates":[]}` |
| normalized format identity has no registry adapter | `FORMAT_UNKNOWN` | `{"path":"<normalized-path>","reason":"NO_SUPPORTED_ADAPTER","format":"<normalized-format-id>","candidates":[]}` |
| inference returns multiple normalized identities | `FORMAT_AMBIGUOUS` | `{"path":"<normalized-path>","candidates":[{"adapter_id":"<mapped-project-adapter-id>","stage":"resolve","reason":"FORMAT_MATCH"},...]}` containing only exact registry matches, sorted/deduplicated by `adapter_id`, and allowed to be empty or single-item |
| inference document path is missing | `DOCUMENT_NOT_FOUND` | `{"path":"<normalized-path>"}` |
| inference document path is invalid | `DOCUMENT_PATH_INVALID` | `{"path":"<normalized-path>","reason":"<Docnav-owned-reason>"}` |
| inference document encoding is unsupported | `DOCUMENT_ENCODING_UNSUPPORTED` | `{"path":"<normalized-path>","encoding":"<encoding>"}` |
| registry duplicate format identity escapes release validation | `INTERNAL_ERROR` | `{"error_id":"registry-format-identity-conflict"}` |
| inference implementation fails outside an existing document diagnostic | `INTERNAL_ERROR` | `{"error_id":"format-routing-failed"}` |
| explicit adapter id is absent from registry | `ADAPTER_UNAVAILABLE` | `{"adapter_id":"<declared-id>","reason":"ADAPTER_NOT_FOUND","selection_source":"<resolved-source>","stage":"resolve"}` |
| selected adapter parse/semantic/operation failure | adapter-owned existing code | adapter owner's existing canonical details, unchanged by routing |

`FORMAT_UNKNOWN.details.format` MUST exist only for `NO_SUPPORTED_ADAPTER`. `FORMAT_UNKNOWN` and `FORMAT_AMBIGUOUS` MUST NOT expose `candidate_failures`. Format candidates MUST use only stage `resolve` and reason `FORMAT_MATCH`; `probe` stage and `PROBE_*` reasons are removed. Third-party enum values, messages, debug/error text, confidence, and detection evidence MUST NOT appear in diagnostic identity, details, message, guidance, logs, or project result facts. Selected adapter parse/semantic/operation failures MUST retain their adapter-owned code and canonical details and MUST NOT be remapped as routing failures.

#### Scenario: Unknown and unsupported remain distinguishable

- **WHEN** automatic routing cannot normalize a format identity
- **THEN** the primary code is `FORMAT_UNKNOWN`
- **THEN** details use `FORMAT_NOT_RECOGNIZED` without a `format` field
- **WHEN** routing normalizes a format identity for which registry has no adapter
- **THEN** the primary code is `FORMAT_UNKNOWN`
- **THEN** details use `NO_SUPPORTED_ADAPTER` and include the normalized `format`

#### Scenario: Multiple inferred identities are ambiguous

- **WHEN** inference returns multiple project-normalized identities
- **THEN** the primary code is `FORMAT_AMBIGUOUS`
- **THEN** candidates contain only exact-mapped, sorted/deduplicated project adapter ids with stage `resolve` and reason `FORMAT_MATCH`
- **THEN** an empty or single candidate list does not permit routing to guess a winner
- **THEN** no raw inference evidence or probe failure is projected

#### Scenario: Registry identity conflict is internal

- **WHEN** runtime defensively observes a duplicate normalized format identity that release validation should have blocked
- **THEN** the primary code is `INTERNAL_ERROR`
- **THEN** details equal `{"error_id":"registry-format-identity-conflict"}`
- **THEN** the failure is not presented as a document-specific format ambiguity

#### Scenario: Code appears at multiple surfaces

- **WHEN** the same routing diagnostic is projected to protocol-json and readable-view
- **THEN** both projections use the same diagnostic code
- **THEN** canonical details keep the same structured meaning

### Requirement: Public failures expose one primary diagnostic

Failure outputs MUST expose a single primary diagnostic record for the failed operation. Additional context must remain secondary, stable, and subordinate to the primary cause. Automatic routing MUST NOT synthesize sibling or nested candidate-failure lists after probe traversal is removed.

#### Scenario: Automatic routing fails

- **WHEN** routing produces unknown, unsupported, ambiguous, document, or internal failure
- **THEN** the operation exposes exactly one primary diagnostic from the owning row of the canonical mapping
- **THEN** no registry-order candidate failure list is attached

#### Scenario: Selected adapter parsing fails

- **WHEN** routing selected one adapter and its actual document parse fails
- **THEN** the adapter-owned diagnostic remains the one primary failure
- **THEN** routing does not emit a second format diagnostic or try another adapter
