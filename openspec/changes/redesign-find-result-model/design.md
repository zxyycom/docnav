**This design records the Current find baseline, the three product candidates, and the approval/evidence gates required before any find-result implementation may begin.**

## State and interpretation

- **Current** means behavior that remains in force and must be verified from owner docs, code, tests, or release artifacts. In this change, Current is occurrence-oriented `FindResult.matches: Entry[]` plus current-page exact-ref auto-read.
- **Target** will mean one exact approved result model, wire contract, work budget, and migration path. Target language may be written only after task 1.1 approval is persisted by task 1.2.
- **Unresolved** means the change deliberately records alternatives without choosing among them. Candidate tables, scenario consequences, and conditional delta language are not recommendations or implementation branches.
- **Provisional delta** means decision scaffolding under `specs/*/spec.md`. Before implementation or archive, task 1.2 must replace Current-preservation and candidate-dependent clauses with the exact Target; task 1.3 audits that rewrite.

The gate is change workflow, not a runtime feature flag. Implementations must not switch behavior according to whether an approval record exists.

## Context

Docnav's navigation path is `outline -> ref -> read`. Find supplements that path by locating query evidence and returning adapter-owned opaque refs that ordinary read can consume.

Current behavior is occurrence-oriented even though the shared type does not name that semantic explicitly:

| Surface | Current behavior |
| --- | --- |
| Shared protocol | `FindResult.matches` is `Entry[]`; `ref` and `label` are required non-empty strings, while `kind`, `location`, `summary`, `excerpt`, `rank`, `cost`, and `metadata` are optional. Find and outline therefore share all nine field names even where find does not currently populate an optional field. |
| Markdown search | Literal source matches are emitted in source occurrence order. Each occurrence becomes one item; the same exact ref can occur more than once. |
| Markdown facts | `label` is a non-empty snippet around the hit, `kind` is `match`, and `location.line_start` is the hit line. `excerpt` is currently absent. |
| Ref relation | Each item ref identifies a readable Markdown region, not a unique occurrence. Ref identity and occurrence evidence therefore have different cardinalities. |
| Pagination | The adapter paginates the occurrence item stream by the current item-text budget and returns the next integer page or null. |
| Auto-read | After validating the base response, navigation string-exact deduplicates non-empty refs from the current returned page. Exactly one distinct ref triggers one nested read from read page 1; later find pages are not inspected. |
| Readable output | The renderer derives display text from raw item facts and preserves the base `matches` projection. It does not own result identity, grouping, or auto-read selection. |

The Current code materializes all Markdown occurrences before applying item pagination, but that implementation detail does not make an exhaustive scan a product requirement. Source-order occurrence pages and first-occurrence distinct-ref pages can instead be justified through adapter-owned monotonic traversal, deterministic replay, a seen-ref set, and lookahead. This change must distinguish those page proofs from query-global completeness facts and distinguish required work from accidental work in the present implementation.

`explore-operation-composition` is the historical predecessor/foundation from which Current auto-read was derived. It is not an implementation prerequisite or an alternate source of Current behavior; Current owner docs, main specs, code, tests, and release artifacts are the baseline.

### Stable terminology

| Term | Meaning in this change |
| --- | --- |
| Candidate labels | **Occurrence**, **distinct exact-ref/node**, and **grouped by exact ref** are the three unresolved models. Shorter table headings refer to these same candidates and do not introduce extra variants. |
| `matches` | The Current top-level `FindResult` array field. Its retention or replacement is unresolved; its name does not imply a dedicated `Match` wire type. |
| Current `Entry` | The shared Current item shape with exactly nine fields under review: required `ref` and `label`; optional `kind`, `location`, `summary`, `excerpt`, `rank`, `cost`, and `metadata`. |
| Source occurrence | One query hit in adapter-owned source order. It is evidence, not automatically a unique navigation identity. |
| Opaque ref | A complete adapter-generated navigation identity passed and compared string-exactly outside the adapter. Multiple occurrences may carry the same ref. |
| Final logical unit | The public item, node, group, or approved group segment on which ordering, page boundaries, and continuation operate. |
| Evidence | Machine facts such as label/snippet, excerpt, location, representative hit, nested occurrences, or multiplicity. Evidence does not silently become identity. |
| Result scope | The declared set over which a fact is true, such as the current returned page or the complete query. |
| Complete fact | A fact whose declared scope has been fully proven. Query-global uniqueness, exact totals, complete grouping, and all-candidate rank/representative choices require exhaustive traversal or an authoritative complete adapter-owned index/count. |
| Work bound | Both scan work for the current request—including replay, duplicate skipping, and lookahead—and retained work such as refs, groups, evidence, counters, offsets, indexes, or spill bytes. |

The semantic dependency is directional: the selected model maps occurrences to final logical units; the logical unit fixes field/evidence roles and ordering; ordering fixes pagination and continuation; returned refs plus declared completeness determine auto-read eligibility; those claims determine proof obligations and work bounds; all observable changes determine compatibility and migration. Choosing a later link cannot silently redefine an earlier one.

### Constraints

