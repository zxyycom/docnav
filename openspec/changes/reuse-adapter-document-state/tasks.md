This checklist is the execution authority for
`reuse-adapter-document-state`. Planning evidence tasks 1.1–1.6 may proceed
now. Task 1.7 requires explicit architecture/product-owner approval; task 1.8
records that decision in every affected artifact; task 1.9 audits the refined
artifacts. Tasks 2.1–6.6 MUST NOT begin until tasks 1.7–1.9 are complete.

## 1. Decision packet and blocking gates

- [ ] 1.1 Audit Current declared and automatic selection, direct outline/read/find/info, full-read mode resolution/hooks/default fallback, and unique-ref nested read; record source-backed acquisition/decode/parse/cleanup counts for Markdown and JSON without changing production behavior.
- [ ] 1.2 Complete the six-candidate matrix for probe-returned opaque prepared state, candidate-scoped open/probe handle, operation-shaped invocation session, core-owned immutable document acquisition/bytes plus adapter-owned decode/parse/ref/source-region behavior, adapter/composition-local reuse, and Current independent operations. For every row, record automatic-discovery ownership, unsupported/invalid cleanup, direct dispatch, full-read match/miss/error/default fallback, nested-read success/fallback, snapshot, memory/lifetime, and linked/external/service consequences.
- [ ] 1.3 Produce a responsibility/lifetime table that separately names behavior owner, storage owner, and reachability controller for navigation composition, document acquisition/bytes, adapter-private preparation, refs, default UTF-8 fallback, and linked/service/external execution. Then produce a stage-by-stage source-view table for declared selection and automatic discovery, covering path replacement, in-place mutation, deletion, encoding change, parse-invalid replacement, and cancellation between probe, cost policy, base operation, content/facts hooks, and nested read.
- [ ] 1.4 Produce a cleanup/failure table that names when candidate/selected state becomes unreachable on unsupported probe, invalid probe, selection failure, resolution failure, threshold miss, hook error, adapter diagnostic, invalid base result, nested-read fallback, invalid composition, cancellation, and unwind; distinguish infallible RAII cleanup from any proposed fallible close.
- [ ] 1.5 Complete the JSON ownership gate: compare the proposed snapshot outcomes with Current `json-document-changed-after-probe` behavior and its deterministic TOCTOU evidence, identify the not-yet-archived JSON capability owner, and prohibit JSON normative or production edits in this change until that owner explicitly accepts a handoff.
- [ ] 1.6 Audit handoffs with `interactive-outline-selection`, `add-ast-grep-code-adapter`, and `enable-local-core-adapter-service-mode`; record only compatibility conditions, owner, and ordering, and do not implement or rebase those changes.
- [ ] 1.7 **Architecture/product-owner decision gate: tasks 1.8–6.6 are blocked until the user or designated owner explicitly approves the exact candidate or bounded combination; behavior, storage, and reachability owners; direct-operation count policy; snapshot/TOCTOU and ref-view model; automatic-discovery cleanup; full-read fallback source view; nested-read cleanup/fallback; memory/lifetime bound; and linked/external/service boundary. Agents, benchmarks, and reviewers cannot close this task by selecting a preferred abstraction.**
- [ ] 1.8 After explicit approval, append a numbered Decision with the selected mechanism and rejected generality; record the final responsibility/lifetime, source-view, cleanup/resource, compatibility, and handoff tables; close every answered Open Question; and replace or define every “approved invocation lifecycle” / “approved document view” placeholder in proposal, design, and affected deltas with complete lifecycle, snapshot, cleanup, diagnostic, fallback, and process-boundary rules before code.
- [ ] 1.9 **Blocking artifact audit: no implementation task may begin until this audit passes.** Verify proposal, design, specs, and tasks agree on status, purpose, owner boundaries, candidate selection, and gate sequence; capability IDs reuse the existing `docnav-architecture`, `adapter-contract`, `navigation-input-resolution`, and `markdown-adapter` owners; every changed Current requirement is present as a complete `MODIFIED` block; no artifact describes an unapproved mechanism as approved or directly implementable; only this change directory has been modified by proposal work; `## Open Questions` contains no unanswered or merely wording-hidden ambiguity; JSON and coordinated active changes remain gates/handoffs rather than implementation scope; and token-cost/find changes remain independent rather than prerequisites.

