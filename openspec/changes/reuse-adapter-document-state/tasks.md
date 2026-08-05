This checklist is the execution authority for
`reuse-adapter-document-state`. The architecture decision is approved: ref
producer/read consistency is foundational, prepared state supplies one reusable
view, and method count/shape is non-normative. Tasks 2.1 onward MUST follow
Decisions 1-7.

## 1. Ref-contract decision and artifact gate

- [x] 1.1 Rebase the change on Current no-probe pathname routing and verify selection performs zero target-document acquisition, decode, parse, or prepared-state creation.
- [x] 1.2 Recover Markdown/JSON preparation, outline/find ref production, read resolution/materialization, auxiliary facts, composition, cleanup, and Current preparation counts from owner docs and source.
- [x] 1.3 Replace the lifecycle-first model with an adapter-document/ref model and add Markdown plus JSON as heterogeneous evidence.
- [x] 1.4 **Architecture/product-owner gate:** confirm one captured invocation view and the prepared-document direction; confirm that algorithm count/callable shape are non-normative and producer/read consistency requires explicit contract laws rather than being implied by shared state.
- [x] 1.5 Record Decision 7; define ref producer/consumer/auxiliary extension, compatible document view, canonicality, round trip, no-hidden-context, correspondence, multiplicity, incompatible-view behavior, and mandatory conformance evidence without fixing the number or shape of adapter algorithms.
- [x] 1.6 **Blocking artifact audit:** verify proposal, design, specs, and tasks agree on Decisions 1-7, capability ids, compatible-view scope, Markdown/JSON proof, public compatibility, and apply order; every `MODIFIED` requirement is a complete then-Current body plus approved delta; no algorithm-count constraint, public state leak, cross-invocation cache, generic node/state registry, routing probe, runtime double-read, or unapproved ownership transfer remains; strict OpenSpec validation passes.

## 2. Approved owner contracts

- [ ] 2.1 Read `docs/coding-style.md`, Decisions 1-7, `docs/architecture.md`, `docs/ref-contract.md`, `docs/adapter-contract.md`, `docs/navigation-input-resolution.md`, and the Markdown/JSON owners. Map each approved rule to one durable owner before implementation.
- [ ] 2.2 Update `docs/ref-contract.md` so producer-emitted refs MUST round-trip through read on the same or independently prepared compatible view; retain opaque pass-through, adapter grammar ownership, multiplicity freedom, and incompatible-view stale behavior.
- [ ] 2.3 Update `docs/architecture.md` with the ref-law foundation, adapter-document/private-state ownership, navigation composition/reachability, extension subordination, linked-process scope, and non-public state boundary.
- [ ] 2.4 Update `docs/adapter-contract.md` with ref producer/consumer responsibilities, compatible-view law, preparation/reuse boundary, closed input separation, auxiliary extension rule, bounded results, and release/non-leakage constraints without prescribing method count.
- [ ] 2.5 Update `docs/navigation-input-resolution.md` with document-boundary creation after final selection/path/input resolution, Current validation-versus-access ordering, one-view composition, nested-read law, mutation scope, and existing fallback behavior.
- [ ] 2.6 Update `docs/adapters/markdown.md` and `docs/adapters/json.md` with same-state/fresh-compatible round trips, owner-specific correspondence, private preparation reuse, mutation boundaries, and unchanged result/ref/diagnostic semantics.
- [ ] 2.7 Confirm protocol, output, CLI/config, schema/example, continuation, and decision-record owners require no public shape or lifecycle-record change. If implementation needs one, stop and return to OpenSpec review.

## 3. Current and failing evidence