- Ref remains an adapter-owned opaque non-empty string outside the selected adapter. Shared layers may use string-exact equality but must not parse ref structure to derive node or group identity.
- Adapter owners retain query semantics, source-to-ref mapping, ordering facts, ref generation/interpretation, and format-specific search limits.
- Navigation owns auto-read eligibility and composition. Adapters must not pre-compose read or encode auto-read eligibility into refs or display facts.
- Protocol owns machine-readable identity, evidence, multiplicity, page, and continuation facts. Output owns readable presentation derived from those facts.
- A paginated result must be bounded, deterministic, and resumable in terms of its final logical unit. Returning a bounded wire page is insufficient if producing it performs unbounded hidden work not approved by the contract.
- Current continuation is an integer next-page value. Any cursor, nested group continuation, cross-request retained state, or result-set identity would be a separate public contract decision.
- Protocol JSON is a process-boundary contract. Changing `Entry[]`, field meaning, item order, or page meaning affects external consumers even when the Rust API compiles.
- Direct linked adapters return the shared `FindResult`; changing its item type can also change the linked Rust adapter contract.
- `add-json-adapter` has completed all of its tasks but remains unarchived and therefore has not yet produced a main `json-adapter` capability. JSON is valid design evidence, but this change may record only a post-archive handoff and must not modify or rebase the unarchived change.
- The active `add-json-readable-renderer` change owns JSON-specific presentation. It may later consume approved raw facts, but this change must not implement or rebase its renderer.
- Approximate token cost and same-invocation document-state reuse are independent changes. Neither is a prerequisite for choosing find semantics, and neither may be smuggled into this change.

### Stakeholders

- Product owner: chooses what one find result means and what evidence users need.
- Protocol/architecture owner: approves wire compatibility, continuation, and bounded-work consequences.
- Adapter owners: prove that the selected model can be produced deterministically from format-private source and refs.
- Navigation owner: defines auto-read scope without taking over adapter search semantics.
- Output owners and machine consumers: consume the approved raw facts without rebuilding hidden identity rules.
- Test/release owners: prove schema, examples, cases, integration, and packaged behavior remain aligned.

## Goals / Non-Goals

**Goals:**

- Make the occurrence, distinct-ref/node, and grouped candidates directly comparable as complete product contracts rather than as isolated data shapes.
- Bind each candidate to explicit identity, evidence, multiplicity, ordering, pagination, continuation, auto-read, scan-work, retained-work, and compatibility semantics.
- Freeze model-independent ownership and opacity invariants now.
- Require explicit user or designated product/architecture-owner approval before revising the provisional deltas into an implementable contract.
- Provide an implementation sequence that updates contract/schema/examples before evidence and code.
- Record a post-archive JSON handoff and a separate renderer handoff without implementing or modifying either owning change here.

**Non-Goals:**

- Selecting or recommending a find model without the required product decision.
- Adding a user-selectable find model, adapter-specific implicit wire variants, or a new query language.
- Designing fuzzy search, ranking, relevance scoring, regex, search indexes, or cross-document search.
- Introducing cross-invocation caches, public sessions, result-set IDs, or opaque continuation cursors unless the approved model separately requires and approves them.
- Solving approximate token calculation, document-state reuse, or general operation composition.
- Modifying JSON find/readable behavior here; this change records only the post-archive adapter handoff and the separate renderer handoff.
- Treating display strings, excerpts, source locations, or ref spelling as interchangeable identities.

## Decision dependency chain

The owner must approve one internally consistent packet. The order below explains dependency, not permission to implement a partial answer:

1. Select occurrence, distinct exact-ref/node, or grouped-by-ref logical units and decide whether one shared model or an explicit public discriminator applies.
2. Select the Rust/wire type and top-level field, then close the nine-field `Entry` gate and any new multiplicity or nested-evidence fields.
3. Define deterministic ordering, representative/nested evidence, final page unit, continuation, and reproducibility for page `k`.
4. Define current-page or query-global auto-read from the refs and completeness facts actually exposed by the selected units.
5. Bound first-page and subsequent-page scan work plus retained work; choose budget-exhaustion behavior. Require complete-query proof only for selected all-candidate facts.
6. Select compatibility/versioning, migration, rollback, and post-archive consumer handoffs.

Tasks 1.1–1.3 close this packet: human approval, persistent exact contract, then blocking artifact audit. Agent analysis may improve the packet but cannot approve it.

## Decisions

### Decision 1: This is one standalone find-model change

**Status: confirmed by the user.**

Find result semantics, auto-read consequences, protocol compatibility, adapter production, and readable projection form one vertical slice. Approximate token cost and same-invocation state reuse are different modifications and live in separate, independent changes. Approval or implementation of either other change does not unblock this one, and no artifact in this change coordinates their delivery.

`explore-operation-composition` is only the predecessor/foundation from which Current auto-read was derived; it is not a coordination record or implementation prerequisite. Current auto-read and its owner evidence are the behavioral baseline.

### Decision 2: Artifact-ready does not mean implementation-approved

**Status: confirmed.**

