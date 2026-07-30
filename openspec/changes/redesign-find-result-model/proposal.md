**This proposal defines the decision and delivery boundary for redesigning find results without selecting an occurrence, distinct exact-ref/node, or grouped-by-ref model on behalf of the product owner.**

**Artifact state:** Planning artifacts are complete, but this change is implementation-blocked. Current occurrence behavior remains the implemented contract; the Target does not exist until task 1.2 persists the complete owner decision in `design.md` and the provisional deltas, and task 1.3 passes.

**Reading path and ownership:** Read `proposal.md` for motivation, scope, capabilities, and impact; `design.md` for the Current baseline, confirmed change-local Decisions, candidate analysis, and unresolved owner packet; the four `specs/*/spec.md` files for provisional capability-contract scaffolding; and `tasks.md` for approval, audit, implementation order, and acceptance. These change artifacts do not replace long-term owner docs or implementation evidence.

## Why

Current find results reuse the outline-oriented `Entry` shape even though Markdown emits one item per source occurrence, uses `label` as a match snippet, records the hit line in `location`, and may repeat the same readable ref. Changing that shape to distinct nodes or groups would also change ordering, continuation, auto-read eligibility, and how much source an adapter must scan before returning a bounded page, so the product model and its work budget must be approved together before implementation.

## What Changes

The items below are change obligations and post-approval work, not an approved product shape:

- Add an explicit product/architecture decision gate comparing occurrence, distinct-ref/node, and grouped results across identity, evidence, multiplicity, ordering, pagination, continuation, auto-read scope, wire compatibility, and resource bounds. No candidate is selected by this draft.
- After approval, define whether find continues to reuse `Entry` or uses a dedicated match type. The manual wire gate must give `ref`, `label`, `kind`, `location`, `summary`, `excerpt`, `rank`, `cost`, and `metadata` an explicit preserve/delete/replace disposition, precise meaning, requiredness, and compatibility treatment; any field not changed must explicitly retain its complete Current contract.
- **BREAKING**: permit an approved change to `FindResult.matches` wire shape or meaning only after the compatibility choice and migration path are explicit. The existing occurrence-oriented shape remains Current until that gate is closed and the deltas are revised to the approved contract.
- Require pagination to operate on the approved final logical unit and remain deterministic and resumable. Source-order occurrence pages and first-occurrence distinct-ref pages may be proven through adapter-owned monotonic traversal, deterministic replay, a seen-ref set, and lookahead while accounting for current-page scan and retained work. Exhaustive traversal or an authoritative complete index/count is required only for approved facts that depend on all eligible candidates, such as query-global uniqueness, exact totals, complete grouping, or global rank/representative selection that compares every candidate.
- Decide whether find unique-ref auto-read remains current-page scoped or becomes query-global. Query-global uniqueness must not be inferred from a bounded prefix without an authoritative complete index/count or an approved exhaustive scan.
- Preserve model-independent ownership: adapters own query/source-to-ref mapping, ref interpretation, result ordering, and format-specific search behavior; navigation owns auto-read composition; protocol owns machine facts; output owns presentation.
- Preserve opaque ref identity separately from occurrence or group presentation evidence. A repeated ref does not itself prove duplicate evidence, and a display string does not become a machine identity.
- Update owner docs, protocol schema/examples, evidence, shared protocol/navigation code, Markdown behavior, readable output, integration checks, and release artifacts only after the human gate is closed.
- Record a post-archive JSON contract handoff and a separate raw-facts/presentation handoff for `add-json-readable-renderer`. `add-json-adapter` has completed its tasks but is still unarchived; this change does not modify or rebase either change.

### Non-goals

- No find model, result type, field set, ordering, continuation, auto-read scope, scan budget, or compatibility strategy is approved by creating these artifacts.
- No implementation of approximate token-cost calculation or same-invocation document-state reuse; `redesign-token-cost-estimation` and `reuse-adapter-document-state` are independent changes and are not prerequisites here. In particular, this change may decide the find wire role of `cost` but does not choose or implement its estimator.
- No cross-invocation cache, search index, ranking redesign, query-language expansion, fuzzy search, or new public auto-read mode.
- No modification of JSON-specific find or readable presentation in this change. The task-complete but unarchived adapter remains untouched, and any model alignment begins only through the recorded post-archive handoff.
- No dependency on `explore-operation-composition`; that change is only the predecessor/foundation from which Current auto-read was derived, not a coordination record for this change. Current owner docs and implementation remain the baseline.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `protocol-contract`: Define the approved find logical item, machine evidence, multiplicity, ordering, pagination, continuation, compatibility, and composed-result shape.
- `output-contract`: Derive readable find presentation from the approved machine facts without turning display text into identity or changing adapter-owned semantics.
- `navigation-input-resolution`: Align unique-ref auto-read eligibility with the approved current-page or query-global find scope and its bounded-work rule.
- `markdown-adapter`: Produce the approved find logical units, evidence, order, pagination, and continuation while preserving Markdown-owned query-to-ref behavior.

All four delta specs are provisional decision scaffolds. They modify existing capability IDs, preserve Current behavior while the decision is open, and must be rewritten into one exact Target contract before implementation or archive.

## Impact

The surfaces below are potentially affected by the eventual Target; listing them does not approve a change to any surface:

- Affected public surfaces: `FindResult`, its item type and JSON schema, readable find projection, `page` semantics for find, and unique-ref auto-read behavior.
- Affected implementation areas: shared protocol and validation, adapter contracts if the result type changes, navigation auto-read selection, shared output, Markdown search/pagination, CLI smoke coverage, schemas, examples, fixtures, case ledgers, and release-package validation.
- Compatibility risk includes deserializers compiled against `matches: Entry[]`, consumers that parse `label` as a snippet, tests that assume occurrence order or repeated refs, and callers that interpret current-page auto-read as query-global uniqueness.
- An approved distinct-ref/node or grouped model may require new bounded accumulators or continuation state, but no shared producer/sink or grouping abstraction is authorized until the logical unit and work budget are fixed and at least two real consumers justify a common lifecycle.
- `add-json-adapter` is task-complete but unarchived. JSON follow-up is therefore limited to a handoff that is consumed only after archive creates a main capability; its source-oriented search and canonical ref decisions remain constraints on that handoff. `add-json-readable-renderer` may consume the final raw facts through its own tasks, but this change does not deliver its renderer.
