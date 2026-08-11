This change defines the Target contract for reusable adapter document state.
Its durable center is ref producer/read consistency, not a fixed number or
shape of adapter methods.

This change is a Target plan, not Current implementation evidence. Stable
behavior remains owned by `docs/`; code, tests, and release artifacts prove
Current behavior. Read `design.md` for the ref laws, compatible-view
definition, evidence model, and non-normative implementation sketch. The deltas
in `specs/` express the Target contract. `tasks.md` owns apply order.

## Why

Current adapters independently reopen and rebuild their private document model
for outline, find, read, info, and full-read hooks. Reusing one prepared model
removes duplicate acquisition/decode/parse work, but reuse alone does not prove
the adapter's ref algorithms agree: an adapter can still emit a ref from
outline/find that its own read path cannot parse or resolve.

The Current shared ref owner deliberately guarantees only pass-through; it
explicitly does **not** guarantee that an adapter-generated ref can be read.
That is weaker than Docnav's core navigation promise:

```text
outline/find -> opaque ref -> read
```

This change therefore makes producer-to-reader consistency the foundation.
Prepared document state supplies one compatible view and avoids repeated work;
the actual guarantee comes from adapter-owned contract laws plus mandatory
conformance evidence.

## What Changes

- After no-probe routing and post-selection input/path resolution, create one
  invocation-private adapter document. At its first Current-compatible document
  access, it prepares one source/model/index/source-region view and reuses that
  state for eligible later work.
- Define **ref producers** as any adapter behavior that emits caller-visible
  refs. Current producers are outline and find; future operations or extensions
  that emit refs inherit the same laws.
- Define read/ref resolution as the **ref consumer**. Exact functions, traits,
  helper count, and whether find is one end-to-end method or several cooperating
  algorithms are implementation details rather than product contract.
- Add a compatible-view round-trip law: every ref successfully emitted by a
  producer MUST be non-empty, canonical for its adapter, and readable against
  the same or an independently prepared compatible document view using the
  existing read input. It MUST NOT require hidden producer-only state.
- Require producer/read semantic correspondence. Outline refs identify the
  entry's documented selection; find refs identify or represent their match
  according to the adapter owner. The law does not require read content to
  contain the literal query when the adapter contract defines normalized or
  container-level selection.
- Keep ref grammar and interpretation adapter-private. Core continues to pass
  refs unchanged and does not parse or runtime-double-read every result. The
  guarantee is enforced as an adapter contract with shared conformance tests,
  not by pretending core understands every grammar.
- Treat cost, metadata, full-read facts, preview/rendering facts, and similar
  auxiliary behavior as extension surfaces around the ref model. Their exact
  callable shape is not fixed by this change; they MUST NOT become a competing
  ref identity owner. Readable rendering remains owned by the output layer.
- Capture one immutable adapter document view at first successful preparation
  and do not refresh it inside the invocation. A later invocation may observe a
  changed path and is outside the earlier round-trip guarantee unless its view
  is compatible.
- Prove the contract with Markdown and JSON, including same-state and
  independently re-prepared equivalent-view round trips, all emitted refs from
  representative paged fixtures, mutation boundaries, reuse counts, cleanup,
  and private-state non-leakage.

## Non-Goals

- No exact number of adapter algorithms, mandatory Rust trait spelling,
  end-to-end-versus-split find implementation, generic parser tree, node DTO,
  downcast API, or arbitrary state lookup.
- No runtime verification that calls read for every emitted ref in production;
  that would multiply work and change operation behavior.
- No guarantee for refs evaluated against an incompatible document view,
  changed adapter ref semantics, or hidden state from another invocation.
- No requirement that different refs select different regions, that repeated
  find matches use different refs, or that read returns a literal source range.
- No redesign of find result units, ranking, paging/continuation, token-cost
  estimation, readable rendering, protocol/output shapes, CLI/config, or ref
  spelling.
- No probe restoration, fallback adapter routing, public session/state id,
  cross-invocation cache, external adapter wire session, or unbounded prompt
  retention.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ref-contract`: strengthen outline/find producer refs from pass-through-only
  strings to readable refs on compatible document views while retaining opacity
  and adapter ownership.
- `adapter-contract`: add an invocation-private reusable document boundary and
  mandatory producer/read consistency responsibility without fixing method
  count or auxiliary extension shape.
- `docnav-architecture`: make the ref contract the semantic foundation,
  prepared state the reuse mechanism, and auxiliary facts/extensions secondary.
- `navigation-input-resolution`: preserve routing/input/fallback behavior while
  carrying one compatible adapter document through composition.
- `markdown-adapter`: prove heading/document-head/full refs round-trip through
  Markdown read on compatible views.
- `json-adapter`: prove base/direct-comment/tail refs round-trip through JSON
  read materialization on compatible views.

## Impact

- Planning edits are confined to this change directory.
- Expected implementation surfaces are `crates/shared/adapter-contracts`,
  `crates/shared/navigation`, `crates/adapters/markdown`,
  `crates/adapters/json`, and their conformance/integration evidence.
- Stable owner docs that must be synchronized during apply include
  `docs/ref-contract.md`, `docs/adapter-contract.md`, `docs/architecture.md`,
  `docs/navigation-input-resolution.md`, and the Markdown/JSON owners.
- Public request/result/ref shapes remain unchanged. The new observable promise
  is that a producer-emitted ref is readable on a compatible view; changed-view
  stale-ref behavior remains adapter-owned.
- `redesign-find-result-model`, `redesign-token-cost-estimation`,
  `add-json-readable-renderer`, `add-project-wide-find`, and the code adapter
  receive compatibility handoffs only and are not prerequisites.
