**Planning state: artifact-ready / implementation-blocked.** Section 1 resolves the human decision gate; tasks 2.1–7.4 MUST NOT start until task 1.6 records every approved Q1–Q7 answer and closes that gate. A later validation failure governed by design Q2–Q4 reopens the affected gate tasks and blocks dependent work again.

## 1. Resolve the implementation decision gate

- [ ] 1.1 Audit the proposal, design, four capability deltas, and tasks as one artifact set. Confirm the proposal capabilities exactly match spec directories, every changed Current requirement is copied in full under `MODIFIED`, Current and Target claims are distinguished, Q1–Q7 contain every unresolved product/architecture choice, owner and cross-change relationships are explicit without prerequisites, and no artifact claims implementation approval.
- [ ] 1.2 Record a reproducible Current release baseline, then compare candidate machine encodings and calculator classes on the same representative Markdown, JSON, code, English, CJK, mixed-language, emoji/combining, escape, whitespace-run, long-piece/scalar, and large state/configuration corpus. Report candidate-specific error distribution and under/over-estimation behavior plus worst-case CPU, peak RSS, cold start, platform/target behavior, and package impact; identify build profile, command, fixture, output mode, page/limit, host/cache assumptions, and measurement noise. Treat the current `bpe-openai` worktree experiment only as an independently reproducible candidate, not as the Current baseline or approval.
- [ ] 1.3 For every proposed new or replacement production dependency, complete a source-backed review of ecosystem adoption, maintainers and release cadence, known security issues/advisories, license compatibility, MSRV/targets, transitive graph, native/build requirements, package impact, worst cases, and viable existing/dependency-free alternatives.
- [ ] 1.4 Obtain and record explicit human approval for Q1–Q4 and Q7: machine representation and scope, calibration/error criteria, CPU/peak-RSS/cold-start/platform/package/per-entry/page budgets, selected calculator and dependency boundary, and structured-outline admission/accounting. The admission rule MUST establish returned-page membership without estimating and later discarding a non-returned entry. Benchmark results or an agent recommendation alone MUST NOT complete this task. When validation reopens Q2–Q4, invalidate the affected prior approval and obtain a new approval from the updated evidence.
- [ ] 1.5 Obtain and record explicit human approval for Q5–Q6: the token-valued unstructured-full-read threshold contract and existing-consumer compatibility/migration. If either answer changes another owner, add that existing capability to the proposal and create its complete delta before implementation; do not hide a `navigation-input-resolution` or `adapter-contract` change in code.
- [ ] 1.6 After tasks 1.1–1.5 are complete, move every approved Q1–Q7 answer from `## Open Questions` into consecutively numbered `## Decisions`, update proposal/design/specs/tasks consistently, rerun strict OpenSpec validation, and record the change as implementation-unblocked. This task is the only gate-closing step. If validation later reopens Q2–Q4, return this task to incomplete until replacement approval and synchronization close the gate again.

## 2. Contract-first owner synchronization

- [ ] 2.1 Update `docs/architecture.md` and the shared text-cost owner contract with the approved approximate-token mechanics, inputs, budgets, and dependency boundary; keep text selection, measurement scope, pagination, adapter semantics, and presentation with their existing owners.
- [ ] 2.2 Update `docs/protocol.md` with the approved machine encoding and exact returned-content/current-page-selection meanings for ordinary read, nested read, unstructured full-read, structured outline, and find non-target measurement.
- [ ] 2.3 Update `docs/output.md` with the built-in readable approximation and scope notation, including mixed-scope measurement behavior and the rule that rendering never calculates missing cost or issues another operation.
- [ ] 2.4 Update `docs/adapters/markdown.md` with returned-page read cost, current-page cheap outline selection estimates, unstructured returned-content cost, find target non-measurement, and unchanged ref/region/character-pagination semantics.
- [ ] 2.5 Update every affected protocol schema and contract example to the approved machine shape. Validate ordinary/nonterminal read, nested read, structured outline, unstructured outline, and find-without-target-cost examples before production code.
- [ ] 2.6 Record the accepted compatibility version and the Decision 7 handoff conditions for `add-json-adapter`, `add-json-readable-renderer`, `interactive-outline-selection`, `add-outline-preview-skim-pack`, `implement-docnav-mcp-bridge`, and relevant adapter changes. Preserve the independent status of `redesign-find-result-model` and `reuse-adapter-document-state`; do not implement, rebase, or edit another owning change from this task.

## 3. Test and benchmark evidence

