本 delta spec 是 `replace-probe-traversal-with-inferred-routing` 的临时契约工件：它规定 automatic routing 只执行一次内部 inference 和精确 lookup，explicit routing 跳过 inference，且 selected adapter failure 永不触发 fallback。

## MODIFIED Requirements

### Requirement: Navigation selects adapter before adapter parameter extraction

Navigation MUST select the adapter using routing inputs and registry facts before filtering adapter-scoped entries for selected-operation candidate extraction and resolution. When caller intent declares an adapter id, navigation MUST skip format inference and perform one exact adapter-id lookup. When no adapter id is declared, navigation MUST invoke the approved internal format inference implementation exactly once, normalize its result to one or more project-owned format identities, and match those identities by exact equality against the core-validated unique registry format index. Automatic selection MUST NOT execute adapter probes, traverse definitions in registry order, or use registry order as an ambiguity tie-breaker. No normalized identity MUST return `FORMAT_UNKNOWN` with reason `FORMAT_NOT_RECOGNIZED`; one normalized identity with no registry adapter MUST return `FORMAT_UNKNOWN` with reason `NO_SUPPORTED_ADAPTER` and the normalized `format`; multiple normalized identities MUST return `FORMAT_AMBIGUOUS` with only exact-mapped, sorted/deduplicated project adapter candidates, even if that list is empty or single-item; and a duplicate registry format identity that escapes release validation MUST return global `INTERNAL_ERROR` with error id `registry-format-identity-conflict`. Full catalog config validation is a separate projection and MUST NOT be treated as adapter parameter extraction. The selected registry entry MUST expose an adapter definition for capability and linked strategy facts. Document-operation parameter declarations MUST come from the core catalog rather than that definition. Raw inference-library enum values, confidence, messages, errors, and detection evidence MUST remain outside diagnostics, protocol, readable output, logs, refs, and continuation values.

#### Scenario: Automatic selection identifies one supported format

- **WHEN** no adapter id is declared
- **AND** one inference invocation normalizes the document to one format identity
- **AND** exactly one registry definition declares that format identity
- **THEN** navigation selects that definition
- **THEN** navigation does not execute any adapter probe
- **THEN** registry order does not affect the outcome

#### Scenario: Automatic selection cannot identify a routable adapter

- **WHEN** inference returns no project-normalized identity
- **THEN** navigation returns `FORMAT_UNKNOWN` with `FORMAT_NOT_RECOGNIZED`, empty candidates, and no `format`
- **WHEN** the normalized identity has no exact registry match
- **THEN** navigation returns `FORMAT_UNKNOWN` with `NO_SUPPORTED_ADAPTER`, the normalized `format`, and empty candidates
- **THEN** no adapter operation is dispatched
- **THEN** no other adapter is tried

#### Scenario: Multiple inferred identities are ambiguous

- **WHEN** inference returns multiple project-normalized identities
- **THEN** navigation returns `FORMAT_AMBIGUOUS`
- **THEN** candidates are exact-mapped, sorted/deduplicated project adapter ids with stage `resolve` and reason `FORMAT_MATCH`
- **THEN** an empty or single candidate list does not permit routing to guess a winner
- **THEN** no candidate-failure or third-party inference evidence is exposed

#### Scenario: Duplicate registry identity is a global invariant failure

- **WHEN** runtime observes a duplicate normalized registry format identity that construction/release validation should have rejected
- **THEN** navigation returns `INTERNAL_ERROR`
- **THEN** details equal `{"error_id":"registry-format-identity-conflict"}`
- **THEN** the failure is not classified as a document-specific ambiguity

#### Scenario: Explicit adapter intent bypasses inference

- **WHEN** routing input declares an adapter id
- **THEN** navigation performs exact adapter-id lookup without invoking format inference or adapter probe
- **THEN** a missing id returns `ADAPTER_UNAVAILABLE` with reason `ADAPTER_NOT_FOUND`, the declared adapter id, resolved selection source, and stage `resolve`
- **THEN** a matching definition advances to normal selected-operation resolution

#### Scenario: Multiple adapters exist

- **WHEN** registry contains multiple adapter definitions
- **THEN** navigation selects only the definition whose declared format identity exactly matches the normalized inferred identity, or whose adapter id exactly matches explicit intent
- **THEN** only core catalog entries applicable to the selected adapter and operation participate in resolution
- **THEN** entries scoped to unselected adapters remain outside the operation field set

#### Scenario: Selected definition provides capability facts

- **WHEN** navigation has selected an adapter
- **THEN** it reads optional capability declarations and the linked strategy from the selected adapter definition
- **THEN** it reads parameter facts from the core catalog

### Requirement: Navigation dispatches linked adapter handlers

After successful input resolution, standard type materialization, configured core pre-dispatch checks, and adapter selection, navigation MUST dispatch the closed standard operation input to the selected linked adapter strategy and return structured result or diagnostic facts to the owning output/protocol layer. Format inference or explicit registry lookup MUST NOT count as document parse or semantic validation. The selected strategy MUST execute its normal document acquisition, decode, parse, and semantic checks required by the requested operation. The strategy reference and capability context MUST come from the selected adapter definition; applicable operation-specific typed fields or accessors MUST be built from core-catalog resolution. The selected strategy MUST NOT require a second caller-data argument or generic parameter handoff. It MAY return semantic validation diagnostics for conditions not guaranteed by core or MAY repeat a core check defensively. Once a strategy has been selected, its parse failure, semantic diagnostic, operation failure, or invalid result MUST NOT cause navigation to infer again, select another registry definition, or dispatch another adapter.

#### Scenario: Dispatch succeeds

- **WHEN** navigation has constructed standard typed operation input
- **THEN** it calls the selected adapter strategy
- **THEN** the strategy performs the document processing required by that operation
- **THEN** navigation preserves the returned structured result facts for projection

#### Scenario: Explicitly selected adapter parses the real document

- **WHEN** explicit adapter intent resolves to a registered definition
- **THEN** inference remains skipped
- **THEN** the selected strategy still acquires and validates the actual document for the requested operation
- **THEN** lookup success alone does not produce an operation success

#### Scenario: Dispatch returns adapter semantic diagnostic

- **WHEN** standard input is well-typed but the actual document or value violates a selected strategy precondition
- **THEN** the strategy returns its normal diagnostic before running the unsafe or invalid algorithm path
- **THEN** navigation preserves that diagnostic for normal protocol/readable projection
- **THEN** navigation does not try another adapter

#### Scenario: Selected adapter operation fails

- **WHEN** the selected strategy returns an operation failure or an invalid result
- **THEN** navigation returns the owner-compatible failure
- **THEN** navigation does not invoke inference again
- **THEN** navigation does not inspect or dispatch any later registry member

#### Scenario: Dispatch combines separate core facts

- **WHEN** navigation dispatches a selected operation
- **THEN** the strategy implementation comes from the selected adapter definition
- **THEN** adapter-scoped typed values come from entries applicable to that adapter and operation in core catalog
- **THEN** routing/strategy facts and parameter facts remain owned by their separate sources
