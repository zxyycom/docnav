**这是一个临时 Target planning artifact：目标是扩展 navigation 的 project-root handoff、raw request union、final inference/manifest routing seam、逐文档 dispatch、fixed-quantum replay 和 project auto-read exclusion，而不决定 single-document find logical unit 或 inference implementation；当前 completed tasks 为 0/43，blocking gates 尚未关闭，因此它既不是 Current 行为，也不是实施授权。**

## MODIFIED Requirements

### Requirement: Core hands raw navigation inputs to navigation

Core CLI MUST hand document-operation command facts, scope-appropriate normalized path facts, config source descriptors/paths, static adapter registry, and core-owned closed parameter catalog to navigation without resolving operation arguments. Selected adapter definition contributes capability, manifest and linked strategy facts; parameter facts come from catalog. A pathless project find MUST carry resolved project root and no synthetic document path; an explicit-path operation MUST retain existing normalized document path handoff.

#### Scenario: Outline handoff
- **WHEN** core parses `docnav outline <path>`
- **THEN** it identifies operation and path facts
- **THEN** navigation receives raw navigation input package and core catalog
- **THEN** selected adapter definition supplies behavior facts only

#### Scenario: Project find handoff
- **WHEN** core parses `docnav find --query <text>` without path
- **THEN** navigation receives project scope, exact resolved project root, query and normalized CLI/config sources
- **AND** it does not receive a fabricated adapter document path

### Requirement: Navigation selects adapter before adapter parameter extraction

Navigation MUST validate global registry/catalog invariants and select an adapter before filtering adapter-scoped entries for per-document resolution. With no declared adapter id, each project candidate MUST invoke the approved inference implementation exactly once within one project request/replay, normalize a recognized result to the predecessor-owned format identity, exact-match registry definition format descriptors, and reuse that invocation-local selection across the document's adapter pages. A later project page request MUST recompute selection during fresh deterministic replay rather than use a cross-run cache. The predecessor's exact `Unknown` outcome and recognized-but-unregistered results MUST be ordinary filtering. Candidate inference document I/O failure, `FORMAT_AMBIGUOUS` caused by multiple inferred identities, and unclassified `format-routing-failed` MUST each be bounded local document failures and advance that document. Only a duplicate/conflicting static registry format-identity invariant MUST be routing-global fatal. With declared adapter id, navigation MUST perform one exact lookup before traversal, skip inference, and use that definition's finalized manifest descriptors for deterministic path eligibility. A missing id MUST return the existing `ADAPTER_UNAVAILABLE` / `ADAPTER_NOT_FOUND` diagnostic; duplicate adapter ids MUST fail static registry validation before caller exact lookup. Descriptor mismatch MUST be ordinary filtering. Raw inference-library facts MUST remain outside protocol, diagnostics, refs, logs and continuation state.

#### Scenario: Automatic project candidate selects one adapter
- **WHEN** no adapter id is declared
- **AND** one inference invocation normalizes candidate to one format identity
- **AND** exactly one registry definition declares that identity
- **THEN** navigation selects that definition
- **AND** registry order does not affect outcome

#### Scenario: Automatic project candidate is unknown or unsupported
- **WHEN** inference returns exact `Unknown` or no definition declares the recognized format
- **THEN** navigation advances past the candidate without adapter dispatch
- **AND** emits neither match nor failure

#### Scenario: Automatic inference cannot read candidate
- **WHEN** inference document I/O fails after unique path identity exists
- **THEN** navigation forms at most one local document failure
- **AND** does not try another adapter

#### Scenario: Multiple inferred identities are local
- **WHEN** one project candidate produces `FORMAT_AMBIGUOUS`
- **THEN** navigation forms at most one local document failure
- **AND** advances the document without trying another adapter

#### Scenario: Unclassified routing failure is local
- **WHEN** one project candidate produces unclassified `format-routing-failed`
- **THEN** navigation forms at most one local document failure
- **AND** advances the document

#### Scenario: Static registry format conflict is fatal
- **WHEN** static registry definitions contain duplicate or conflicting normalized format identities
- **THEN** navigation returns a fatal registry diagnostic
- **AND** does not use registry order as tie-breaker