- [ ] 3.1 Before changing tests, follow `docs/testing.md`, each behavior owner, `docs/testing/case-maintenance.md`, and the `test-evidence-review` skill; run the project wrapper that proves the complete current tree's static entities, runner entities, and Case mappings are closed.
- [ ] 3.2 Add or update protocol/schema/example evidence for machine-identifiable approximation, returned-content versus visible-selection scope, nested-read parity, and absence of implied find target cost.
- [ ] 3.3 Add Markdown evidence for nonterminal and terminal read pages, `doc:full`, `HEAD:leading`, nested read, unstructured full-read, current-page outline entries, entries beyond the page, and one very large visible section without complete-section tokenization.
- [ ] 3.4 Add estimator correctness evidence for the approved multilingual/adversarial corpus, including empty text, Unicode, combining sequences, whitespace runs, long pieces/scalars, escapes, and boundary-sized returned pages.
- [ ] 3.5 Add reproducible CPU, peak-RSS, cold-start, platform/package, and worst-case benchmark checks that enforce the approved budgets without embedding an unapproved dependency or treating local worktree state as baseline.

## 4. Approved shared estimation mechanics

- [ ] 4.1 Implement the approved returned-text estimator in the smallest existing shared text-cost boundary, using only the approved dependency set and machine semantics.
- [ ] 4.2 Implement the approved cheap-fact selection estimator only if the approved design requires a distinct mechanic; keep it free of document paths, refs, parser trees, adapter sessions, serialization, pagination, and output policy.
- [ ] 4.3 Preserve existing line/byte measurement behavior unless the approved contract explicitly changes it, and attach token scope without conflating exact selection facts with returned-page estimates.
- [ ] 4.4 Prove helpers stay within approved error and resource budgets on ordinary and adversarial inputs. On failure, execute the design's gate-reopening transition: block dependent apply work, invalidate and reopen the affected Q2–Q4 approvals, return to task 1.2 and task 1.3 when dependency evidence is affected, and require tasks 1.4 and 1.6 again rather than relaxing acceptance criteria.

## 5. Protocol, Markdown, and readable integration

- [ ] 5.1 Implement the approved protocol model, validation, and serialization for approximate-token facts while preserving operation/result pairing, closed auto-read shape, and existing page fields.
- [ ] 5.2 Change Markdown ordinary and nested read so token estimation receives only the content already selected for the current character-bounded page, never the unreturned section remainder.
- [ ] 5.3 Change Markdown structured outline so only current-page entries receive cheap selection estimates and one visible large section cannot trigger complete serialization, materialization, or tokenization.
- [ ] 5.4 Align Markdown unstructured full-read with returned-content estimates and ensure Markdown find does not read or measure a referenced target solely for cost.
- [ ] 5.5 Update the built-in readable renderer to identify approximation and scope from the immutable protocol response without calculating missing values, issuing another operation, or mislabeling returned-page cost.

## 6. End-to-end and resource verification

- [ ] 6.1 Validate raw/readable/schema/example parity for ordinary read, nested read, structured outline, unstructured full-read, and find; confirm required token cost has no disable switch.
- [ ] 6.2 Run real CLI pagination round trips and prove character boundaries/page numbering are unchanged while every read page reports only its own returned-content estimate.
- [ ] 6.3 Use instrumentation or equivalent evidence to prove structured outline performs estimate work only for current-page entries and does not fully serialize/tokenize a large visible target; prove find does no ref-triggered target estimate.
- [ ] 6.4 Re-run the approved accuracy and CPU/RSS/cold-start/platform/package budget suite against release-mode artifacts, then run `bun run verify:docnav-workspace` and scope-appropriate release-package verification. Any Q2–Q4 or dependency failure executes the design's gate-reopening transition and blocks handoff work until task 1.6 closes the replacement decision.

## 7. Handoff and closure

- [ ] 7.1 Deliver the accepted contract/version/handoff notes to each adjacent change owner without implementing or rebasing JSON readable rendering, interactive outline selection, skim, MCP, or other adapter work here.
- [ ] 7.2 Confirm the eventual archived `json-adapter` owner has a blocked handoff for returned-content/current-page token behavior; do not create a synonymous capability delta while it is absent from main specs.
- [ ] 7.3 Run `ai-ready-docs` over changed owner and OpenSpec material and perform the coding-style self-review over implementation diffs; confirm authority, state labels, no unnecessary abstraction, no unapproved dependency, and no edits to adjacent changes.
- [ ] 7.4 Run final strict OpenSpec, workspace, schema/example, corpus-budget, and release verification. A Q2–Q4 or dependency failure reopens the gate and invalidates the affected approval; archive only after replacement evidence, human approval, task 1.6 synchronization, all implementation evidence, and accepted handoffs are complete.