This change may contain complete proposal, design, provisional delta specs, and tasks so that OpenSpec considers its planning artifacts present. It remains **implementation-blocked** until a user or designated product/architecture owner explicitly approves every item in the find decision packet.

Agent review, benchmarks, code convenience, existing `Entry` reuse, or one adapter's natural representation cannot close that gate. After approval, the selected answers must be recorded as new numbered Decisions, removed from Open Questions, and written into the delta specs before the blocking audit can pass.

### Decision 3: Identity, evidence, and presentation remain separate

**Status: confirmed as a model-independent invariant.**

- `ref` is adapter-owned navigation identity and remains opaque outside the adapter.
- An occurrence is source evidence. Multiple occurrences may map to one exact ref without becoming duplicates.
- A distinct node or group may use exact opaque-ref equality as its shared identity rule, but shared layers must not parse or normalize ref spelling.
- `label`, `excerpt`, and `location` are evidence/presentation facts whose precise roles must be approved. None silently becomes an item ID.
- Readable `display` is derived output and cannot carry a machine fact omitted from the protocol.
- Adapters may expose format-specific facts in approved structured fields/metadata, but they may not select different undisclosed find models. Any cross-adapter variant requires a public discriminator and explicit approval.

This separation preserves `outline -> ref -> read` for all candidates while avoiding the false equation “one ref equals one hit.”

### Decision 4: Adapter, navigation, protocol, and output ownership do not move

**Status: confirmed as a model-independent invariant.**

The selected adapter owns query matching, source-to-ref mapping, adapter-private node knowledge, format order, representative evidence extraction, and ref interpretation. Navigation owns unique-ref auto-read selection after a validated base result. Protocol owns the public logical result and machine facts. Output owns only presentation derived from the immutable protocol response.

Alternatives rejected:

- Navigation-level grouping by parsed ref is invalid because navigation cannot interpret adapter-private ref grammar or reconstruct format order/evidence.
- Renderer-level grouping is invalid because protocol JSON and readable output would expose different logical results and auto-read would not see the renderer's groups.
- Adapter-triggered nested read is invalid because composition and output-independent orchestration belong to navigation.

### Decision 5: Logical unit and bounded work are one approval

**Status: confirmed as a decision rule; the unit and budget remain open.**

Pagination is defined over the final logical unit selected for public find results, not over a convenient intermediate occurrence stream. The approved contract must state:

1. how far the adapter may scan before returning the current page;
2. how much match/group state it may retain while doing so;
3. whether counts and uniqueness are exact, lower-bound, partial, or absent;
4. how lookahead establishes a non-null continuation;
5. how a caller resumes without losing or duplicating logical units; and
6. what happens when the work bound is reached before the requested fact can be proven.

A bounded output array does not justify unrecorded or unbounded hidden work. An exhaustive traversal or authoritative complete adapter-owned index/count is required only for a declared fact whose correctness depends on all eligible candidates: query-global unique-ref proof, an exact query-global total or multiplicity, complete grouping across the query, or global rank/“best” representative selection whose approved rule compares every eligible candidate.

That complete-query proof is not required merely to return source-order occurrence pages or distinct-ref pages ordered by each ref's first occurrence. An adapter may prove those pages with monotonic traversal, deterministic replay for page `k`, a seen-ref set, and enough lookahead to establish the next logical unit. The approved contract must still record and bound the scan work actually performed for the current page—including duplicate occurrences traversed during lookahead or replay—and the retained seen-ref/evidence/offset state.

Alternatives rejected:

- “Implement the attractive shape, then optimize” could make unbounded work part of the public behavior before resource limits are defined.
- “Always scan everything” conflicts with Docnav's finite, continuable navigation objective unless explicitly approved for a bounded document class.
- “Always stop at output budget” cannot produce exact global facts and must not label partial evidence as complete.

### Decision 6: Compatibility is explicit, not inferred from field similarity

**Status: confirmed as a decision rule; the migration choice remains open.**

Current consumers may depend on `matches: Entry[]`, occurrence order, repeated refs, snippet-valued `label`, hit-line `location`, integer `page`, and page-local auto-read. A shape that still serializes as objects is not compatible if any of those meanings change.

The owner must explicitly choose one of:

- preserve the Current wire and semantics;
- make an additive compatible extension with defined old-consumer behavior;
- run a bounded dual-read/dual-field migration window with one declared source of truth; or
- make an intentional breaking protocol/Rust contract change with versioning, release notes, fixtures, and rollback boundaries.

The implementation must not emit old and new meanings under the same field without a migration rule.

### Decision 7: Shared producer, sink, or accumulator abstractions are deferred

**Status: confirmed.**

Occurrence streaming, distinct-ref accumulation, and complete grouping have different lifecycle, memory, continuation, and failure needs. No shared iterator/producer/sink/grouping boundary is authorized merely because all candidates traverse matches.

After the model and work budget are approved, implementation should start with the smallest adapter/private mechanism. A shared boundary requires at least two real consumers with the same logical unit, lifecycle, error, budget, and continuation semantics.

### Decision 8: JSON is evidence now and a handoff later