#### Scenario: Explicit adapter uses manifest eligibility
- **WHEN** routing input declares a valid adapter id
- **THEN** navigation performs exact id lookup without inference
- **AND** derives candidate eligibility from that definition's finalized format descriptors
- **AND** descriptor-ineligible files are ordinary filtering

#### Scenario: Explicit adapter id is absent
- **WHEN** routing input declares an adapter id absent from the validated static registry
- **THEN** navigation returns the existing `ADAPTER_UNAVAILABLE` / `ADAPTER_NOT_FOUND` missing-id diagnostic
- **AND** does not enter descriptor prefilter or adapter dispatch

#### Scenario: Duplicate adapter id is a registry defect
- **WHEN** static registry contains duplicate adapter ids
- **THEN** registry validation fails before exact caller lookup
- **AND** caller exact lookup is not attempted

### Requirement: Request construction consumes typed resolution results

Navigation MUST construct protocol operation arguments/request envelopes, strategy-facing standard operation input, and `PreparedNavigationRequest` / core output projection as consumer-specific projections of the same typed resolution result. Standard input MUST be the closed operation-specific Rust contract shared by navigation and adapter strategies. Core-defined bindings MUST populate only strategy-visible values through compile-time fields, typed accessors, or closed enum variants rather than generic parameter lookup. `pagination.enabled` MUST combine with `limit` to normalize effective adapter limit before dispatch; `output` MUST populate only prepared/core output projection and MUST NOT enter adapter input. Protocol `Options` MUST retain stable serialized values shape. Raw argv, raw config JSON, declaration metadata, display output and serialized protocol representation MUST remain outside strategy input.

For find, request construction MUST form a closed target union. Explicit-document scope MUST retain the existing raw envelope with `document.path`. Project scope MUST construct `ProjectFindRequest` with resolved `project.root` and no `document`. Both use existing find arguments. For each project candidate, navigation MUST independently construct existing closed single-document strategy input; project root, outer page state, traversal state and project failures MUST remain outside adapter input.

#### Scenario: Read request
- **WHEN** navigation has normalized document path/ref and typed page/limit
- **THEN** it constructs read operation arguments
- **AND** adapter receives normalized facts through closed typed read input

#### Scenario: Adapter-scoped value is selected
- **WHEN** typed resolution produces core-defined adapter-scoped values
- **THEN** request construction binds only values applicable to selected adapter/operation
- **AND** strategy receives them through compile-time fields or typed accessors

#### Scenario: Existing document find request is unchanged
- **WHEN** find scope contains explicit normalized document path
- **THEN** navigation constructs the existing `document.path` raw request branch
- **AND** no project target field is added

#### Scenario: Project request has no document
- **WHEN** project scope has exact resolved root
- **THEN** navigation constructs raw request branch with `project.root`
- **AND** does not construct synthetic `document.path`

#### Scenario: Project dispatch constructs one existing strategy input
- **WHEN** project routing selects a document and adapter
- **THEN** navigation resolves that adapter's current-operation catalog view
- **AND** constructs existing single-document closed find input
- **AND** does not serialize project root/page/state into adapter options

### Requirement: Navigation dispatches linked adapter handlers

After successful typed resolution, core pre-dispatch checks and adapter selection, navigation MUST dispatch each closed standard operation input to selected linked strategy and preserve structured result/diagnostic facts for protocol/output owner. Format inference or manifest eligibility MUST NOT count as document parse or semantic validation. Selected strategy MUST perform real acquisition, decode, parse and find behavior. Once selected, its parse, semantic, operation or invalid-result failure MUST NOT trigger another inference, registry lookup or adapter dispatch. Project scope MAY perform multiple document-scoped dispatches but MUST NOT create a project-aware adapter operation.

#### Scenario: Dispatch succeeds
- **WHEN** navigation has constructed standard typed operation input
- **THEN** it calls selected adapter strategy
- **AND** preserves structured result facts for projection

#### Scenario: Explicitly selected adapter parses real document
- **WHEN** explicit descriptor prefilter admits one candidate
- **THEN** selected strategy still acquires and parses actual document
- **AND** metadata eligibility alone does not produce success

