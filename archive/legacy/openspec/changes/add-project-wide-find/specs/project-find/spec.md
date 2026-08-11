**这是一个临时 Target planning artifact：目标是定义省略 path 的 `docnav find` 如何在 current project root 内以 per-directory deterministic DFS、final inference/manifest routing seam 和 fixed-quantum numeric replay 编排 single-document find，同时保持 path 与 opaque ref 独立；当前 completed tasks 为 0/43，blocking gates 尚未关闭，因此它既不是 Current 行为，也不是实施授权。**

## ADDED Requirements

### Requirement: Pathless find selects the resolved project root

`docnav find` MUST use project scope only when caller omits document path. Project scope MUST use the current invocation's existing project-root resolution and MUST remain confined to that root. Supplying any path MUST retain single-document find and existing file validation.

#### Scenario: Nearest project marker defines scope
- **WHEN** pathless find starts below a directory containing the nearest `.docnav/`
- **THEN** discovery uses that directory as project root
- **AND** does not search its parent or a sibling project

#### Scenario: Cwd is the fallback root
- **WHEN** pathless find starts without an ancestor `.docnav/`
- **THEN** discovery uses invocation cwd as project root

#### Scenario: Explicit path remains single-document
- **WHEN** caller supplies a file path to find
- **THEN** project discovery does not run
- **AND** existing single-document find applies

### Requirement: Project discovery uses per-directory sorted deterministic DFS

Project discovery MUST recursively consider regular files under project root, apply project-local `.gitignore`, nested `.gitignore`, and `.ignore` rules, always exclude `.docnav`, `.git`, `.hg`, and `.svn` control directories, and ignore user-global ignore configuration. It MUST include ordinary hidden entries unless a project-owned rule excludes them, MUST NOT follow file or directory symlinks, and MUST NOT exclude a regular file by size. Each directory's immediate entries MUST be buffered only at directory scope, normalized losslessly, sorted by case-sensitive UTF-8 bytes, and visited depth-first in that order. Implementation MUST NOT precollect or globally sort all project entries.

#### Scenario: Directory-local order drives depth-first traversal
- **WHEN** one directory contains multiple files and child directories
- **THEN** its immediate entries are visited in normalized byte order
- **AND** each selected child directory is traversed depth-first before the next sibling
- **AND** no flat all-project candidate list is built

#### Scenario: Project-local ignore excludes an entry
- **WHEN** a project-owned ignore rule excludes a file or subtree
- **THEN** discovery does not route or search that entry

#### Scenario: Global ignore cannot change the checkout result
- **WHEN** two callers have different user-global Git ignore configuration
- **AND** project files and project-local ignore rules are identical
- **THEN** project discovery produces the same traversal sequence

#### Scenario: Symlink is not traversed
- **WHEN** project root contains a symlink to a file or directory
- **THEN** discovery skips that symlink without following it
- **AND** cannot reach project-external content through it

#### Scenario: Large supported file is not silently skipped
- **WHEN** an eligible regular file is selected for an adapter
- **THEN** file size alone does not remove it from project find
- **AND** the adapter's existing single-document behavior applies

### Requirement: Traversal dependency is approval-gated

Any walker/ignore dependency MUST remain a candidate until a change-local audit compares ecosystem, maintenance, security, license, MSRV, supported targets, dependency/package size, startup, correctness, alternatives and no-new-dependency. A named human MUST approve the exact crate, version and features or approve no new dependency before any Cargo manifest, lockfile or production traversal modification.

#### Scenario: Candidate library has not been approved
- **WHEN** investigation has identified a preferred walker but no named human approval is recorded
- **THEN** implementation does not modify Cargo manifests or lockfile
- **AND** production traversal work remains blocked

#### Scenario: No-new-dependency is selected
- **WHEN** the audit and named human approval select the no-new-dependency alternative
- **THEN** implementation uses only the approved existing workspace/platform facilities
- **AND** does not add an unreviewed ignore parser or wrapper framework

### Requirement: Automatic project routing uses one final inference-to-exact-adapter seam

After global registry/catalog validation, automatic project routing MUST invoke the approved inference implementation exactly once for each eligible file within one project request/replay, normalize a recognized outcome to the predecessor-owned project format identity, and exact-match that identity against adapter manifest format ids. The invocation-local selection MUST be reused while that document advances through adapter pages. A later project page request MUST reconstruct selection during its fresh deterministic replay and MUST NOT use a cross-run selection cache. The predecessor's exact `Unknown` outcome and recognized-but-unregistered formats MUST be ordinary filtering. Candidate-specific inference document I/O failure, `FORMAT_AMBIGUOUS` caused by multiple inferred identities, and unclassified `format-routing-failed` MUST each be one bounded local document failure and advance that document. Only a duplicate/conflicting static registry format-identity invariant MUST be routing-global fatal. Once exactly one adapter is selected, its real single-document parse/find MUST run and navigation MUST NOT try another adapter.

#### Scenario: Multiple supported formats select exact adapters
- **WHEN** deterministic discovery reaches supported Markdown and JSON files
- **THEN** each file performs one automatic inference
- **AND** each normalized format exact-matches its one registered adapter
- **AND** each selected adapter receives one document-scoped find input at a time

