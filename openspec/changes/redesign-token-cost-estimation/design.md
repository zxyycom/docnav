**Planning state: artifact-ready / implementation-blocked.** This design fixes the scope and work bounds of AI-facing token estimates while leaving their machine encoding, calculator, budgets, and dependency subject to the explicit human decision gate below.

## Context

Docnav's finite navigation flow is useful only when the work needed to describe a bounded result is itself bounded. When this change was created, owner material marked `docnav-text-cost::token_cost` `o200k_base` counting semantics as Current, and Markdown read reported token cost for the complete selected section even when it returned only one character-bounded page. Structured outline entries similarly reported complete section cost. On a large structured or state/configuration document, exact tokenization or hidden serialization can therefore dominate the result the AI actually receives. Because this change is not Current implementation evidence, task 1.2 must verify that baseline against a reproducible release before approval.

Unique-ref auto-read is already Current navigation behavior: it may add an existing `ReadResult` to one outline/find response. `explore-operation-composition` is the historical exploration from which that behavior evolved; it is not a dependency, owner, or prerequisite for this token-cost change. Nested read must obey the same returned-content estimate contract as ordinary read.

The worktree's `bpe-openai` experiment is not a Current baseline, selected calculator, or approved dependency. It can contribute reproducible evidence only under the same comparison criteria as other candidates.

This change crosses shared text-cost mechanics, raw protocol facts, readable projection, and format-owned cost attachment. Stable rules ultimately belong in their owner docs and schema/examples; these temporary artifacts plan the change and do not prove current implementation.

## Contract Vocabulary

- **Approximate-token fact** is the conceptual machine fact required by this
  change. Q1 owns its eventual field/unit shape and compatibility encoding; the
  term does not preselect that shape.
- **Returned-content estimate** describes only text present in the current
  result, including the current read page or unstructured outline content.
- **Visible-selection estimate** describes the readable content behind a
  structured-outline entry already admitted to the current returned page. It is
  derived from approved cheap facts or another approved bounded input, not from
  a hidden full read.
- **Current page** means page membership established by the existing
  character-based pagination contract before entry-specific token estimation
  runs.

## Goals / Non-Goals

**Goals:**

- Keep token cost visible to AI callers without requiring exact BPE parity.
- Bound token-estimation work to returned content, except for cheap estimates describing selections represented by entries on the current structured-outline page.
- Make approximation and measured scope honest in machine and readable output.
- Preserve existing character pagination and continuation behavior.
- Prevent a ref or a visible large entry from authorizing hidden full target serialization or tokenization.
- Choose and record the machine representation, calibration contract, budgets, estimator, and dependency boundary only after explicit evidence review and human approval.

**Non-Goals:**

- Treating planning-artifact cleanup, benchmark output alone, or an agent recommendation as selection or approval of the machine encoding, calculator, coefficients, reference tokenizer, production dependency, or numeric budgets before the gate closes.
- Adding a token-cost disable switch or tokenizer selector.
- Using token estimates as a pagination or continuation budget.
- Redesigning find results, auto-read eligibility, operation composition, parser-state reuse, refs, or format parsing.
- Implementing JSON readable rendering, outline skim/selection, MCP transport, or another active adapter change.

## Implementation Decision Gate

The OpenSpec artifact graph is complete, but implementation is not approved.
Tasks 2.1–7.4 MUST NOT start until task 1.6 closes this gate.

The gate closes only when:

1. task 1.1 confirms artifact/capability integrity and that every unresolved
   product or architecture choice is represented by Q1–Q7;
2. tasks 1.2–1.3 record reproducible baseline, candidate, resource, and
   dependency evidence;
3. the user or designated product/architecture owner explicitly approves Q1–Q7
   through tasks 1.4–1.5; and
4. task 1.6 moves every approved answer into `## Decisions`, expands the
   capability set if an answer changes another owner, synchronizes all
   artifacts, and passes strict OpenSpec validation.

Benchmark results and agent recommendations are evidence only. They cannot
close a question, select a calculator or dependency, or change this state to
implementation-unblocked.

