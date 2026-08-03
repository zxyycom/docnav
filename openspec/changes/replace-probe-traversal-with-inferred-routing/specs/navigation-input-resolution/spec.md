本 delta spec 定义 `replace-probe-traversal-with-inferred-routing` 对 `navigation-input-resolution` 尚未应用的 Target：automatic routing 只执行一次 manifest-derived pathname lookup，explicit routing 跳过 automatic routing，selected adapter failure 永不触发 fallback；它不表示 Current 主规范或实现已经迁移。

## MODIFIED Requirements

### Requirement: Navigation selects adapter before adapter parameter extraction

Navigation MUST select the adapter using routing inputs and registry facts before filtering adapter-scoped entries for selected-operation candidate extraction and resolution. When caller intent declares an adapter id, navigation MUST skip automatic pathname routing and perform one exact adapter-id lookup. When no adapter id is declared, navigation MUST perform one lexical lookup over facts derived from validated manifests before any target-document metadata lookup, open, canonicalize, read, or parse. It MUST first match the routing pathname's complete basename by case-sensitive equality against `formats[].filenames[]`. Only when no exact filename matches, it MUST compare the complete basename with `formats[].extensions[]` as leading-dot suffixes after ASCII case normalization. Extension hints MAY contain multiple dots. If multiple different suffixes match, navigation MUST choose the longest declared suffix. A matched hint MUST resolve privately to its manifest-owned normalized format identity and then to the unique registry definition for that identity. Exact-filename precedence MUST allow a specific basename route to override a generic suffix route. Only after selection MAY filesystem-backed path/access normalization occur for construction of the selected operation input.

Automatic selection MUST NOT execute adapter probes, inspect content, traverse definitions in registry order, or use registry order to choose a winner. No pathname-hint match MUST return `FORMAT_UNKNOWN` with the lexical routing pathname, reason `FORMAT_NOT_RECOGNIZED`, empty `candidates`, and no `format`. A duplicate registry format identity that escapes construction/release validation MUST return global `INTERNAL_ERROR` with error id `registry-format-identity-conflict`; a duplicate same-kind normalized-suffix or exact-filename hint that escapes validation MUST return global `INTERNAL_ERROR` with error id `registry-path-hint-conflict`. Because validated manifest hints are unique and each hint is owned by a registered format, Target automatic routing MUST NOT produce `NO_SUPPORTED_ADAPTER`, `FORMAT_AMBIGUOUS`, `FORMAT_MATCH`, or routing candidate-failure facts.

Full catalog config validation is a separate projection and MUST NOT be treated as adapter parameter extraction. The selected registry entry MUST expose an adapter definition for capability and linked strategy facts. Document-operation parameter declarations MUST come from the core catalog rather than that definition. Derived filename/suffix keys, indexes, matched hints, and matched format identities MUST remain outside diagnostics, protocol, readable output, logs, refs, continuations, typed fields, and adapter operation input. A pathname match is a cheap selection convenience that MAY be wrong about document contents; the selected strategy MUST receive only the normal closed operation input and MUST establish real format validity itself.

#### Scenario: Exact filename takes precedence

- **WHEN** no adapter id is declared
- **AND** the case-sensitive basename matches one manifest `filenames[]` hint
- **THEN** navigation selects the registry definition for that hint's format identity
- **THEN** navigation does not consult the basename's generic suffix mapping
- **THEN** navigation does not inspect metadata, open, canonicalize, read, or parse the document before selection

#### Scenario: Suffix routes when no filename matches

- **WHEN** no adapter id is declared
- **AND** no exact filename hint matches
- **AND** the complete basename ends with one manifest `extensions[]` suffix after ASCII case normalization
- **THEN** navigation selects that definition
- **THEN** navigation does not execute any adapter probe
- **THEN** registry order does not affect the outcome

#### Scenario: Longest compound suffix wins

- **WHEN** no exact filename matches basename `model.schema.JSON`
- **AND** manifests declare `.json` and `.schema.json` for different format identities
- **THEN** navigation selects the identity owned by `.schema.json`
- **THEN** suffix matching is ASCII case-insensitive
- **THEN** the different-length overlap is not a registry conflict

#### Scenario: Suffix is anchored to the basename end

- **WHEN** no exact filename matches basename `settings.json.backup`
- **AND** `.json` is the only declared suffix
- **THEN** `.json` does not match
- **THEN** navigation returns the pathname no-match outcome without target-document filesystem I/O