**Status: confirmed by current capability ownership.**

JSON's active decisions require source-text literal search and deterministic mapping from each source hit to a canonical readable ref. Those facts expose the same occurrence-to-ref cardinality issue as Markdown and must inform the product choice.

This change does not create a `json-adapter` delta while that capability is absent from the main specs. Once `add-json-adapter` is archived, its owner must receive a handoff containing the approved logical unit, fields, ordering, page/continuation, scan budget, and auto-read semantics. The active JSON renderer receives a separate raw-facts/presentation handoff and remains responsible for its own readable contract.

`add-json-adapter` is currently task-complete but unarchived. Nothing in this change changes its completed artifacts, adds work to it, or treats the handoff as effective before archive.

### Decision 9: Every Current Entry field requires an explicit wire disposition

**Status: confirmed as a decision rule; every field disposition remains open.**

The human approval packet is incomplete unless it covers all Current `Entry` fields: `ref`, `label`, `kind`, `location`, `summary`, `excerpt`, `rank`, `cost`, and `metadata`. For each field, the owner must state whether it is preserved, deleted, or replaced; its precise find meaning; its required/optional/omitted/null behavior; and whether that disposition is compatible, migrated, or intentionally breaking. A field with no intended change must be explicitly declared to retain its complete Current name, meaning, requiredness, and serialization behavior rather than disappearing from the discussion.

This gate decides only the find wire role and observable meaning of `cost`. Token estimator choice, approximation behavior, dependencies, and calculation resource bounds remain owned solely by the independent `redesign-token-cost-estimation` change and are not prerequisites here.

## Candidate Analysis

No row in this section is an approval or recommendation. Selecting the occurrence candidate would not by itself retain `Entry[]`, its field meanings, page semantics, or compatibility; the same nine-field, work-budget, auto-read, and migration gates apply to every candidate.

### Core model comparison

| Dimension | Occurrence | Distinct ref/node | Grouped by ref |
| --- | --- | --- | --- |
| Final logical unit | One source query occurrence. | One exact opaque ref / adapter-readable node represented once in the declared result scope. | One exact opaque ref plus a collection or summary of its occurrences. |
| Ref cardinality | Many items may carry the same ref. | One item per ref within the approved dedup scope. | One group per ref within the approved grouping scope. |
| Direct evidence | Natural: each item can carry its own hit location and excerpt. | Requires choosing representative/aggregate evidence or omitting occurrence detail. | Can retain bounded per-occurrence evidence, but completeness and nested truncation must be explicit. |
| Multiplicity | Implicit in repeated items; an explicit total is usually unnecessary. | Optional count is useful but exact query-global count generally requires exhausting matches. | Count appears natural, but exact total generally requires exhausting matches; partial count must be marked. |
| Natural order | Source occurrence order or another adapter-owned match order. | First-occurrence order, adapter node order, ref order, or score must be selected. | First-occurrence group order, adapter node order, ref order, or score must be selected. |
| Page unit | Occurrence. | Distinct ref/node. | Complete group, partial group segment, or group summary—must be selected. |
| Early first page | Often possible after enough occurrences plus bounded lookahead. | Possible for first-occurrence ordered refs if counts/representative evidence are prefix-bounded; exact totals/global uniqueness are not. | Complete groups may be impossible to finalize early when occurrences are interleaved; partial groups add continuation semantics. |
| Retained work | Current-page occurrences and bounded lookahead; current implementation may materialize more accidentally. | Seen-ref set plus representative facts; query-global dedup across page-number requests may require rescanning or a complete index. | Ref-to-group accumulator plus retained occurrence facts/counts; potentially proportional to all matches/refs. |
| Auto-read signal | Current-page exact-ref dedup is cheap; query-global uniqueness is separate. | One returned item does not prove one global ref unless the dedup scope is query-global and complete. | One returned group does not prove one global group unless grouping is query-global and complete. |
| Current wire compatibility | Highest if `Entry[]` and existing facts remain unchanged. | Semantic break even if objects still resemble `Entry`; repeated evidence disappears and field roles change. | Usually needs a dedicated nested shape and is a clear wire/Rust break. |
| Primary product strength | Complete grep-like evidence and straightforward continuation. | Compact navigation choices and less repeated ref noise. | Combines navigation choice with multiple evidence points. |
| Primary product cost | Repetition can be noisy; one heavily matched region may fill many pages. | Loses or compresses evidence; query-global uniqueness, exact totals, or all-candidate ranking can hide whole-input work. | Most complex completeness, pagination, memory, and consumer contract. |

### Work-bound comparison

Let `N` be searchable input size, `M` the number of occurrences, `U` the number of distinct exact refs, and `P` the bounded facts that fit in one public page. These symbols describe relative work only; the owner must approve concrete limits and fallback semantics.