- [ ] 3.1 Before changing tests, follow `docs/testing.md`, each affected behavior owner, `docs/testing/case-maintenance.md`, and the `test-evidence-review` skill; run the project wrapper proving complete Current static, runner, and Case mappings are closed.
- [ ] 3.2 Add test-only instrumentation for acquisition, decode, complete parse/model construction, reuse, ref production, read resolution, auxiliary extension access, and drop. Do not add release logging, protocol fields, globals, or timing-dependent assertions.
- [ ] 3.3 Add a shared black-box ref conformance harness that accepts opaque producer results and invokes read without parsing or reconstructing refs.
- [ ] 3.4 For representative Markdown/JSON fixtures, traverse outline/find to terminal pages, collect every emitted full ref, and add failing then passing same-state page-1 read assertions.
- [ ] 3.5 Independently prepare identical fixtures with identical relevant facts and add failing then passing fresh-compatible read assertions for every collected ref; prove no hidden producer call order, pointer, or producer-only option is required.
- [ ] 3.6 Add owner-specific correspondence assertions for Markdown sections/head/full refs and JSON base/direct-comment/tail refs, including cases where JSON read content need not literally contain find punctuation/whitespace evidence.
- [ ] 3.7 Cover repeated find refs, multiple refs selecting one region, empty/fallback structures, virtual/comment entries, visibility options, long refs with truncated labels, and terminal pagination.
- [ ] 3.8 Prove routing performs zero preparation; direct paths perform at most one; cost/full-read/structured fallback and unique-ref nested read reuse one view where selected-adapter state participates; preserve pre-document diagnostic ordering.
- [ ] 3.9 Add deterministic replacement, in-place mutation, deletion, repair, encoding-change, parse-invalid replacement, and relevant-configuration cases that separate compatible from incompatible views.
- [ ] 3.10 Add compile-time/runtime and serialization evidence that state handles, parser/source types, snapshot/cleanup facts, conformance-only facts, and nested failures do not enter closed caller input, raw/readable output, refs, continuations, schemas, examples, or logs.

## 4. Adapter-document implementation

- [ ] 4.1 Implement the smallest `AdapterDefinition -> AdapterDocument -> private prepared state` boundary. Do not standardize algorithm count, generic nodes/trees, downcasts, arbitrary lookup, operation combinations, caller-visible handles, or cross-invocation caches.
- [ ] 4.2 Create the adapter document only after final no-probe selection, filesystem-backed path/access normalization, and approved input resolution. Initialize private state at the first Current-compatible document access and preserve adapter semantic-validation ordering.
- [ ] 4.3 Keep operation arguments closed and typed. The existing read input plus opaque ref and compatible view MUST suffice; no hidden producer-only context or generic state parameter may be required.
- [ ] 4.4 Migrate Markdown preparation and current outward operations through one private document state; make every emitted heading/document-head/full ref pass the conformance harness.
- [ ] 4.5 Migrate JSON preparation and current outward operations through one private document state; make every emitted base/direct-comment/tail ref pass the harness without reducing read to a source range.
- [ ] 4.6 Route direct operations, selected-adapter full-read policy/fallback stages, and unique-ref nested read through the reusable boundary while preserving independently owned extension/result behavior.
- [ ] 4.7 Release private state immediately after the last eligible stage, validation fallback, error, cancellation, or unwind. Add no public cleanup operation or cross-page/invocation retention.

## 5. Cross-change handoffs

These tasks deliver compatibility statements only; they do not authorize edits
to other active change directories or make those changes prerequisites.

- [ ] 5.1 Require `redesign-find-result-model` to preserve the compatible-view read law for every ref emitted by its approved occurrence/distinct/grouped unit; do not constrain its private find method shape here.
- [ ] 5.2 Require `add-project-wide-find` to apply the ref law to every project-find ref and to preserve path/adapter facts needed to identify a compatible view.
- [ ] 5.3 Require `add-ast-grep-code-adapter` to pass the same harness for symbol refs without imposing a raw-source-range or generic AST API.
- [ ] 5.4 Tell token-cost and JSON readable-renderer owners that auxiliary facts/presentation cannot reinterpret ref identity or weaken producer/read correspondence.
- [ ] 5.5 Tell interactive selection and local service mode that no adapter state crosses an unbounded prompt or request; retention requires a separate lifecycle decision.

## 6. Verification and closure

- [ ] 6.1 Run targeted adapter-contract/ref-contract, navigation routing/outline-mode/auto-read, Markdown, JSON, core CLI, invocation-log, and smoke tests for changed success, failure, fallback, and cleanup branches.
- [ ] 6.2 Report baseline versus final preparation counts, conformance coverage, peak retained bytes/model lifetime, and drop counts. Interface construction alone is not acceptance evidence.
- [ ] 6.3 Run every compatible/incompatible-view mutation case and confirm outcomes match Decisions 3 and Laws 3/7.
- [ ] 6.4 Validate raw/readable outputs, refs, pagination/continuation, schemas, examples, selection order, auto-read eligibility, auxiliary facts, and fallback results remain compatible and contain no private state.
- [ ] 6.5 Run `dnm outline`/targeted reads for changed owner docs, strict OpenSpec validation, scope-matched formatting/static checks, `bun run verify:docnav-workspace`, and release-package validation.
- [ ] 6.6 Review the final diff against Decisions 1-7 and the ref laws, obtain final architecture/product-owner acceptance of the implementation evidence, and only then mark the change ready for archive assessment.