#### Scenario: Selected adapter parse fails locally
- **WHEN** selected project strategy returns parse/semantic/operation failure
- **THEN** navigation preserves one owner-compatible local diagnostic
- **AND** does not select or dispatch another adapter

#### Scenario: Project dispatch remains document scoped
- **WHEN** project find searches documents selected by different adapters
- **THEN** navigation invokes each through existing single-document find strategy
- **AND** each strategy observes only its document, query, adapter page/limit and applicable typed options

### Requirement: auto-read mode has one canonical CLI and config declaration

The core-authored document parameter catalog MUST declare `docnav.defaults.auto_read` as a `Replace` string enum with CLI locator `--auto-read`, config locator `defaults.auto_read`, built-in default `unique-ref`, no environment locator, and operation bindings limited to `outline` and single-document `find`. Resolution MUST project selected mode to core/navigation orchestration and MUST NOT serialize it into protocol arguments, adapter options or standard input. Project-find scope MUST remain outside auto-read selected view while full config validation recognizes the canonical field.

#### Scenario: Omitted eligible sources resolve built-in mode
- **WHEN** outline or single-document find has no CLI/project/user auto-read candidate
- **THEN** canonical resolution materializes `unique-ref`
- **AND** base adapter request retains existing operation-specific shape

#### Scenario: Explicit CLI overrides config
- **WHEN** CLI, project config and user config provide valid values for eligible scope
- **THEN** canonical resolution selects CLI value
- **AND** records lower-priority candidates as overridden

#### Scenario: Project config overrides user config
- **WHEN** CLI omits auto-read for eligible single-document scope
- **AND** project and user config both provide valid values
- **THEN** canonical resolution selects project value

#### Scenario: Valid config is project-scope inapplicable
- **WHEN** loaded config contains valid `defaults.auto_read`
- **AND** requested operation is pathless project find
- **THEN** full config validation recognizes field
- **AND** project-scope resolution does not project it

#### Scenario: Explicit project mode is rejected
- **WHEN** pathless project find contains explicit `--auto-read`
- **THEN** navigation returns existing scope-inapplicable input diagnostic
- **AND** no project traversal or adapter dispatch starts

#### Scenario: Invalid config value is source attributed
- **WHEN** project or user config contains an invalid auto-read enum value
- **THEN** config validation returns existing source-attributed diagnostic
- **AND** no adapter operation is dispatched

#### Scenario: Undeclared environment input has no effect
- **WHEN** process environment contains similarly named auto-read value
- **THEN** env extractor emits no auto-read candidate
- **AND** resolution continues with declared sources

## ADDED Requirements

### Requirement: Project find consumes finalized single-document units without parsing refs

Navigation MUST wrap each finalized single-document find logical unit with normalized document path and MUST preserve adapter order and opaque ref unchanged. It MUST NOT reconstruct occurrence, node, group, evidence, multiplicity or ranking semantics.

#### Scenario: One document result is added
- **WHEN** selected adapter returns validated single-document find page
- **THEN** navigation emits each complete logical unit under independent document path
- **AND** preserves exact adapter-produced ref/order

### Requirement: Project replay advances a closed three-coordinate state

Navigation MUST own deterministic project replay state `(document_position, adapter_page, logical_unit_offset)` and a positive, finite work quantum fixed within one build. Exact quantum value MUST remain implementation-private and MUST NOT be encoded in public schema/examples or treated as compatibility contract. Every non-fatal transition MUST advance a coordinate or emit one bounded failure while advancing document position. Empty-but-continuable adapter pages MUST advance adapter page. Adapter `limit` MUST remain unchanged and MUST NOT control project transition/wrapper quota.

#### Scenario: Empty adapter page cannot stall replay
- **WHEN** selected adapter page is empty and returns a validated next page
- **THEN** navigation advances adapter-page coordinate without output
- **AND** project page may remain empty with continuation

### Requirement: Project find never composes nested read

Navigation MUST return validated project find results without evaluating project refs for unique-ref auto-read and without dispatching nested read.

#### Scenario: One project match does not trigger read
- **WHEN** project page contains exactly one match wrapper
- **THEN** navigation returns project result unchanged
- **AND** does not invoke any adapter read strategy
