**Planning state: artifact-ready / implementation-blocked.** This change keeps AI-facing token cost while redefining it as a low-overhead approximation whose work is bounded to returned content or cheap facts for currently visible selections.

## Why

Token cost helps an AI decide what to read next, but owner material marked an exact-token helper and selection-scoped measurements as Current when this change was created; those mechanics can make cost calculation more expensive than the bounded result itself. This is especially harmful for large structured state/configuration documents, where hidden serialization or tokenization can defeat Docnav's finite `outline -> ref -> read` workflow. Because this change is not Current implementation evidence, task 1.2 must re-establish the release baseline before approval.

## What Changes

- **BREAKING**: redefine public `tokens` cost from exact tokenizer parity and complete-selection measurement to an explicitly approximate estimate with an approved machine encoding and documented scope.
- Require ordinary read, nested read, and unstructured full-read outline to estimate only the content actually returned. Do not calculate token cost over an unreturned page remainder.
- Require structured outline to enrich only current-page entries with a cheap selection estimate. A visible large entry does not authorize complete hidden serialization, materialization, or tokenization; entries outside the returned page receive no estimate work.
- Require find not to read, serialize, or tokenize referenced target content solely to attach cost. A later or composed read owns the estimate for its own returned content.
- Keep the existing character-based pagination and continuation budgets. Token cost remains an AI-facing report, not a pagination budget.
- Keep token cost required; this change does not add a switch for disabling it.
- Keep implementation blocked until the decision gate in `design.md` is closed by the user or a designated product/architecture owner.
- Return the change to implementation-blocked if later calculator, accuracy, resource, package, platform, or dependency evidence invalidates an approved Q2–Q4 answer. The affected approval must be replaced through the evidence and human gate before dependent implementation continues.
- Record compatibility handoffs for affected active changes while keeping their implementation and task ownership independent.

Non-goals:

- Exact BPE parity with an OpenAI tokenizer or any other model tokenizer.
- Token-based pagination, a public tokenizer selector, or a token-cost off switch.
- Preselecting the machine encoding, calculator, production dependency, calibration target, or budgets from artifact cleanup, benchmark output alone, or an agent recommendation. After sufficient evidence and explicit human approval, the accepted choices belong in `design.md` Decisions and the synchronized standard artifacts.
- Redesigning find result semantics or same-invocation document-state reuse; `redesign-find-result-model` and `reuse-adapter-document-state` remain independent changes.
- Implementing JSON readable rendering, skim behavior, MCP transport, or another adapter change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `docnav-architecture`: replace exact tokenizer semantics in the shared text-cost boundary with mechanism-neutral, bounded approximate-token mechanics while preserving caller ownership of selection, scope, pagination, and presentation.
- `protocol-contract`: require explicit approximate-token facts for returned read/full-read content and cheap current-page structured-outline selection estimates, without implying find target measurement.
- `output-contract`: require readable output to identify estimates and their returned-content or visible-selection scope without calculating missing values.
- `markdown-adapter`: change Markdown read and outline cost scope/work bounds while preserving Markdown refs, regions, character pagination, and continuation semantics.

## Impact

- Eventual implementation may affect `docnav-text-cost`, protocol result models, navigation/adapters that attach cost scope, the built-in readable renderer, Markdown, protocol schemas/examples, and tests. Exact files and any migration shape depend on approved answers to `design.md` Q1–Q7.
- The existing token-valued unstructured-full-read threshold must be audited against the no-hidden-work rule before implementation. If its public selection semantics must change, the proposal and deltas must first add the owning `navigation-input-resolution` and/or `adapter-contract` capability rather than silently changing them in code.
- Structured-outline page admission must be resolved together with the approved encoding so character budgets remain valid without estimating a candidate that is later excluded from the returned page.
- `add-json-adapter` has not yet produced an archived main `json-adapter` capability, so this change does not create a synonymous delta. Its accepted cost contract must be handed to that owner once available.
- `redesign-find-result-model` and `reuse-adapter-document-state` are independent, not prerequisites. `add-json-readable-renderer`, `interactive-outline-selection`, `add-outline-preview-skim-pack`, `implement-docnav-mcp-bridge`, and relevant adapter changes may consume the accepted contract later; their implementation and task edits remain with their own changes.
- Any proposed new dependency requires explicit user approval after source-backed review of ecosystem adoption, maintenance, security, license, MSRV/platform support, transitives, package impact, and viable alternatives.