| Fact to produce | Minimum proof obligation | Occurrence | Distinct ref/node | Grouped |
| --- | --- | --- | --- | --- |
| Current page in natural source order | Enough ordered logical units plus lookahead to know whether more exist. | Scan until `P` occurrences plus bounded lookahead; retain approximately `P`. | Scan until `P` new refs plus lookahead if ordering is first occurrence and no exact counts are required; retain seen refs, whose size may exceed `P` on later pages. | Cannot finalize complete interleaved groups from a prefix; partial groups can stop early only with explicit partial/continuation facts. |
| Page `k` with integer page and stateless requests | Reproduce all preceding unit boundaries deterministically. | Monotonically rescan/skip preceding occurrences or use adapter-private indexing. | Monotonically replay while rebuilding the seen-ref set, or use an adapter-private first-occurrence index; neither method must be query-complete merely to reproduce the requested page. | Rescan/rebuild groups or use a complete index when groups themselves must be complete; partial group boundaries must also be reproduced. |
| Query-global one-ref proof | Exhaust matches or query an authoritative complete index/count. | Up to `N` scan; retained exact-ref set may stop at two refs if only uniqueness is needed. | Proving that the complete query has only one ref requires checking all remaining matches unless an authoritative complete index/count proves it. | Complete grouping discovers all group refs; still up to `N`. |
| Exact multiplicity for a returned ref | Observe every occurrence assigned to that ref. | Not required when multiplicity is represented by items. | Up to `N` unless an authoritative count exists. | Up to `N` unless an authoritative count exists. |
| Representative first occurrence | Observe the first ordered hit for that ref. | Each occurrence is direct. | Prefix-bounded for each discovered ref under source order. | Prefix-bounded, but later group evidence may still be incomplete. |
| Representative “best” excerpt/rank | Compare all eligible evidence under a stable rule. | Only bounded if ranking is local/streamable and the contract permits partial order. | Often up to all occurrences for each ref. | Often up to all group evidence. |
| Complete occurrence list per ref | Observe and retain or spill every occurrence. | Distributed naturally across pages. | Not represented unless separately attached. | Up to `M` evidence retained/serialized or a nested continuation is required. |
| Stable non-null continuation | Prove at least one later logical unit. | One occurrence lookahead is sufficient. | May require scanning through arbitrarily many duplicate occurrences to find the next unseen ref. | May require scanning to find/finalize another group or another segment. |

An approved budget must cover both **scan work** (bytes/scalars/nodes/occurrences examined) and **retained work** (seen-ref sets, group maps, excerpts, counters, offsets, or spill files) for each current page, including replay and lookahead. It must also say whether existing adapter-private parse/index cost is counted, because an authoritative complete index can make a completeness fact cheap only after that index has been constructed.

### Item and field-shape comparison

| Choice | Benefit | Cost / unresolved contract |
| --- | --- | --- |
| Keep `Entry[]` | Maximum source compatibility for occurrence behavior; existing renderer and adapters already consume it. | `Entry` is outline-shaped and makes occurrence/node/group distinctions implicit; grouped evidence does not fit naturally; changing `label` meaning is still a semantic break. |
| Dedicated `FindMatch[]` occurrence type | Makes occurrence identity/evidence explicit and prevents outline fields from silently becoming find contract. | Rust and wire break unless introduced through an approved migration; field mapping and old-consumer behavior must be defined. |
| Dedicated distinct-node type | Can name ref identity, representative evidence, and multiplicity deliberately. | Must define dedup scope, representative rules, exact/partial counts, and loss of per-hit evidence. |
| Dedicated group type with nested occurrences | Expresses ref identity and evidence cardinality directly. | Requires group completeness, nested limit/continuation, total-count accuracy, ordering, and resource rules. |
| Discriminated union of all models | Makes variants explicit if the model is intentionally caller-selectable or adapter-variable. | Expands every consumer and permits behavior divergence; no user-selectable model is currently in scope, so this requires separate product approval. |
| Emit old and new fields together | Can stage a migration. | Duplicates payload and risks disagreement; must have one source of truth, a bounded removal window, and schema/version rules. |

The wire gate must close every Current `Entry` field, not only the fields a selected candidate expects to populate:

| Field | Current `Entry` contract / Current Markdown find | Required approval record |
| --- | --- | --- |
| `ref` | Required non-empty string; Current find uses an exact opaque readable-region ref. | Preserve/delete/replace; exact identity and read relationship; requiredness; compatibility/migration. Ref opacity remains invariant. |
| `label` | Required non-empty string; Current Markdown find uses a match snippet. | Preserve/delete/replace; snippet, structural, group, or other exact meaning; minimum content and truncation; requiredness; compatibility/migration. |
| `kind` | Optional non-empty string; Current Markdown find emits `match`. | Preserve/delete/replace; allowed meaning/value set; requiredness; compatibility/migration. |
| `location` | Optional object; Current Markdown find emits hit `line_start`. | Preserve/delete/replace; occurrence, representative, node, or group location; units/subfields/truncation; requiredness; compatibility/migration. |
| `summary` | Optional string; the shared shape permits it without establishing a new find-specific meaning here. | Preserve/delete/replace; exact find meaning and compaction; requiredness; compatibility/migration, or explicit retention of the complete Current contract. |
| `excerpt` | Optional string; Current Markdown find leaves it absent. | Preserve/delete/replace; exact/compacted evidence, query/whitespace/truncation rules; requiredness; compatibility/migration, or explicit Current retention. |
| `rank` | Optional number; no new find ranking semantics are approved here. | Preserve/delete/replace; local/global meaning, ordering relationship, completeness proof, requiredness, and compatibility/migration, or explicit Current retention. |
| `cost` | Optional structured cost object. | Preserve/delete/replace; what find item/group scope it measures; requiredness; compatibility/migration, or explicit Current retention. Estimator/calculator mechanics remain outside this change. |
| `metadata` | Optional object. | Preserve/delete/replace; permitted find facts and validation/extension boundary; requiredness; compatibility/migration, or explicit Current retention. |

