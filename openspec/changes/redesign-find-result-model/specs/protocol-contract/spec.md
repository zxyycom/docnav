**This provisional delta fixes model-independent find protocol constraints while leaving the Current occurrence wire unchanged until the owner selects and records one exact model.**

## MODIFIED Requirements

### Requirement: Protocol facts are structured before display
Protocol result fields MUST expose machine-readable facts instead of relying on display strings for semantics. Readable output MUST derive any display text from those facts or from adapter-owned presentation hooks. Find protocol facts MUST keep adapter-owned opaque ref identity separate from occurrence, representative, or grouped evidence. Before find behavior differs from the Current occurrence-oriented `matches: Entry[]` contract, the approved logical unit, item type, multiplicity completeness, and compatibility path MUST be recorded in this change and finalized in this requirement. The manual wire gate MUST separately cover every Current `Entry` field—`ref`, `label`, `kind`, `location`, `summary`, `excerpt`, `rank`, `cost`, and `metadata`—with an explicit preserve/delete/replace disposition, precise meaning, requiredness, and compatibility treatment, or explicitly retain that field's complete Current contract.

#### Scenario: Cost facts
- **WHEN** an operation reports cost
- **THEN** protocol output exposes structured cost measurements
- **THEN** readable output may render a compact cost summary from those measurements

#### Scenario: Navigation item facts
- **WHEN** outline, find, or info returns items
- **THEN** protocol output includes stable item facts owned by the operation
- **THEN** display text remains an output-layer convenience

#### Scenario: Repeated ref does not erase occurrence evidence
- **WHEN** two find occurrences map to the same exact opaque ref
- **THEN** protocol identity treats the ref as one navigation target
- **AND** occurrence evidence remains separate machine facts unless the approved model explicitly defines a lossless or declared lossy aggregation

#### Scenario: Multiplicity completeness is machine-readable
- **WHEN** an approved find result exposes multiplicity or grouped occurrence facts
- **THEN** the protocol distinguishes exact complete facts from lower-bound, page-local, truncated, or otherwise partial facts
- **AND** readable output is not the only place where that distinction appears

#### Scenario: Unapproved model keeps the Current contract
- **WHEN** the product and compatibility decisions have not all been recorded
- **THEN** `FindResult.matches` retains its Current occurrence-oriented `Entry[]` shape and field meanings
- **AND** no producer emits a distinct-ref/node or grouped meaning under those fields

#### Scenario: Approved wire change closes every Current Entry field
- **WHEN** the owner approves a find item/group wire contract
- **THEN** the approval records the disposition, meaning, requiredness, and compatibility treatment of `ref`, `label`, `kind`, `location`, `summary`, `excerpt`, `rank`, `cost`, and `metadata`
- **AND** any field not otherwise changed explicitly retains its complete Current contract
- **AND** the `cost` decision does not select or redefine estimator/calculator mechanics owned by the independent token-cost change

### Requirement: Page and continuation are bounded protocol facts
Paginated protocol results MUST expose bounded content and a stable continuation value or null. Callers continue through protocol fields rather than readable text parsing. Find pagination MUST operate on the one approved final logical unit, reproduce deterministic unit boundaries, and expose only completeness facts justified by the approved scan and retained-work budget. Before that model and budget are approved and finalized in this requirement, Current occurrence pagination MUST remain unchanged.

#### Scenario: More content remains
- **WHEN** a result is truncated by the active budget
- **THEN** the protocol result includes the next page value
- **THEN** the caller can request that page explicitly

#### Scenario: No content remains
- **WHEN** the returned content is complete for the request
- **THEN** the protocol result page continuation is null

#### Scenario: Find continuation uses final logical units
- **WHEN** the approved find model paginates occurrences, distinct refs/nodes, or groups
- **THEN** page boundaries and continuation are defined over that selected logical unit
- **AND** a continuation round trip neither loses nor duplicates a logical unit