## 2. Approved contracts and owner documents

- [ ] 2.1 Read `docs/coding-style.md`, the approved Decision, and the four owner documents; map each approved rule to exactly one owner before modifying implementation.
- [ ] 2.2 Update `docs/architecture.md` with navigation-owned composition/lifetime, the approved document acquisition/storage owner, adapter-owned format semantics/private state, linked-process scope, and the approved service/external handoff; distinguish document acquisition from Current navigation configuration-source loading, and do not add a public session or cache contract.
- [ ] 2.3 Update `docs/adapter-contract.md` with the exact approved internal lifecycle boundary, unsupported-candidate cleanup, operation/full-read participation, and private-state non-leakage. Name a shared type only if the selected candidate requires one; otherwise record the mechanism-neutral behavior without inventing a shared data structure.
- [ ] 2.4 Update `docs/navigation-input-resolution.md` with the approved stage order, automatic-discovery promotion/drop rules, direct dispatch, full-read match/miss/error/default fallback source view, nested-read lifecycle, snapshot/TOCTOU, ref-view relation, and diagnostic/fallback behavior.
- [ ] 2.5 Update `docs/adapters/markdown.md` with the approved source view, Markdown-private decode/parse/ref reuse, and snapshot semantics while preserving every existing Markdown result, ref, pagination, and error owner rule.
- [ ] 2.6 Confirm `docs/protocol.md`, `docs/output.md`, `docs/ref-contract.md`, schemas, examples, CLI, and parameter catalog require no shape changes; if implementation would change one, stop and return to artifact review instead of expanding this change.

## 3. Current and failing evidence

- [ ] 3.1 Before adding or changing tests, follow `docs/testing.md`, each behavior owner, `docs/testing/case-maintenance.md`, and the `test-evidence-review` skill; run the project wrapper that proves the complete current tree's static entities, runner entities, and Case mappings are closed.
- [ ] 3.2 Add test-only instrumentation that counts source acquisition, decode, complete parse/model construction, reuse, and cleanup without adding protocol/output/log fields or changing release behavior.
- [ ] 3.3 Add current/failing evidence for declared and automatic direct outline/read/find/info showing the approved selected-candidate count target and bounded destruction of every unsupported/invalid candidate.
- [ ] 3.4 Add current/failing evidence for full-read threshold match, miss, cost-measurement error, content/facts error, structured fallback, and navigation-owned default UTF-8 fallback, including the approved source view and preparation counts.
- [ ] 3.5 Add current/failing evidence for unique-ref auto-read success, adapter diagnostic, invalid nested result, and invalid composed response, proving the base response fallback remains byte-for-byte/semantically compatible and private state is released.
- [ ] 3.6 Add deterministic mutation evidence for every approved snapshot outcome, including the Current JSON post-probe replacement case as a gate/handoff rather than modifying the unarchived JSON owner from this change.
- [ ] 3.7 Add compile-time/runtime and serialization evidence that parser types, state IDs, handles, snapshot metadata, cleanup facts, and nested failure details do not enter closed operation input, protocol JSON, readable output, refs, continuations, invocation logs, schemas, or examples.

## 4. Approved linked-adapter implementation

