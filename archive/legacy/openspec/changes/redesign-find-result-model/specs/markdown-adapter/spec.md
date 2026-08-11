**This provisional delta preserves Markdown's Current source-occurrence find behavior until the owner approves one logical model, evidence contract, order, and bounded-work rule.**

## MODIFIED Requirements

### Requirement: Markdown find returns bounded readable matches
Markdown find MUST search according to Markdown-owned literal source semantics and return adapter-generated refs that ordinary read can consume. Find result identity, evidence, multiplicity, ordering, and final logical units MUST follow the one model explicitly approved and finalized in this requirement. Until that approval, Markdown MUST retain Current occurrence order, repeated exact refs, non-empty snippet-valued `label`, `kind: "match"`, and hit-line `location.line_start`; it MUST NOT silently deduplicate or group those occurrences.

#### Scenario: Match in section
- **WHEN** find matches text inside a Markdown section
- **THEN** the finalized logical result includes the Markdown-owned ref required by the approved model
- **THEN** read with that ref returns content corresponding to the matched region

#### Scenario: Current occurrences preserve separate evidence
- **WHEN** two Current Markdown source occurrences map to the same exact heading ref
- **THEN** find returns two occurrence items in source order
- **AND** each item retains its own non-empty snippet label and hit line

#### Scenario: Approved aggregation preserves declared evidence semantics
- **WHEN** the approved model combines multiple Markdown occurrences under one distinct ref/node or group
- **THEN** Markdown applies the finalized representative or nested evidence rule deterministically
- **AND** exact, partial, omitted, and truncated multiplicity/evidence remain distinguishable as required by protocol

#### Scenario: Ref and evidence remain independent
- **WHEN** a Markdown logical result carries a ref plus label, excerpt, location, or multiplicity facts
- **THEN** the ref remains complete and readable independently from truncation of those evidence facts
- **AND** shared layers do not need to parse the ref to present or paginate the result

#### Scenario: Unapproved model keeps occurrence behavior
- **WHEN** the product model or compatibility path remains unanswered
- **THEN** Markdown emits the Current occurrence-oriented `Entry` facts
- **AND** implementation of distinct-ref/node or grouped results remains blocked

### Requirement: Markdown pagination and cost use selected output text
Markdown outline, read, and find MUST apply the active pagination budget to selected output text and MUST report cost through shared protocol-compatible cost measurements. This change may finalize the find wire role and measured item/group scope of `cost`, but estimator choice and calculation mechanics remain owned by the independent token-cost change. Find pagination MUST operate on the finalized logical unit in its deterministic approved order, obey the approved current-page scan and retained-work bounds, and produce a continuation that reproduces unit boundaries. Source-order occurrences and first-occurrence distinct refs may use adapter-owned monotonic traversal, deterministic replay, seen-ref state, and lookahead; they do not require a complete-query proof merely to return a page. Before those choices are finalized, Current occurrence pagination MUST remain unchanged.

#### Scenario: Read exceeds budget
- **WHEN** a Markdown section exceeds the active limit
- **THEN** read returns bounded content
- **THEN** it exposes the next page value

#### Scenario: Find page uses final logical units
- **WHEN** Markdown returns a find page under the approved model
- **THEN** the active page budget is applied to complete protocol facts for the selected occurrence, distinct ref/node, group, or approved partial-group segment
- **AND** requesting the returned continuation neither loses nor duplicates such a logical unit

#### Scenario: One item exceeds the output budget
- **WHEN** one approved Markdown find logical unit cannot fit its complete optional evidence within the active page budget
- **THEN** Markdown preserves the complete ref and the finalized minimum non-empty machine facts
- **AND** it applies the approved evidence truncation, omission, or continuation rule while allowing pagination to advance

#### Scenario: Scan budget ends before an all-candidate fact is proven
- **WHEN** Markdown reaches the approved scan or retained-work bound before proving selected query-global uniqueness, an exact total, complete grouping, or a global rank/representative rule that compares all eligible candidates
- **THEN** it follows the finalized partial, omission, continuation, or diagnostic behavior
- **AND** it does not label a prefix-derived fact as query-global or complete

#### Scenario: Monotonic page proof does not require exhaustive search
- **WHEN** the approved model returns source-order occurrences or distinct refs ordered by first occurrence without an all-candidate completeness fact
- **THEN** Markdown may establish the requested page through adapter-owned monotonic traversal or deterministic replay, seen-ref state where needed, and lookahead for the next logical unit
- **AND** the approved contract records and Markdown observes the current-page scan and retained-work budget, including duplicates examined during replay or lookahead

#### Scenario: Current pagination remains before approval
- **WHEN** the final logical unit or work budget has not been explicitly approved
- **THEN** Markdown paginates Current source occurrences in their Current order
- **AND** it does not perform extra exhaustive work solely to simulate an unapproved distinct or grouped model

### Requirement: Find 命中 document head 时必须返回可 read 的区域 ref
Markdown find MUST search the complete source. When find evidence is located in the document head and the current structured outline has at least one visible heading entry, every finalized occurrence/node/group representing that evidence MUST use `HEAD:leading` as its readable ref. When the current outline uses the `doc:full` fallback, find MUST preserve the existing readable fallback behavior. Aggregation, representative evidence, or truncation MUST NOT change this Markdown-owned source-to-ref mapping.

#### Scenario: find 使用 HEAD ref
- **WHEN** query 命中第一个有效 Markdown heading 前的普通前导正文
- **AND** 当前 structured outline 至少有一个可见 heading entry
- **THEN** the finalized find logical result representing that evidence uses ref `HEAD:leading`
- **THEN** 使用该 ref 执行 read 返回包含命中文本的 content

#### Scenario: Multiple head occurrences keep one readable identity
- **WHEN** multiple query occurrences in document head map to `HEAD:leading`
- **THEN** the approved occurrence, distinct-ref/node, or grouped model preserves their finalized evidence and multiplicity semantics
- **AND** every exposed navigation identity for those occurrences remains exact ref `HEAD:leading`

#### Scenario: fallback find 保持可读
- **WHEN** query 命中 document head
- **AND** 当前 outline 使用 `doc:full` fallback
- **THEN** the finalized find logical result ref remains readable through ordinary read
- **AND** aggregation does not replace it with an output-only or shared-generated identity