Any new multiplicity or nested-occurrence field also needs an explicit name, meaning, requiredness, completeness/partial marker, limits, ordering, continuation, and compatibility treatment. A bare integer must not ambiguously mix exact, lower-bound, and page-local counts.

### Auto-read scope comparison

| Scope | Selection evidence | Work | User-visible meaning |
| --- | --- | --- | --- |
| Current-page exact refs | Deduplicate non-empty refs represented by the final logical units on the current returned page. | Bounded by the produced page. | “This page points to one ref,” not “the query has one ref globally.” Current behavior. |
| Query-global exact refs | Prove the complete query result maps to exactly one exact ref. | Exhaust matches or use an authoritative complete index/count; may scan `N` before first output. | “All query evidence maps to one ref.” Stronger but potentially expensive. This proof does not apply merely because a distinct-ref page uses first-occurrence deduplication. |
| Candidate-specific hybrid | For example, page-local for occurrence results but global for a complete grouped index. | Depends on a stable public discriminator and completion proof. | More complex and easy for consumers to misinterpret; requires explicit variant contract. |

For every candidate, a page containing one item/group can coexist with later pages containing another ref. Pagination shape alone is not proof of query-global uniqueness.

### Scenario matrix

| Scenario | Occurrence consequence | Distinct ref/node consequence | Grouped consequence | Decision/evidence required |
| --- | --- | --- | --- | --- |
| No match | Empty occurrence page. | Empty distinct page. | Empty group page. | `page: null`; no auto-read. |
| One occurrence mapping to one ref | One direct evidence item. | One node item with one representative hit. | One group with one occurrence. | All models can auto-read under either scope once success is validated. |
| Many occurrences in one Markdown section | Repeated exact ref with separate hit lines/snippets. | One ref; evidence must be selected or compressed. | One group; occurrence list/count may exceed budget. | Approve multiplicity, evidence retention, and long-group continuation. |
| Occurrences across two Markdown sections | Source-order items expose both refs. | Two node items under approved order. | Two groups under approved order. | No unique-ref auto-read after global proof; a current page may still contain one ref. |
| Interleaved refs `A, B, A` | Natural source order. | First-occurrence order can emit `A, B`; exact counts require reading final `A`. | Complete `A` cannot be finalized before later input is checked. | Approve first-occurrence vs node order and whether groups may be partial. |
| One heavily matched ref fills page 1; another ref appears later | Page 1 contains only ref `A`; Current auto-read triggers. | Page 1 may contain `A` only if dedup/page semantics permit; global distinct result also has `B`. | Page 1 may contain group `A`; later group `B`. | Explicitly choose page-local vs query-global auto-read. |
| One occurrence/excerpt exceeds page budget | Preserve complete ref and minimum evidence; advance page. | Preserve complete ref and minimum representative evidence. | Preserve group identity; nested evidence must truncate/continue deterministically. | Approve minimum fields and truncation marker/completeness facts. |
| One group has more evidence than any page can hold | Spread occurrences over top-level pages naturally. | Usually one node item; evidence details may be omitted. | Must split group, truncate nested evidence, or allow an oversized group. | Approve final page unit and continuation ownership. |
| Requested page beyond end | Empty logical-unit page with null continuation. | Same, after deterministic dedup replay/index. | Same, after deterministic group replay/index. | Existing page contract can remain only if unit boundaries are reproducible. |
| Unicode hit and whitespace-heavy context | Each occurrence location/excerpt must remain deterministic and valid UTF-8. | Representative rule must not change under truncation. | Nested occurrences must use the same evidence rule. | Approve location units, whitespace compaction, query preservation, and Unicode truncation. |
| Markdown document-head hit | Occurrence maps to `HEAD:leading` when Current visibility rules apply. | Document head is one distinct ref/node. | Group key is exact `HEAD:leading`. | Preserve Markdown-owned mapping and read roundtrip for every model. |
| Markdown `doc:full` fallback | All hits can share `doc:full`. | Entire document becomes one distinct readable ref. | One potentially very large group. | Shows why one ref does not equal one occurrence and why global grouping can be costly. |
| JSON source has repeated spelling inside one canonical node | Separate source hits may share one canonical JSON ref. | One JSON node with a representative source hit. | One ref group with source evidence. | Handoff must preserve source-text search and canonical read ref decisions. |
| JSON source spelling maps to nested nodes | Source order may differ from canonical tree/navigation order. | Must choose first-hit order or JSON node order. | Must choose group order independently of occurrence order. | Do not infer cross-format order from Markdown. |
| Code query hits declaration and many uses mapped to one symbol/body ref | Evidence-rich but noisy. | Compact symbol/body navigation loses individual uses. | Grouping is attractive but occurrence roles may differ (declaration/use). | Future code adapter is evidence for field extensibility, not implementation scope. |
| Large state/config document with most hits under root/full-document ref | Occurrence pages remain bounded. | One ref can collapse the whole result. | One huge group/count can require exhaustive scan. | Product value and work budget must be approved together. |
| Adapter already owns a complete index | Occurrence paging may use it privately. | Global dedup/count/uniqueness may be cheaper. | Complete groups may be cheaper. | Contract must not require every adapter to build such an index unless explicitly approved. |
| No complete index and work budget is exhausted | Return only a provable occurrence page or an explicit partial/error outcome. | May preserve first-occurrence distinct-ref paging, but cannot claim a complete distinct-ref set, query-global uniqueness, or exact total beyond proof. | Cannot claim complete group/count beyond proof. | Approve degradation, continuation, or stable diagnostic; never silently claim completeness. |
| Nested read fails after eligible selection | Base find response remains successful and unchanged under Current composition rules. | Same unless owner explicitly changes failure semantics. | Same unless owner explicitly changes failure semantics. | This change does not propose public nested-read failure facts. |
| Protocol JSON vs readable view | Raw occurrence facts serialize directly; display is derived. | Raw node facts serialize directly; display is derived. | Raw group facts serialize directly; display is derived. | Both output modes must consume the same immutable response. |

