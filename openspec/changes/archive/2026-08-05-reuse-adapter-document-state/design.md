This design records the approved architecture for
`reuse-adapter-document-state`: ref producer/read consistency is the durable
contract; one prepared adapter document is the reuse and same-view mechanism;
algorithm count and callable shape are intentionally non-normative.

## How to Read This Design

- **Current** statements are backed by the named owner docs and source.
- **Target** statements are the approved contract expressed by this change's
  spec deltas.
- **Compatible document view** has the exact meaning defined below; it is not a
  synonym for “same path.”
- **Implementation sketch** illustrates the ownership split but does not name a
  mandatory Rust trait, object count, or method set.

`proposal.md` owns scope and impact. This file owns approved decisions, ref
laws, view compatibility, and evidence requirements. `tasks.md` owns apply
order. Stable behavior moves to the named `docs/` owners only during apply.

## Current Baseline

Current routing is lexical and target-document-I/O-free. Explicit adapter-id
lookup or automatic complete-basename matching selects one linked adapter
before the document is opened. The selected Markdown/JSON operation or hook
then independently loads and builds its own private model.

Current source anchors:

- routing and dispatch:
  `crates/shared/navigation/src/routing.rs::select_adapter` and
  `crates/shared/navigation/src/execution.rs`;
- full-read composition:
  `crates/shared/navigation/src/outline_mode/unstructured.rs`;
- unique-ref nested read:
  `crates/shared/navigation/src/auto_read.rs`;
- linked adapter interface:
  `crates/shared/adapter-contracts/src/lib.rs::Adapter`;
- Markdown state/ref paths: `MarkdownDocument::load`, `outline_entries`,
  `find_entries`, and `resolve_ref`;
- JSON state/ref paths: `reload_document`, `preorder_entries`, `find_entries`,
  and `resolve_selection`.

The Current ref owner guarantees that core passes outline/find refs unchanged
to read, but explicitly does not guarantee that the selected adapter accepts,
uniquely resolves, or successfully reads its own emitted ref. The adapter owns
correctness, yet the cross-operation success law is not currently normative.

Current adapter behavior already exhibits the intended roles:

| Role | Markdown | JSON |
| --- | --- | --- |
| prepare private view | UTF-8 source, headings, line starts, section facts | decoded JSONC source, logical tree, comments, source regions |
| ref producer: outline | headings, document head, full fallback | logical preorder and comment views |
| ref producer: find | source occurrences -> visible containing region | original-source occurrences -> direct/tail/deepest base region |
| ref consumer: read | ref -> Markdown section/head/full source | ref -> strict JSON or comment-aware JSONC materialization |
| auxiliary behavior | info, cost, full-read source facts | info metadata, cost, JSON/JSONC full-read facts |

Current successful preparation counts are secondary evidence:

| Path | Current complete preparations | Target complete preparations |
| --- | ---: | ---: |
| routing only | 0 | 0 |
| direct operation | 1 | 1 |
| cost -> full-read content | 2 | 1 |
| cost -> structured outline | 2 | 1 |
| outline/find -> nested read | 2 | 1 |

## Definitions

### Ref producer

Any adapter-owned behavior that successfully emits a caller-visible ref. The
Current producer operations are outline and find. A future preview, selection,
or other extension becomes a ref producer whenever it emits refs and therefore
inherits the same laws; an extension that emits no ref does not.

### Ref consumer

The adapter-owned read/ref-resolution behavior that interprets an opaque ref
and materializes the adapter-defined selection. The implementation may split
parsing, lookup, selection, serialization, and paging across helpers; the
contract observes the resulting read behavior.

### Compatible document view

Two views are compatible for a ref when all of the following hold:

1. the same adapter identity and ref semantics interpret the ref;
2. the adapter consumes the same source bytes/text and the same fixed
   format/configuration facts relevant to ref generation and resolution; and
3. read requires no hidden producer-only state beyond the existing read input,
   the opaque ref, and the compatible prepared view.

