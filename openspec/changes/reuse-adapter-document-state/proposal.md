This proposal defines the mechanism-neutral Target for
`reuse-adapter-document-state`. It does not select a session, handle, shared
document type, source snapshot, or concrete Rust ownership shape.

**Planning status: artifact-ready / implementation-blocked.** OpenSpec artifact
completion means the required planning files exist; it does not prove Current
behavior or authorize apply. Stable behavior remains owned by `docs/`, while
code, tests, and release artifacts prove the current implementation. This
change owns only its Target plan, later implementation tasks, and audit history.

Use this proposal as the active-change entry point. Read `design.md` for
Current/Target interpretation, the six unselected candidates, responsibility
boundaries, and open decisions; read `specs/` for the mechanism-neutral Target
deltas; then follow `tasks.md`, which exclusively owns the 1.1–6.6 execution
order and the 1.7–1.9 implementation gate.

## Why

Current linked adapters independently acquire and decode the document during probe, parse it there when format detection requires parsing, and then perform complete preparation again for each operation or optional full-read hook. Navigation-owned composition can therefore multiply whole-document work even when the caller receives one bounded result; JSON and future code adapters make that cost and the associated source-snapshot ambiguity materially larger than the current Markdown-focused path suggests.

## What Changes

- Define a mechanism-neutral same-invocation outcome: navigation continues to own composition and invocation lifecycle, while the approved mechanism lets the selected adapter reuse a compatible document source view and adapter-private decoded/parser/index/source-region/ref facts so composition alone does not repeat complete acquisition, decode, or parse of that view.
- Require the architecture decision to compare six distinct candidates before implementation: probe plus opaque prepared state, candidate-scoped open/probe handle, operation-shaped invocation session, core-owned immutable document acquisition/bytes plus adapter-owned decode/parse/ref/source-region behavior, adapter/composition-local reuse, and the Current independent-operation model.
- Require the comparison to cover declared selection and automatic discovery, disposal of unsupported/invalid candidates, direct operations, cost-selection plus unstructured full-read hooks, unique-ref nested read, failure cleanup, and linked versus external/service execution boundaries.
- Make source snapshot and TOCTOU semantics an explicit product/architecture decision. In particular, the owner must decide whether probe, base operation, full-read selection/content, and nested read observe one invocation snapshot or permitted later views, and must deliberately reconcile the Current JSON post-probe reload diagnostic.
- Preserve adapter ownership of format detection, decode/parse semantics, private tree/index/source regions, ref generation/resolution, and operation algorithms. Reusable state remains process-private and MUST NOT enter protocol, output, ref strings, logs, continuation values, or caller-visible IDs.
- Put an explicit human architecture/product-owner gate before normative mechanism refinement or implementation. The gate selects the lifecycle, snapshot, cleanup/fallback, and process-boundary model; an agent, benchmark, or artifact review cannot close it.
- Coordinate non-dependency handoffs with `interactive-outline-selection`, `add-ast-grep-code-adapter`, `enable-local-core-adapter-service-mode`, and the eventual archived JSON capability without implementing or rebasing those changes here.

## Non-Goals

- No request-scoped session is selected by this proposal, and no generic document object, operation-combination method matrix, parser-tree DTO, downcast contract, public state ID, or cross-invocation cache is introduced.
- No change to ref grammar or opacity, protocol request/response shape, output modes, continuation semantics, adapter selection order, or Current auto-read eligibility/result fallback.
- No selection of token-cost producer/sink mechanics, calculator, dependency, find result model, find pagination, or scan budget. `redesign-token-cost-estimation` and `redesign-find-result-model` remain independent changes and are not prerequisites.
- No external adapter runtime or service wire protocol. Local service mode may coordinate with an approved linked-adapter lifecycle but does not gain ownership of adapter-private state through this change.
- `explore-operation-composition` is historical context for Current auto-read, not a dependency, owner, or implementation prerequisite for this change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `docnav-architecture`: Clarify navigation-owned composition/lifecycle versus adapter-owned format state, leave document acquisition/storage ownership to the explicit gate, and preserve the linked/process-private boundary.
- `adapter-contract`: Require a mechanism-neutral private reuse boundary and bounded cleanup without exposing parser state through public or serialized contracts.
- `navigation-input-resolution`: Require direct dispatch, full-read policy, and nested-read composition to participate in one approved invocation lifecycle while preserving selection and fallback semantics.
- `markdown-adapter`: Require eligible same-invocation Markdown paths not to repeat complete acquisition/decode/parse work while retaining Markdown-owned ref, snapshot, and operation semantics.

## Impact

- Potential implementation surfaces only after tasks 1.7–1.9: `crates/shared/navigation`, `crates/shared/adapter-contracts`, `crates/adapters/markdown`, and core integration tests that instrument acquisition/decode/parse and cleanup. The not-yet-archived JSON capability is evidence and a handoff gate, not an implementation surface for this change.
- Coordination surfaces: automatic adapter discovery, unstructured full-read cost/content hooks, unique-ref auto-read, interactive outline-to-read orchestration, code-adapter parser lifetimes, and local core service mode.
- Public compatibility: no planned protocol, schema, example, ref, output, CLI, parameter, or continuation shape change. Any proposal that requires such a change must return to artifact review rather than entering implementation under this scope.
- Resource behavior: the approved design must prove reduced complete preparation counts without introducing cross-invocation retention, unbounded memory, leaked candidate state, or hidden fallback re-parsing.
