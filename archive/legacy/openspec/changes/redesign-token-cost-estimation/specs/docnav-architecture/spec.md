**Target delta.** It bounds shared token-estimation mechanics; `../../design.md` owns the unresolved calculator, encoding, budgets, dependency, and implementation gate.

## MODIFIED Requirements

### Requirement: Shared helpers preserve policy ownership
Shared crates and helpers MUST centralize only reusable mechanics. CLI behavior, adapter semantics, protocol envelopes, output projections, diagnostics, refs, and validation material keep their owning capabilities even when they share helper code. Shared token-estimation mechanics MUST operate only on returned text supplied by the caller or on cheap facts supplied to produce a visible-selection estimate for an entry on the current structured-outline page. They MUST NOT resolve refs, select or serialize format content, inspect entries outside the current page, or own measurement scope, pagination, protocol attachment, or readable presentation. Approximate-token mechanics MUST meet the explicitly approved accuracy and CPU, memory, cold-start, platform, and package budgets; they are not required to reproduce an exact model tokenizer.

#### Scenario: Helper is reused across layers
- **WHEN** multiple components consume a shared helper
- **THEN** each component keeps its observable policy in its own capability
- **THEN** the helper exposes mechanics without redefining that component's public contract

#### Scenario: Returned content is estimated
- **WHEN** a read or unstructured full-read caller requests a token estimate
- **THEN** the caller supplies only the content selected for return
- **THEN** the helper does not acquire the document, resolve a ref, or inspect an unreturned page remainder

#### Scenario: Visible-selection estimate uses cheap facts
- **WHEN** an adapter estimates the readable selection represented by a current-page structured-outline entry
- **THEN** the approved helper consumes only cheap existing facts or an input bounded by the approved per-entry and per-page budgets
- **THEN** the helper does not serialize or tokenize the complete target merely because its ref is visible

#### Scenario: A referenced target is not returned
- **WHEN** find or another result exposes a ref without returning the referenced content
- **THEN** shared token-estimation mechanics are not invoked on that target solely to enrich the result