### Gate reopening after failed validation

Gate closure is conditional on the evidence that supported Q2–Q4. If task 4.4,
6.4, 7.4, or equivalent later validation shows that the approved calculator
misses calibration/error criteria or resource budgets, or that its dependency
boundary fails the approved maintenance, security, license, MSRV/target,
transitive, build, or package criteria, the following state transition is
mandatory:

1. Stop dependent apply, handoff, and archive work and set the change back to
   **implementation-blocked**.
2. Mark each affected Q2–Q4 Decision and its approval as invalidated by the new
   evidence; restore the affected question to `## Open Questions` with the
   failing evidence. A calculator failure reopens Q4 plus every Q2/Q3 premise it
   violated. A dependency failure reopens the dependency part of Q4 and any Q3
   budget it invalidated.
3. Reopen task 1.2, task 1.3 when dependency evidence is affected, task 1.4, and
   task 1.6. Also reopen every completed downstream task whose result or
   acceptance evidence depended on an invalidated Decision. The invalidated
   approval and dependent evidence cannot be reused.
4. Repeat the affected comparison or dependency review and replace the failed
   evidence.
5. Obtain new explicit human approval, record the replacement Decisions,
   synchronize the standard artifacts, and pass strict OpenSpec validation
   through task 1.6 before dependent implementation resumes.

Validation criteria MUST NOT be relaxed merely to preserve an earlier choice.
Only the same evidence → approval → synchronization sequence can close a
reopened gate.

## Decisions

### Decision 1: Token cost remains required but becomes an explicit estimate

**Status: product direction confirmed; encoding and calculator are open.**

AI callers need token cost to decide whether a result or selectable region is worth reading. Public token cost therefore remains required on the confirmed surfaces; this change does not solve overhead by omitting the fact or adding a disable switch.

The value is an estimate, not a promise of parity with `o200k_base`, another BPE vocabulary, or any specific current/future model. The eventual machine encoding and readable notation must make that approximation observable rather than preserving an exact-looking integer with undocumented changed semantics.

Retaining exact BPE parity does not meet the overhead goal. Omitting the fact or making it optional would remove or inconsistently provide the navigation signal, so neither is the target contract.

### Decision 2: Each public surface has one bounded token meaning

**Status: scope confirmed.**

| Surface | Required token meaning | Work boundary |
| --- | --- | --- |
| ordinary `ReadResult` | returned-content estimate for this page's `content` | calculator sees only returned content |
| `AutoReadResult.read` | returned-content estimate with ordinary-read semantics | nested read does not measure the unreturned remainder |
| unstructured `OutlineResult` | returned-content estimate for its `content` | content is already selected for return |
| structured outline entry | visible-selection estimate | only current-page entries; cheap existing span/facts or an approved bounded calculation |
| find item without nested read | no implied target-content estimate | ref does not authorize target read/serialization/tokenization |

If a find item later carries cost for its own returned label/excerpt payload, that fact must be scoped to the returned payload and cannot be interpreted as target cost. A composed read carries its own returned-content estimate.

Line and byte measurements are not automatically redefined by this decision. The approved machine encoding must allow consumers to distinguish measurement units and scopes when a result carries exact selection facts alongside a returned-page token estimate.

### Decision 3: Structured outline uses visible-selection estimates without hidden rendering

**Status: work boundary confirmed; estimator inputs and encoding are open.**

Structured outline is intentionally different from read. An AI benefits from a rough estimate of the content behind each visible ref before selecting it, but the estimate is enrichment of the current returned entry page—not permission to build the content.

Eligible inputs are cheap facts already produced while parsing or selecting the entry, such as source byte/character span length, line count, bounded samples, or equivalent adapter-owned facts. Any bounded scan must be part of the approved calculator and per-entry/page budgets. The implementation must not:

- serialize a complete structured subtree or Markdown section solely for cost;
- tokenize the complete target solely for cost;
- traverse entries outside the current returned outline page;
- treat one unusually large visible entry as an exemption from these limits.

If an adapter cannot obtain an approved estimate from cheap facts, that is a contract/encoding decision to resolve before implementation; it is not permission to omit required cost silently or perform hidden full work.