## Risks / Trade-offs

- [Product choice optimizes compactness but removes needed hit evidence] → Require representative/occurrence evidence scenarios and machine-field roles in the approval packet before selecting distinct/grouped.
- [A bounded page hides unrecorded work] → Specify current-page scan and retained-work budgets, including replay/seen-set/lookahead, and add large-document evidence that observes work rather than output length. Require exhaustive/index proof only for approved all-candidate completeness facts.
- [Query-global uniqueness delays every first page] → Approve it only with an exhaustive-scan budget or authoritative index; otherwise retain explicitly page-local semantics.
- [Distinct refs repeat across pages because each page deduplicates locally] → Define dedup scope over final logical units and test cross-page round trips; do not call page-local dedup a distinct-result model.
- [Complete groups require unbounded memory or spill files] → Bound nested evidence/count state and approve partial groups, private spill, or a stable limit diagnostic before implementation.
- [Changing `label` while retaining `Entry` looks wire-compatible] → Treat field meaning as contract; update schema descriptions, fixtures, examples, release notes, and consumer migration evidence.
- [New dedicated types cause a broad Rust/protocol break] → Inventory linked adapters, protocol deserializers, renderers, CLI snapshots, MCP/skim/interactive consumers, and release artifacts before choosing the migration path.
- [Page-number continuation cannot cheaply resume global grouping] → Either accept deterministic rescans, approve adapter-private complete indexes, or explicitly redesign continuation; do not add hidden cross-request state.
- [Adapters silently diverge] → Keep one shared logical model or add an explicit approved discriminator; validate Markdown and later JSON against the same shared invariants.
- [Readable output invents missing group facts] → Require raw protocol facts to be sufficient; renderer tests must prove it only projects the immutable response.
- [The task-complete but unarchived JSON change is edited opportunistically] → Record only a post-archive handoff here; do not edit or rebase `add-json-adapter`, and leave renderer implementation to its own change.
- [Independent performance changes become accidental prerequisites] → Keep token-estimator and state-reuse implementation tasks absent from this change and test find semantics independently of them.

## Migration Plan

This is a conditional execution sequence, not evidence that a Target or migration path has been approved.

1. **Close the human decision gate.** The user or designated product/architecture owner selects the result model and item/wire type; records a complete disposition for all nine Current `Entry` fields; selects multiplicity, order, page/continuation, auto-read scope, monotonic-versus-complete proof obligations, current-page scan/retained-work budget, budget-exhaustion behavior, and compatibility path.
2. **Persist the approved contract.** Add numbered Decisions, remove answered Open Questions, and rewrite every provisional delta requirement/scenario so it states one implementable behavior. Re-run the blocking artifact audit; no code task may start before it passes.
3. **Update stable validation surfaces first.** Modify the owner docs, protocol schema, schema descriptions, contract examples, compatibility/migration notes, and any direct linked adapter contract. If the change is breaking, introduce the approved protocol/version or bounded transition mechanism before changing producers.
4. **Establish evidence.** Follow the test owner and Case-maintenance workflow, prove the current tree is closed, preserve Current tests where they remain independent evidence, and add failing/current evidence for logical units, repeated refs, cross-page behavior, every approved field disposition, truncation, monotonic replay/seen-set/lookahead, all-candidate completeness proofs where selected, current-page work bounds, auto-read, and output parity.
5. **Implement the vertical slice.** Update shared protocol/validation, navigation auto-read, the minimum adapter contract surface, Markdown production/pagination, and built-in readable output. Add no shared producer/sink/group accumulator without the evidence required by Decision 7.
6. **Integrate and validate.** Run schema/example validation, targeted Rust/static/integration checks, continuation round trips, large-document resource checks, workspace verification, and release-package smoke validation.
7. **Handoff consumers without changing their active work.** `add-json-adapter` is task-complete but unarchived; after it is archived, provide the new main owner the approved raw contract and required evidence. Until then, keep the handoff recorded only here and do not edit or rebase that change. Provide `add-json-readable-renderer` only the raw-facts/presentation handoff; do not implement its change here. Record any MCP, skim, interactive-outline, or code-adapter follow-up in their own owner changes.
8. **Rollout and rollback.** For an additive compatible model, retain old-consumer evidence for the approved window. For a breaking model, release atomically with schema/examples/version notes; rollback restores the previous protocol, Markdown producer, navigation selector, renderer, and validation artifacts together. Do not leave producers and consumers on mixed meanings.