The same prepared state is compatible with itself. An independently prepared
view from identical source and relevant facts MUST also be compatible. An
adapter owner MAY document a broader equivalence, such as changes that do not
affect its ref-relevant structure, but this change does not infer one.

Same pathname alone is not compatibility. Source mutation, adapter/ref semantic
changes, or different relevant configuration MAY make a view incompatible.

### Auxiliary extension

Behavior that supplies cost, metadata, source/full-read facts, preview facts,
or other non-ref identity information. Its implementation may be a method,
hook, data accessor, helper, or shared projection. This change does not fix that
shape. If it emits a ref, it additionally becomes a ref producer. Readable
rendering remains an output-layer responsibility even when it consumes
adapter-produced facts.

## Approved Ref Consistency Laws

### Law 1: Opaque pass-through

Shared layers validate only the shared non-empty input boundary and pass a ref
unchanged. They do not parse, normalize, reconstruct, or infer it from display
text.

### Law 2: Producer canonicality

Every successfully emitted ref MUST be a complete non-empty canonical ref under
the emitting adapter's documented grammar. Paging or label truncation MUST NOT
truncate or rewrite it.

### Law 3: Compatible-view round trip

For every ref `r` in a validated successful producer result over view `V`, read
with valid existing read arguments and `r` MUST return a validated successful
`ReadResult` on:

1. the same prepared view `V`; and
2. any independently prepared compatible view `V'`.

The result MUST echo `r` unchanged. `REF_INVALID`, `REF_NOT_FOUND`,
`REF_AMBIGUOUS`, or another failure caused by producer/consumer disagreement is
an adapter contract violation on a compatible view, not an allowed outcome.

This law does not require runtime core to invoke read after every producer
entry. It is normative adapter behavior proved by conformance evidence.

### Law 4: No hidden producer context

A producer MUST encode or otherwise make available every identity fact that
read needs through the documented ref and compatible view. Read MUST NOT depend
on an in-memory pointer, producer call order, unexposed producer-only option, or
state that cannot be reconstructed from an independently prepared compatible
view.

Invocation-private indexes MAY accelerate resolution; they MUST NOT be the only
source of ref meaning.

### Law 5: Semantic correspondence

The resolved selection MUST correspond to the producer evidence according to
the adapter owner:

- an outline ref resolves to the documented selection represented by its entry;
- a find ref resolves to the documented owning/representative selection for the
  match evidence; and
- another ref-producing extension documents the relation it promises.

Correspondence does not universally mean the read content contains the literal
query. JSON may map whitespace, punctuation, or a child-crossing occurrence to
a container ref and return normalized content. The JSON owner defines that
relation.

### Law 6: Multiplicity remains adapter-owned

The laws do not require one-to-one identity. Multiple refs MAY select one
region, one ref MAY appear for multiple find occurrences, and read MAY return a
container-level selection. Each emitted ref still has to satisfy canonicality,
round trip, and documented correspondence.

### Law 7: Incompatible-view stale behavior remains adapter-owned

When read evaluates a ref against an incompatible view, the adapter MAY resolve
it differently or return its documented `REF_NOT_FOUND`, `REF_INVALID`, or
ambiguity behavior. The producer guarantee does not create cross-version,
cross-mutation, or cross-configuration ref stability.

## Approved Decisions

### Decision 1: Ref consistency, not method count, is the minimal contract

**Status: approved.**

The architecture does not require exactly three functions or any fixed helper
shape. Outline and find are Current ref producers; read is the Current consumer.
An adapter may implement find end-to-end, split matching from location, share a
codec, or use other private algorithms. What matters is that every public ref
emission obeys the approved laws.

### Decision 2: One adapter document owns reusable private preparation

**Status: approved.**

After routing and input/path resolution, the selected adapter creates one
invocation-private document boundary. Each adapter algorithm preserves its
Current validation-versus-document-access ordering. At the first required
access, the boundary captures and prepares one view, which later eligible work
reuses.

The adapter document is a lifecycle/state boundary, not a generic document DTO
or a public session. Concrete state remains adapter-private.