Page assembly must not estimate a candidate and then discard it as outside the returned page. The approved encoding and pagination design therefore need a bounded admission rule—such as reserving a known maximum representation size or another owner-approved method—that establishes current-page membership before entry-specific estimation while still honoring the existing character budget. Q7 owns that unresolved rule.

### Decision 4: Character budgets continue to paginate

**Status: confirmed.**

Existing character limits and page continuation determine the returned content. Token estimation runs after the returned text/page is selected, or consumes cheap current-page selection facts for structured outline. It does not choose a cut point, preload later pages, or alter page numbering.

This preserves deterministic continuation and avoids circular behavior where calculating the budget itself requires the expensive tokenizer this change is removing.

### Decision 5: The shared helper owns mechanics, not selection or presentation

**Status: ownership confirmed; helper API and algorithm are open.**

The shared text-cost boundary may provide the approved estimator mechanics. Callers continue to own:

- which returned text or cheap selection facts are supplied;
- whether the measurement is returned-content or selection-scoped;
- measurement ordering and protocol attachment;
- adapter pagination and format serialization;
- readable projection.

The helper must not accept a document path, opaque ref, parser tree, operation result, or adapter session merely to discover text. A fact-based selection estimator may be a distinct mechanical entry point if approved, but it must not become a generic document/navigation abstraction.

Centralizing reusable mechanics avoids calibration drift across adapters, while keeping selection and ref resolution with callers prevents hidden navigation work. A generic producer/sink is not justified unless the approved estimator needs streaming and at least two real callers share its lifecycle and error semantics.

### Decision 6: Evidence and human approval determine the calculator and dependency boundary

**Status: no candidate selected.**

The candidate set remains open. Evaluation must compare viable dependency-free, existing-capability, and source-verified dependency candidates on the same bounded inputs; this design gives no class priority. The comparison corpus must include Markdown, JSON, code, ordinary English, CJK, mixed language, emoji/combining text, whitespace runs, long single pieces/scalars, escapes, and large state/configuration shapes. It must report the approved accuracy statistic(s) and under/over-estimation behavior as well as CPU, peak RSS, cold-start, platform, package-size, and worst-case behavior. Benchmarks and agent recommendations are evidence, not approval.

Any new or replacement production dependency requires a source-backed review of ecosystem adoption, maintainers and release cadence, known security issues/advisories, license compatibility, MSRV and supported targets, transitive graph, native/build requirements, package impact, worst cases, and viable dependency-free/existing alternatives. The user or designated product/architecture owner must explicitly approve it.

### Decision 7: Compatibility flows through independent owner handoffs

**Status: boundary confirmed.**

This change owns the cost contract and records relationship or handoff
requirements only:

| Change or owner | Relationship to this change |
| --- | --- |
| `redesign-find-result-model` | Independent. Its find model does not determine this calculator or encoding; this change requires only that a returned ref not imply target-content measurement. |
| `reuse-adapter-document-state` | Independent. State reuse is neither required nor redesigned; the cost work bounds apply to either lifecycle. |
| `add-json-adapter` | Handoff recipient through its eventual `json-adapter` owner. No synonymous delta is created here while that capability is absent from main specs. |
| `add-json-readable-renderer`, `interactive-outline-selection`, `add-outline-preview-skim-pack` | Handoff recipients that must consume the accepted scope without freezing an unapproved encoding or treating returned-page cost as complete-selection cost. |
| `implement-docnav-mcp-bridge` | Handoff recipient that relays the accepted Docnav fact instead of calculating a separate token cost. |
| Other adapter changes, including the code adapter change | Handoff recipients through their own adapter owners. |

None is a prerequisite or apply-order dependency. Task edits, rebases, and
implementation remain in those changes.

## Risks / Trade-offs