#### Scenario: Automatic selection cannot identify a routable adapter

- **WHEN** neither the exact basename nor any complete-basename suffix matches a manifest hint
- **THEN** navigation returns `FORMAT_UNKNOWN` with `FORMAT_NOT_RECOGNIZED`, empty candidates, and no `format`
- **THEN** no adapter operation is dispatched
- **THEN** no other adapter is tried
- **THEN** no target-document metadata lookup, open, canonicalize, read, or parse occurs

#### Scenario: Duplicate registry identity is a global invariant failure

- **WHEN** runtime observes a duplicate normalized registry format identity that construction/release validation should have rejected
- **THEN** navigation returns `INTERNAL_ERROR`
- **THEN** details equal `{"error_id":"registry-format-identity-conflict"}`
- **THEN** the failure is not classified as a document-specific ambiguity

#### Scenario: Duplicate pathname hint is a global invariant failure

- **WHEN** runtime observes a duplicate same-kind ASCII-normalized suffix or exact-filename hint that construction/release validation should have rejected
- **THEN** navigation returns `INTERNAL_ERROR`
- **THEN** details equal `{"error_id":"registry-path-hint-conflict"}`
- **THEN** the failure is not classified as a document-specific ambiguity

#### Scenario: Explicit adapter intent bypasses pathname routing

- **WHEN** routing input declares an adapter id
- **THEN** navigation performs exact adapter-id lookup without invoking pathname routing or adapter probe
- **THEN** a missing id returns `ADAPTER_UNAVAILABLE` with reason `ADAPTER_NOT_FOUND`, the declared adapter id, resolved selection source, and stage `resolve`
- **THEN** a matching definition advances to normal selected-operation resolution

#### Scenario: Multiple adapters exist

- **WHEN** registry contains multiple adapter definitions
- **THEN** navigation selects only the definition whose format owns the matched pathname hint, or whose adapter id exactly matches explicit intent
- **THEN** only core catalog entries applicable to the selected adapter and operation participate in resolution
- **THEN** entries scoped to unselected adapters remain outside the operation field set

#### Scenario: Selected definition provides capability facts

- **WHEN** navigation has selected an adapter
- **THEN** it reads optional capability declarations and the linked strategy from the selected adapter definition
- **THEN** it reads parameter facts from the core catalog

### Requirement: Navigation dispatches linked adapter handlers

After adapter selection, filesystem-backed path/access normalization, successful input resolution, standard type materialization, and configured core pre-dispatch checks, navigation MUST dispatch the closed standard operation input to the selected linked adapter strategy and return structured result or diagnostic facts to the owning output/protocol layer. Manifest pathname lookup or explicit registry lookup MUST NOT count as document parse or semantic validation. The selected strategy MUST execute its normal document acquisition, decode, parse, and semantic checks required by the requested operation. The strategy reference and capability context MUST come from the selected adapter definition; applicable operation-specific typed fields or accessors MUST be built from core-catalog resolution. The selected strategy MUST NOT require matched pathname/format state, a second caller-data argument, or generic parameter handoff. It MAY return semantic validation diagnostics for conditions not guaranteed by core or MAY repeat a core check defensively. Once a strategy has been selected, its path/access failure, parse failure, semantic diagnostic, operation failure, or invalid result MUST NOT cause navigation to route again, select another registry definition, or dispatch another adapter.

#### Scenario: Dispatch succeeds

- **WHEN** navigation has constructed standard typed operation input
- **THEN** it calls the selected adapter strategy
- **THEN** the strategy performs the document processing required by that operation
- **THEN** navigation preserves the returned structured result facts for projection

#### Scenario: Explicitly selected adapter parses the real document

- **WHEN** explicit adapter intent resolves to a registered definition
- **THEN** automatic pathname routing remains skipped
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
- **THEN** navigation does not invoke pathname routing again
- **THEN** navigation does not inspect or dispatch any later registry member

#### Scenario: Dispatch combines separate core facts

- **WHEN** navigation dispatches a selected operation
- **THEN** the strategy implementation comes from the selected adapter definition
- **THEN** adapter-scoped typed values come from entries applicable to that adapter and operation in core catalog
- **THEN** routing/strategy facts and parameter facts remain owned by their separate sources