- [ ] 4.1 Implement the smallest approved invocation-private lifecycle mechanics in the owner modules named by the approved Decision. Modify `docnav-adapter-contracts` representation only if the selected candidate requires it; for an approved local candidate, record why no shared lifecycle/type is needed. Retain fixed outline/read/find/info semantics and full-read capability ownership, and do not add combination methods, a generic tree/node API, downcasts unless explicitly approved, or a caller-visible state lookup.
- [ ] 4.2 Implement the approved navigation-owned candidate lifecycle for declared selection and automatic discovery, including creation/promotion/destruction only where the selected mechanism requires them, while preserving registry order, first-supported selection, candidate evidence, explicit-selection diagnostics, and the approved cleanup/failure bound.
- [ ] 4.3 Make the approved lifecycle available across input resolution and direct operation dispatch through the selected mechanism without putting private state into `StandardOperationInput`, `RequestEnvelope`, parameter resolution, or output projection.
- [ ] 4.4 Apply the approved lifecycle to full-read cost selection, threshold miss, content/facts hooks, structured outline fallback, and default UTF-8 fallback; prove no branch silently reacquires/reparses the same approved view outside an explicit bounded refresh rule.
- [ ] 4.5 Carry the same selected lifecycle through Current unique-ref auto-read, preserving opaque ref pass-through, read page `1`, existing read arguments, nested-result validation, composed validation, and silent validated-base fallback.
- [ ] 4.6 Align Markdown probe, direct operations, full-read hooks, and nested read with the approved source view and Markdown-private decode/parse/ref representation; preserve all existing Markdown ref grammar, structure-snapshot meaning, result facts, cost, pagination, and diagnostics.
- [ ] 4.7 Add no cross-invocation cache or unbounded interactive retention; release selected state immediately after the final eligible stage, validation fallback, error, or cancellation under the approved policy.

## 5. Cross-change gates and handoffs

These tasks produce compatibility handoffs only. They do not make the changes
dependencies, authorize edits to their directories, or merge their
implementation scope into this change.

- [ ] 5.1 Leave JSON production/spec changes unimplemented while `add-json-adapter` remains unarchived; deliver the count/snapshot/TOCTOU handoff packet to that capability owner and require explicit acceptance before any later JSON reuse change.
- [ ] 5.2 Deliver a bounded-lifetime handoff to `interactive-outline-selection` that answers whether a prompt separates invocations; do not retain adapter state across an unbounded user wait or edit that change here.
- [ ] 5.3 Deliver parser-lifetime and private-model constraints to `add-ast-grep-code-adapter`; do not force borrowed ast-grep state onto an unapproved generic interface or edit that change here.
- [ ] 5.4 Deliver linked-invocation-only constraints to `enable-local-core-adapter-service-mode`; forbid cross-request parser caches, public session IDs, or an internal wire format under this change, and do not implement service mode here.

## 6. Verification and closure

- [ ] 6.1 Run targeted adapter-contract, navigation selection/outline-mode/auto-read, Markdown unit/integration, core CLI, invocation-log, and smoke tests for every changed branch and cleanup path.
- [ ] 6.2 Run instrumented large-document tests or benchmarks that report baseline versus final acquisition/decode/parse counts, peak retained bytes/model lifetime, and cleanup counts for direct, full-read, and nested-read paths; interface construction alone is not acceptance evidence.
- [ ] 6.3 Run deterministic approved TOCTOU cases and confirm each result/diagnostic matches the owner-approved source-view table; record platform limitations without treating missing mutation evidence as a pass.
- [ ] 6.4 Validate that protocol/readable outputs, refs, continuations, schemas, examples, adapter selection order, auto-read eligibility, and fallback results remain compatible and contain no private lifecycle facts.
- [ ] 6.5 Run `dnm outline`/targeted reads for changed owner docs, strict OpenSpec validation, formatting/static checks, and `bun run verify:docnav-workspace`; run release-package validation because the change crosses shared navigation, adapter contract, and adapter boundaries.
- [ ] 6.6 Review the final diff against the approved candidate and rejected generality, confirm coordinated changes were not edited and token-cost/find work did not enter scope, obtain final architecture/product-owner acceptance, and only then mark the change ready for archive assessment.