### Decision 3: One invocation does not refresh its prepared view

**Status: approved.**

After successful preparation, all later eligible producer, consumer, and
auxiliary extension work in that invocation observes the same view. Path
replacement, in-place mutation, deletion, repair, encoding change, or invalid
replacement affects the next invocation, not the prepared state already in use.

This same-view rule is necessary for composed consistency but not sufficient;
the round-trip laws and evidence are what prove producer/read agreement.

### Decision 4: Adapter owns the guarantee; shared infrastructure owns the gate

**Status: approved.**

Adapters own grammar, canonical generation, parsing, lookup, selection, and
correspondence. Core cannot prove those semantics by inspecting opaque strings.
Shared contract infrastructure owns a reusable conformance harness and requires
each built-in adapter to pass it. Navigation owns validation of composed
results, not ref grammar.

Production runtime does not double-read every emitted ref. Test evidence and
owner-specific contracts make violations defects.

### Decision 5: Auxiliary behavior is extensible and subordinate to ref identity

**Status: approved.**

Cost, metadata, full-read/source facts, preview facts, and rendering inputs may
use the smallest suitable extension form. This change neither counts them as
foundational ref algorithms nor bans additional methods. They cannot parse,
rewrite, or become an alternative identity source for refs unless explicitly
acting as a ref producer under the same laws.

### Decision 6: Current public shapes and routing remain compatibility boundaries

**Status: approved.**

Routing stays lexical, no-probe, exact/no-fallback, and state-free. Existing
operation/result shapes, ref strings, pagination/continuation, output, CLI,
config, diagnostics, and auto-read eligibility remain unchanged. Find result
unit/ranking and token-cost/rendering redesigns remain separately owned.

### Decision 7: Prepared-view reuse and ref consistency are separate guarantees

**Status: approved.**

One captured invocation view defines which prepared state eligible work reuses.
It does not, by itself, prove that a ref producer and read agree. Therefore:

1. ref production/consumption, rather than algorithm count or callable shape,
   defines the architectural boundary; and
2. compatible-view round trip is an explicit adapter contract backed by
   executable conformance evidence.

## Non-Normative Implementation Shape

```text
AdapterDefinition
  manifest / capabilities
  create_document(normalized_path) -> AdapterDocument

AdapterDocument
  private state: uninitialized -> prepared once -> released

  ref producers
    outline(...)
    find(...)
    future ref-emitting extensions...

  ref consumer
    read(ref, ...)

  auxiliary extensions
    info/cost/full-read/preview/render facts...
```

This diagram describes roles, not required methods. The first implementation
may keep the existing outward operation interface and route it through a shared
document boundary. A single adapter-private canonical ref encoder/parser is a
useful simplification, but the contract requires behavior, not that type.

The normalized path may remain in existing closed operation input for
compatibility. Once private state is prepared, later work MUST NOT reopen the
path merely because navigation dispatched another stage.

## Conformance Evidence

The shared harness MUST treat the adapter as a black box with opaque refs. For
each selected built-in adapter it must cover:

1. **same-state round trip:** collect every ref emitted by representative
   outline/find pages and read page 1 through the same adapter document;
2. **fresh-compatible round trip:** independently prepare the identical fixture
   with the same relevant facts and read every collected ref;
3. **all emitted page entries:** traverse deterministic pages to the terminal
   page and test full refs even when labels/optional facts are truncated;
4. **producer variety:** cover outline structure, repeated find refs, special
   refs, comments/virtual entries, empty/fallback structures, and adapter-owned
   visibility options;
5. **correspondence:** assert owner-specific region/selection meaning rather
   than only “read returned success”;
6. **incompatible-view boundary:** mutate structure/source deterministically and
   assert the owner-documented stale-ref outcome without treating it as a
   compatible-view failure; and
7. **non-leakage:** confirm no private handle, snapshot id, parser value, or
   conformance-only fact enters protocol/output/ref/continuation/log surfaces.