#### Scenario: All-candidate completeness fact requires complete proof
- **WHEN** a find result claims query-global uniqueness, an exact query-global total or multiplicity, complete query-wide grouping, or a global rank/representative choice whose approved rule compares every eligible candidate
- **THEN** the claim is supported by an exhaustive scan or an authoritative complete adapter-owned index/count
- **AND** a bounded prefix is not serialized as a complete global fact

#### Scenario: Monotonic page facts do not require complete-query proof
- **WHEN** a find page contains source-order occurrences or distinct refs ordered by first occurrence and does not claim an all-candidate completeness fact
- **THEN** adapter-owned monotonic traversal or deterministic replay, seen-ref state where needed, and lookahead may prove the page and its continuation
- **AND** the approved contract records and bounds current-page scan and retained work
- **AND** exhaustive scan or a complete index is not required solely to preserve that order or page unit

#### Scenario: Work budget ends before proof
- **WHEN** the approved scan or retained-work budget is exhausted before a requested completeness fact is proven
- **THEN** protocol behavior follows the approved partial, omission, continuation, or diagnostic rule
- **AND** it does not silently serialize the unproven fact as complete

### Requirement: outline and find expose a success-only auto-read object

When unique-ref auto-read successfully reads the one eligible distinct ref, the outline or find result MUST include a closed `auto_read` object with `reason: "unique_ref"` and a complete existing `ReadResult`. In every other outcome, `auto_read` MUST be absent. Outline eligibility continues to use refs in its current returned structured result. Find eligibility MUST use exactly the current-page or query-global scope approved and finalized in this requirement; until that approval is recorded, the Current page-local exact-ref eligibility MUST remain unchanged and find implementation MUST NOT adopt another scope.

#### Scenario: successful auto-read contains its trigger and read result
- **WHEN** nested read returns a validated success for the eligible ref
- **THEN** `auto_read.reason` is `unique_ref`
- **AND** `auto_read.read` is the complete existing `ReadResult`
- **AND** the object contains no `mode`, `status`, sibling `ref` or `error`

#### Scenario: no successful auto-read adds no field
- **WHEN** auto-read is disabled, eligible refs are not unique under the approved scope, the result is incomplete under a scope that requires completeness, or nested read does not succeed
- **THEN** the base result contains no `auto_read` field
- **AND** no skipped reason or nested diagnostic is added elsewhere in the public result

#### Scenario: base fields remain present
- **WHEN** an outline or find result contains `auto_read`
- **THEN** the existing outline base fields or the finalized find logical-result fields retain their documented shape and meaning
- **AND** no base item is removed, reordered or rewritten by composition

#### Scenario: Query-global scope requires global proof
- **WHEN** the approved find auto-read scope is query-global
- **THEN** one eligible ref is established from the complete query result or an authoritative complete adapter-owned index/count
- **AND** one ref on the current page alone does not trigger auto-read

#### Scenario: Current scope remains normative before approval
- **WHEN** the find auto-read scope has not been explicitly approved and finalized
- **THEN** navigation uses exact non-empty refs from only the Current returned find page
- **AND** it does not inspect later pages or claim query-global uniqueness

### Requirement: existing page fields retain their operation meaning

Unique-ref auto-read MUST reuse the existing base result and `ReadResult` page fields. It MUST NOT add a generic composition continuation field. The find base `page` MUST continue the approved final find logical units independently from nested read continuation; any new group-level or cursor continuation requires explicit protocol approval and revision of this requirement before implementation.

#### Scenario: base continuation remains on the base result
- **WHEN** a base result with non-null `page` successfully triggers auto-read
- **THEN** the base `page` retains the documented next page meaning for outline or the finalized find logical unit
- **AND** nested read does not consume, replace, or advance that base continuation

#### Scenario: read continuation remains nested
- **WHEN** nested read succeeds with a non-null page
- **THEN** `auto_read.read.page` retains the documented next read page number
- **AND** the caller can continue normal read using the nested read ref and page

#### Scenario: New continuation is not implicit
- **WHEN** a grouped or distinct-ref candidate cannot resume correctly with the existing integer find page
- **THEN** implementation remains blocked until the owner explicitly approves and specifies a new continuation contract
- **AND** no renderer-only or adapter-private cross-request token is exposed as the public continuation