- **[Approximate integers are mistaken for exact model tokens]** → require machine-identifiable approximation and readable wording; document the calibration target and error budget.
- **[Underestimation causes an AI to request too much content]** → select explicit under/over-error criteria, include adversarial multilingual and long-scalar fixtures, and prefer a documented conservative policy if approved.
- **[A visible large outline entry triggers hidden full work]** → accept only cheap source facts or approved bounded samples and test one-entry worst cases.
- **[Outline pagination estimates an item and then drops it]** → approve a bounded page-admission/accounting rule that never runs entry-specific token estimation for non-returned entries.
- **[Returned-page cost accidentally remains selection-scoped]** → give returned read content explicit scope and test nonterminal pages and reassembly.
- **[Mixed line/byte/token scopes become ambiguous]** → machine encoding must distinguish each measurement's meaning; readable output must not collapse incompatible scopes into a misleading summary.
- **[Full-read token thresholds still precompute a non-returned target]** → resolve the existing threshold contract at the human gate; add its owner capabilities before implementation if behavior must change.
- **[A lightweight calculator adds heavy startup/RSS/package cost]** → make resource and package budgets blocking acceptance criteria; a later failure reopens the gate and invalidates the affected approval.
- **[Existing consumers assume exact `tokens`]** → choose an explicit compatibility/migration plan before schema or code changes and verify raw/readable consumers.
- **[The worktree experiment becomes a de facto choice]** → evaluate it only as a reproducible candidate and do not treat existing code or lockfile state as approval.

## Migration Plan

1. Complete tasks 1.1–1.5 and close the implementation decision gate through task 1.6. Record the approved Q1–Q7 answers in `## Decisions`; no implementation task starts before then.
2. Contract first: amend the capability set if the approved encoding changes additional owners, then update full OpenSpec deltas, owner docs, protocol schema, and examples before production code.
3. Follow the testing owner and Case-maintenance process; establish current/failing contract and resource evidence for returned-page read, nested read, unstructured full-read, current-page outline estimates, large visible selections, and find non-measurement.
4. Implement the smallest approved shared mechanics and attach cost in eligible adapters/results without unapproved dependencies or abstractions. If implementation evidence invalidates Q2–Q4, execute the gate-reopening transition before continuing.
5. Update readable projection and accepted active-change handoffs, then validate protocol/readable/schema/example parity, real CLI pagination, corpus accuracy, CPU/RSS/cold-start/package budgets, workspace checks, and release artifacts. Any failure governed by Q2–Q4 returns to the same gate rather than weakening acceptance criteria.
6. Archive only after the approved contract is synchronized and verified. If a public encoding migration requires compatibility staging, use the explicitly approved staging/rollback plan; do not silently revert or dual-write fields.

## Open Questions

These are the complete blocking decision register. Agents may prepare evidence
and recommendations, but only the user or designated product/architecture owner
may approve an answer. Task 1.6 removes an item only by promoting its approved
answer into `## Decisions`.

1. **Q1 — Machine representation:** What machine encoding identifies approximation and distinguishes returned-content estimates, visible-selection estimates, and any approved unavailable state while meeting the compatibility policy?
2. **Q2 — Calibration and error:** Which reference tokenizer(s), corpus weighting, accuracy statistics, maximum error/underestimation, and worst-case behavior define an acceptable estimate?
3. **Q3 — Resource budgets:** What CPU, peak-RSS, cold-start, package-size, platform/target, per-entry/page, and adversarial input-shape budgets must the calculator meet?
4. **Q4 — Calculator and dependency:** Which measured calculator is approved, and is every new or replacement production dependency explicitly approved after the required ecosystem, maintenance, security, license, MSRV, transitive, native/build, package, worst-case, and alternatives review?
5. **Q5 — Full-read threshold:** What explicit contract lets the existing token-valued unstructured-full-read threshold obey the rule that target token cost is not calculated for content that is not returned? If no approved bounded fact can support it, what migration changes the owning capability?
6. **Q6 — Consumer migration:** What compatibility/versioning and migration policy applies to schema, examples, and consumers that interpret `unit: "tokens"` as exact selection cost?
7. **Q7 — Structured-outline admission:** How does page assembly establish current-page membership before entry-specific estimation while preserving the existing character budget and accounting for the approved machine/readable representation?