Representative fixtures and property/fuzz evidence cannot mathematically prove
every adapter implementation, but together with the normative law they make a
violation unambiguously an adapter defect. A new built-in adapter cannot claim
contract support without the same harness coverage.

## Invocation and Failure Behavior

| Path | Approved behavior |
| --- | --- |
| routing/input failure | no adapter document or target-document I/O |
| adapter semantic rejection before Current document access | preserve Current diagnostic ordering; no preparation |
| first preparation failure | existing selected-adapter path/encoding/content diagnostic; no reroute |
| direct operation | one preparation at most; execute requested behavior; release after validation |
| cost/full-read/structured fallback | reuse the same view wherever the selected adapter path participates; preserve existing fallback semantics |
| navigation default UTF-8 fallback | remains a navigation-owned exception when no prepared source capability exists; it is not a ref producer or reuse proof |
| outline/find -> nested read | producer and consumer share one view; opaque ref is unchanged; round-trip law applies |
| nested/composed validation failure | preserve validated base fallback, then release state |
| cancellation/unwind | release private state through bounded RAII; no public cleanup result |

Pagination requests remain separate invocations. “All refs” means all refs
emitted by the deterministic logical sequence for the tested view across its
pages; this change does not retain state or snapshot identity across page
requests.

## Compatibility and Handoffs

- **Ref owner:** `docs/ref-contract.md` must replace its pass-through-only
  disclaimer with the compatible-view producer/read law while retaining opaque
  shared handling and incompatible-view boundaries.
- **Markdown:** heading/document-head/full refs must round-trip on the same and
  independently prepared identical Markdown view. Existing structural snapshot
  behavior applies only when the later view is incompatible.
- **JSON:** base/direct-comment/tail refs must round-trip through the existing
  strict JSON or comment-aware JSONC materialization. JSON demonstrates why
  selection correspondence is not literal source containment.
- **Find result redesign:** may change occurrence/distinct/grouped units only
  through its own gate; every resulting emitted ref still inherits these laws.
- **Code adapter:** may use symbol-oriented search or split private algorithms;
  its emitted symbol refs must pass the same compatible-view harness.
- **Cost/rendering changes:** may change auxiliary facts or presentation through
  their owners; neither may reinterpret ref identity.
- **Interactive/service/external use:** gains no state retention across an
  unbounded prompt, request, or process boundary.

## Risks / Trade-offs

- A strong round-trip law exposes existing adapter bugs that pass-through-only
  wording tolerated. That is intended, but implementation must add failing
  evidence before refactoring.
- “Compatible” can become vague. The minimum guaranteed case is independently
  prepared identical source plus identical relevant adapter facts; any broader
  equivalence must be adapter-documented.
- A shared harness can accidentally test only happy refs. It must traverse all
  emitted fixture pages and include special/virtual/fallback refs.
- Runtime double-read would enforce the law at excessive cost and could change
  diagnostics. Keep enforcement in contract tests and composed-result
  validation.
- Longer-lived models can increase peak memory. Measure retained size/lifetime
  and release immediately after the last eligible stage.
- Auxiliary extensions can grow into a second generic framework. Add only facts
  or hooks justified by real consumers and keep ref identity single-owned.

## Migration Plan

1. Synchronize the ref, adapter, architecture, navigation, Markdown, and JSON
   owners with Decisions 1-7 before changing implementation.
2. Follow the testing/Case-maintenance workflow; prove the Current tree closes;
   add failing compatible-view round-trip and preparation-count evidence.
3. Introduce the smallest invocation-private adapter-document state boundary
   without fixing unnecessary method shapes.
4. Migrate Markdown and JSON through the boundary and make every emitted fixture
   ref pass same-state and fresh-compatible read.
5. Preserve public results and independently owned extension behavior; run
   targeted, workspace, mutation, conformance, and release verification.

Rollback restores independent operation preparation and the prior ref owner
contract. Because this change intentionally strengthens the producer/read
promise, rollback must also revert its owner docs and conformance evidence; no
wire migration is required.
