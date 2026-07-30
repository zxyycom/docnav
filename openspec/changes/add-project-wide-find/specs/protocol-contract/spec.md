**这是一个临时 Target planning artifact：目标是把 raw `find` request 扩展为 backward-compatible single-document/project closed union，并新增 `scope: "project"` success branch，使 project root 不伪装成 document path且 result 分开表达 document path、single-document unit、local failure 和 numeric page；当前 completed tasks 为 0/43，blocking gates 尚未关闭，因此它既不是 Current 行为，也不是实施授权。**

## MODIFIED Requirements

### Requirement: Protocol envelopes are self-describing

Protocol request and response envelopes MUST carry stable operation identity and enough structured context for machine validation, replay, and failure attribution. The raw `find` request MUST be a closed union of exactly `SingleDocumentFindRequest` and `ProjectFindRequest`. `SingleDocumentFindRequest` MUST retain the existing required `document: { path }` encoding and MUST NOT contain `project`. `ProjectFindRequest` MUST contain required `project: { root }`, where `root` is the resolved normalized project root, and MUST NOT contain `document`. Both branches MUST retain `operation: "find"` and the existing closed `FindArguments` fields. A request containing both targets, neither target, or extra target fields MUST be invalid rather than inferred. Response envelopes MUST retain operation/result pairing and one-response semantics.

#### Scenario: Existing document find request remains valid
- **WHEN** a request uses the existing `operation: "find"`, `document.path`, and find arguments encoding
- **THEN** it validates as `SingleDocumentFindRequest`
- **AND** no new discriminator or field is required

#### Scenario: Project find request carries resolved root
- **WHEN** caller requests project find
- **THEN** raw request validates as `ProjectFindRequest` with `project.root`
- **AND** it contains no `document` or synthetic document path

#### Scenario: Mixed target request is rejected
- **WHEN** a find request contains both `document` and `project`, or contains neither
- **THEN** closed-union validation rejects it
- **AND** runtime does not guess scope from path/query content

#### Scenario: Failure remains self-describing
- **WHEN** project request handling fails before a success result exists
- **THEN** protocol returns one failure envelope with request/operation context
- **AND** does not emit a partial sibling response

### Requirement: Operations bind to success result types

Each protocol operation MUST bind to its valid success result shape. A response is valid only when operation identity and result type match. `find` MUST accept either the finalized single-document find result for `SingleDocumentFindRequest` or `ProjectFindResult` with required `scope: "project"` for `ProjectFindRequest`; neither shape may be substituted for another operation or request branch.

#### Scenario: Outline result pairing
- **WHEN** response operation is `outline`
- **THEN** success result is an outline result
- **THEN** read, find, or info result fields are not substituted

#### Scenario: Document request returns single-document result
- **WHEN** a valid `SingleDocumentFindRequest` succeeds
- **THEN** result validates as finalized single-document find result
- **AND** does not acquire project-only wrappers or failures

#### Scenario: Project request returns project result
- **WHEN** a valid `ProjectFindRequest` succeeds
- **THEN** result validates as `ProjectFindResult`
- **AND** `scope` is exactly `"project"`

## ADDED Requirements

### Requirement: Project request remains orchestration-owned

`ProjectFindRequest.project.root`, outer page replay state and project failure aggregation MUST be consumed by core/navigation and MUST NOT be serialized into adapter `Options` or a project-aware adapter request. Each selected document MUST use the existing closed single-document strategy input, including existing query, adapter-owned limit/page and applicable options semantics.

#### Scenario: Project request dispatches one document
- **WHEN** project orchestration selects a concrete document
- **THEN** adapter receives existing single-document find input for that document
- **AND** receives no project root, outer page or traversal position

### Requirement: Project find keeps document identity separate from each match

`ProjectFindResult` MUST contain required closed `matches`, `failures`, and `page` fields plus exact discriminator `scope: "project"`. Each `matches[]` item MUST contain exactly one `document.path` and one `match` that validates as the finalized single-document find logical unit. `document.path` MUST use the existing normalized slash-path contract. Nested ref MUST remain an adapter-owned opaque string and MUST NOT contain core-added project path.