#### Scenario: Unknown or unsupported format is normal filtering
- **WHEN** inference returns its exact `Unknown` outcome or no adapter declares the recognized normalized format
- **THEN** project routing advances past the file
- **AND** emits neither match nor document failure

#### Scenario: Inference document I/O fails locally
- **WHEN** inference cannot read one uniquely identified candidate
- **THEN** project result records at most one local failure for that document
- **AND** bounded orchestration may continue to later entries

#### Scenario: Multiple inferred identities fail locally
- **WHEN** one candidate produces `FORMAT_AMBIGUOUS` because inference yields multiple identities
- **THEN** project result records at most one local failure for that document
- **AND** advances to later deterministic entries

#### Scenario: Unclassified routing failure is local
- **WHEN** one candidate produces unclassified `format-routing-failed`
- **THEN** project result records at most one local failure for that document
- **AND** advances to later deterministic entries

#### Scenario: Static registry format identity conflicts
- **WHEN** static registry definitions contain duplicate or conflicting normalized format identities
- **THEN** project find returns a top-level fatal diagnostic
- **AND** registry order does not choose a winner

#### Scenario: Selected parse fails
- **WHEN** exact routing selects one adapter but its real parse or find execution fails
- **THEN** project result records one bounded local document failure
- **AND** navigation does not infer again or dispatch another adapter

### Requirement: Explicit adapter project routing uses manifest eligibility before real parse

When caller declares an adapter, navigation MUST perform one exact adapter-id lookup before traversal and MUST skip inference. A missing adapter id MUST return the existing `ADAPTER_UNAVAILABLE` / `ADAPTER_NOT_FOUND` missing-id diagnostic. Duplicate adapter ids MUST be rejected by static registry validation before caller exact lookup. For a valid definition, navigation MUST derive deterministic file eligibility from that definition's finalized manifest format descriptors using the descriptor owner's approved comparison semantics. Descriptor-ineligible files MUST be ordinary filtering. Every eligible file MUST still run that selected adapter's real single-document parse/find; metadata eligibility MUST NOT be treated as parse success and navigation MUST NOT fall back to another adapter.

#### Scenario: Explicit adapter id is missing
- **WHEN** caller declares an adapter id absent from the validated static registry
- **THEN** project find returns the existing `ADAPTER_UNAVAILABLE` / `ADAPTER_NOT_FOUND` missing-id diagnostic
- **AND** does not enter descriptor prefilter or adapter dispatch

#### Scenario: Duplicate adapter id fails registry validation
- **WHEN** static registry contains duplicate adapter ids
- **THEN** registry validation fails before caller exact lookup
- **AND** caller exact lookup is not attempted

#### Scenario: Explicit adapter descriptor excludes a file
- **WHEN** a candidate path does not match any finalized format descriptor for the explicitly selected adapter
- **THEN** project routing advances without adapter dispatch
- **AND** emits neither match nor failure for that ordinary filter

#### Scenario: Explicit adapter descriptor admits malformed content
- **WHEN** a candidate path is descriptor-eligible but the selected adapter rejects its real content
- **THEN** project result records one bounded local failure for that document
- **AND** no other adapter is selected

### Requirement: Project match identity combines independent path and opaque ref facts

Each project match MUST contain a normalized document path and one complete logical unit from the finalized single-document find contract. Cross-document navigation identity MUST be the pair of that `document.path` and the unit's adapter-owned opaque ref. Core, navigation, protocol and output MUST preserve both strings independently and MUST NOT add project path data to ref.

#### Scenario: Equal refs in two documents are different navigation targets
- **WHEN** two documents produce the same exact ref string
- **THEN** different document paths keep their project matches distinguishable
- **AND** neither ref is rewritten or deduplicated

#### Scenario: Project match continues through ordinary read
- **WHEN** caller selects a project match
- **THEN** caller can invoke ordinary read with that document path and exact ref
- **AND** the selected document adapter remains the only ref parser

### Requirement: Project ordering composes deterministic DFS with adapter order

Documents MUST be processed in the per-directory sorted deterministic DFS sequence. Within one selected document, project find MUST preserve finalized single-document logical-unit order. Project find MUST NOT globally sort project-relative paths, relevance-rank, lexically sort refs, or interleave units using shared semantic interpretation.

#### Scenario: Nested directory order is depth-first
- **WHEN** an earlier sorted child directory and a later sibling file both produce units
- **THEN** units from the child subtree precede units from the later sibling

#### Scenario: Adapter order remains intact
- **WHEN** one selected adapter returns multiple finalized logical units
- **THEN** their relative order in project matches is unchanged

#### Scenario: Path identity cannot be represented uniquely
- **WHEN** traversal cannot form a unique lossless normalized slash path for an entry
- **THEN** project find returns a top-level fatal diagnostic
- **AND** does not emit lossy or colliding document identity

### Requirement: Adapter limit remains single-document owned