## Open Questions

All questions below require one explicit approval packet from the user or designated product/architecture owner. Until every question is answered and persisted in Decisions/specs, this change is artifact-ready and implementation-blocked.

Closure means more than answering the list conversationally: task 1.2 must record the approved answers as numbered Decisions, replace provisional alternatives in all four delta specs with one exact contract, and remove the corresponding questions. Task 1.3 must then verify that no candidate-dependent or ambiguously “resolved” language remains. Questions 1–3 establish logical identity; 4–8 establish fields, evidence, ordering, and continuation; 9–12 establish auto-read and bounded proof; 13–14 establish compatibility and handoff.

1. **Logical unit:** Is find occurrence-oriented, distinct exact-ref/node-oriented, or grouped by exact ref? Is the model shared across adapters, or is an explicit public variant/discriminator intended?
2. **Rust and wire type:** Does `FindResult.matches` remain `Entry[]`, move to a dedicated occurrence/node/group type, or use a versioned/additive transition? Is the top-level field still named `matches`?
3. **Identity:** What identifies one logical unit? If exact opaque ref equality is insufficient for the selected model, what adapter-owned identity fact is exposed without shared ref parsing?
4. **Complete Current `Entry` wire-field gate:** For each of `ref`, `label`, `kind`, `location`, `summary`, `excerpt`, `rank`, `cost`, and `metadata`, is it preserved, deleted, or replaced; what is its exact find meaning; is it required, optional, omitted, or nullable; and is that disposition compatible, migrated, or intentionally breaking? Any field not otherwise changed must explicitly retain its complete Current name, meaning, requiredness, and serialization behavior. For node/group results, how is representative evidence selected? For `cost`, decide only the wire role and measured item/group scope; estimator/calculator choice remains in the independent token-cost change.
5. **Multiplicity:** Is occurrence count absent, page-local, lower-bound, or exact/query-global? How does the wire format distinguish complete from partial counts?
6. **Ordering:** Is order source occurrence, first occurrence per ref, adapter node order, ref lexical order, rank, or another adapter-owned deterministic rule? Does the rule apply within groups and across pages?
7. **Pagination:** What is the final logical page unit? Must groups be complete, may one group span pages, or is nested evidence truncated/continued separately? Does the existing integer `page` remain sufficient?
8. **Continuation:** What exact lookahead proves another page, how are unit boundaries reproduced on page `k`, and what happens to an unfinished group? Are new cursor/result-set fields permitted?
9. **Auto-read scope:** Does unique-ref find auto-read remain based on exact refs in the current returned page, or must it prove query-global uniqueness? Which logical units contribute refs, do incomplete/partial results suppress auto-read, and does the existing `reason: "unique_ref"` remain sufficient or must scope/completeness become an explicit wire fact?
10. **Scan and proof budget:** How many bytes/scalars/nodes/occurrences may an adapter examine before producing the first and subsequent pages, including deterministic replay and duplicate-skipping lookahead? What retained seen-set/offset facts prove source-order occurrence or first-occurrence distinct-ref pages? Which selected facts require full traversal or an authoritative complete index because they assert query-global uniqueness, an exact total, complete grouping, or an all-candidate global rank? Does construction of an adapter-private parse/index count against that budget?
11. **Retained-work budget:** How many refs, occurrences, excerpts, counters, offsets, or spill bytes may be retained? Are private spill files allowed, and what cleanup/failure rules apply?
12. **Budget exhaustion:** Must the adapter degrade to occurrence results, return explicitly partial facts, continue scanning on another page, omit unproven optional facts, or return a stable diagnostic? Silent model changes are not permitted.
13. **Compatibility:** Is the approved path compatible addition, bounded dual transition, or intentional breaking change? What protocol/schema/Rust version, deprecation window, release note, and rollback proof are required?
14. **Post-archive JSON handoff:** `add-json-adapter` is task-complete but unarchived and must not be changed here. After it is archived, which new owner/change consumes this recorded model-alignment handoff, on what compatibility schedule, and which separate renderer milestone consumes the approved raw facts?