#### Scenario: Same ref in different documents remains distinguishable
- **WHEN** two documents return the same exact opaque ref
- **THEN** protocol contains two wrappers with different `document.path` values
- **AND** both refs remain byte-for-byte unchanged

#### Scenario: Project match can be passed to ordinary read
- **WHEN** project find returns a match wrapper
- **THEN** caller can pass its `document.path` and nested opaque ref to ordinary explicit-path read
- **AND** shared layers do not parse or concatenate either identity component

#### Scenario: Presentation does not replace identity
- **WHEN** project match carries label, evidence, excerpt, location, multiplicity or other finalized facts
- **THEN** those facts remain nested under `match`
- **AND** no protocol `display` field replaces path/ref identity

### Requirement: Project find reports bounded document failures inside success

Each `ProjectFindResult.failures[]` item MUST contain one normalized `document.path` and one error object using the existing protocol diagnostic projection. A local failure MUST NOT create a second response envelope. Inference document I/O, `FORMAT_AMBIGUOUS`, unclassified `format-routing-failed`, and selected parse/find failure MUST be document-local. Unknown/unsupported automatic routing and explicit manifest-descriptor mismatch MUST create neither match nor failure. Traversal/ignore, global input, explicit lookup, duplicate/conflicting static registry format-identity invariant, result-construction and output-preparation failure MUST remain top-level `ProtocolResponse::Failure`.

#### Scenario: Mixed success and local failure
- **WHEN** one project document returns matches and another selected document fails
- **THEN** one `ProtocolResponse::Success` contains match wrappers and one bounded document failure
- **AND** outer operation remains `find`

#### Scenario: Unsupported file is normal filtering
- **WHEN** inference is unknown, normalized format is unregistered, or explicit descriptor rejects a path
- **THEN** file contributes no match and no failure
- **AND** project traversal continues within its quantum

#### Scenario: Ambiguous inferred identities remain local
- **WHEN** one candidate produces `FORMAT_AMBIGUOUS`
- **THEN** one success envelope may contain its bounded document failure
- **AND** later documents may remain continuable

#### Scenario: Static registry identity conflict remains fatal
- **WHEN** static registry definitions contain duplicate or conflicting normalized format identities
- **THEN** protocol output is one top-level failure envelope
- **AND** no partial `ProjectFindResult` is substituted

### Requirement: Project find page uses fixed numeric continuation

Project request page and `ProjectFindResult.page` MUST be positive integers when present. Adapter `limit` MUST retain single-document meaning and MUST NOT set project wrapper/work quota. Each logical project page MUST use a positive, finite work quantum fixed within the current build and MUST contain a finite number of match/failure wrappers bounded by that private quantum. Exact quantum value MUST NOT be serialized or encoded in public schema/examples as a compatibility promise. Non-null response page MUST equal request page plus one; null MUST mean terminal replay state was reached. Navigation MAY conservatively return next page when the quantum ends before terminal state is proven, including an empty response caused by filtered entries or empty-but-continuable adapter pages.

#### Scenario: Project page reaches its private finite wrapper bound
- **WHEN** page reaches current build's private positive finite quantum and more state may remain
- **THEN** response stops between complete logical units
- **AND** returns request page plus one

#### Scenario: Empty adapter page advances continuation
- **WHEN** adapter returns no units and a validated next adapter page
- **AND** outer quantum ends before a project wrapper is emitted
- **THEN** project response may contain empty `matches` and `failures`
- **AND** returns request page plus one

#### Scenario: Caller requests beyond the end
- **WHEN** deterministic replay reaches terminal state before requested page
- **THEN** response `matches` and `failures` are empty
- **AND** response page is null

### Requirement: Project find success never contains auto-read

`ProjectFindResult` MUST NOT contain `auto_read`, nested read content, auto-read status or auto-read failure facts. Single-document outline/find auto-read remains governed by its finalized owner contract.

#### Scenario: One project match remains a base project result
- **WHEN** project find returns exactly one match wrapper
- **THEN** result contains no `auto_read`
- **AND** wrapper and project page retain their base meanings