Resolved positive `limit` MUST retain its existing adapter-owned result-budget meaning for every selected single-document dispatch. Project traversal, dispatch count, outer wrapper count and continuation MUST NOT reinterpret `limit` as their quota.

#### Scenario: Project page invokes an adapter
- **WHEN** project routing dispatches one selected document
- **THEN** the adapter receives resolved `limit` under the existing single-document contract
- **AND** project work accounting does not decrement or redefine that value

### Requirement: Project pages use a fixed advancing transition machine

Project owner MUST define a positive, finite, non-configurable work quantum that is fixed within one build and whose exact value remains implementation-private. Replay state MUST be `(document_position, adapter_page, logical_unit_offset)`. Each non-fatal transition MUST advance at least one component: filtering advances document position; a local failure emits one complete failure and advances document position; emitting a complete match increments logical-unit offset; consuming an empty or exhausted continuable adapter page advances adapter page and resets offset; consuming a terminal page advances document position and resets adapter page/offset. One transition MUST emit at most one complete match/failure wrapper, so every project page MUST be finite and contain no more wrappers than the current build's private quantum. Exact quantum value MUST NOT be exposed as CLI/config/protocol input or encoded as a public schema/example/compatibility promise. Implementation MUST NOT precollect the complete project or retain cross-run state.

#### Scenario: Match transition advances offset
- **WHEN** current validated adapter page has a remaining finalized unit
- **THEN** one transition emits that complete wrapped unit
- **AND** increments logical-unit offset without splitting the unit

#### Scenario: Empty but continuable adapter page advances
- **WHEN** selected adapter returns no logical units and a validated non-null next page
- **THEN** one transition advances adapter page and resets logical-unit offset
- **AND** does not repeat the same replay state

#### Scenario: Local failure advances document position
- **WHEN** one candidate produces a bounded local failure
- **THEN** one transition emits at most one failure wrapper
- **AND** advances to the next deterministic document position

#### Scenario: Quantum ends without output
- **WHEN** the current build's finite advancing-transition quantum produces no match or failure and terminal state is not proven
- **THEN** project result may contain empty `matches` and `failures`
- **AND** returns a non-null continuation

### Requirement: Numeric continuation deterministically replays the transition state

Project page input MUST remain a positive integer defaulting to `1`. To answer page `n`, navigation MUST start from `(0, 1, 0)`, replay the preceding `n - 1` logical page steps using that build's same private quantum while discarding earlier outputs, and then execute one finite bounded step for page `n`. A non-null response page MUST equal request page plus one when terminal state is not proven; terminal state MUST return null. Stable project root, query, adapter intent, options, adapter limit, files/content and project-local ignore rules under the same build MUST reproduce ordering and page boundaries. Implementation MUST NOT create an opaque cursor, persistent result set, snapshot or cross-run cache.

#### Scenario: Stable project state reproduces a page
- **WHEN** caller repeats the same project page request with unchanged project inputs and state
- **THEN** traversal, routing and adapter replay reproduce the same wrappers and boundary

#### Scenario: Empty adapter continuation remains actionable
- **WHEN** empty-but-continuable adapter pages consume the outer quantum
- **THEN** project result may be empty
- **AND** its response page is request page plus one

#### Scenario: Caller requests beyond the end
- **WHEN** replay reaches terminal state before requested project page
- **THEN** response `matches` and `failures` are empty
- **AND** response page is null

#### Scenario: Project changes between calls
- **WHEN** files, contents or project-local ignore rules change between invocations
- **THEN** later invocation evaluates current state
- **AND** protocol does not claim snapshot consistency

### Requirement: Document failures are local but global invariants remain fatal

After a unique normalized document identity exists, candidate metadata/open, inference document I/O, `FORMAT_AMBIGUOUS`, unclassified `format-routing-failed`, selected adapter parse/semantic/find, or selected result validation failure MUST produce at most one bounded document failure, advance that document, and MUST NOT cancel later independent work within the quantum. Project-root/nested traversal or owned-ignore failure, identity collision, global input/config/catalog failure, explicit adapter lookup failure, duplicate/conflicting static registry format-identity invariant, project result validation or output preparation MUST remain fatal.

#### Scenario: One bad document does not cancel another
- **WHEN** one selected document returns matches and another selected document fails
- **THEN** project result contains successful wrappers and the bounded local failure
- **AND** orchestration may continue within remaining transitions

#### Scenario: Failure-only page is a success
- **WHEN** a project page contains only bounded document failures
- **THEN** it remains a project find success
- **AND** callers can continue through its page field

#### Scenario: Fatal traversal failure stops invocation
- **WHEN** project root, nested directory or owned ignore source cannot be enumerated deterministically
- **THEN** project find returns the existing top-level failure
- **AND** does not substitute a partial success

### Requirement: Project find does not auto-read

Project find MUST NOT evaluate path/ref pairs for unique-ref auto-read, dispatch nested read, or expose `auto_read`. Single-document find MUST retain the auto-read contract finalized by its owner.

#### Scenario: One project unit remains uncomposed
- **WHEN** current project page has exactly one match
- **THEN** project result contains that match and page facts only
- **AND** no nested read is attempted
