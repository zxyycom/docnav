**This provisional delta keeps Current page-local find auto-read until the owner approves one model, one ref scope, and the work needed to prove it.**

## MODIFIED Requirements

### Requirement: unique-ref eligibility uses refs in the current returned result

After a successful validated base response, navigation MUST evaluate eligible non-empty opaque refs using string-exact equality and MUST invoke read exactly once only when the finalized auto-read scope proves one ref. Structured outline MUST continue to use refs in its current returned result. Find MUST use exactly the current-page or query-global scope explicitly approved and finalized in this requirement; before that approval, find MUST retain Current current-page eligibility and implementation of another scope is blocked. Navigation MUST consume adapter-produced logical units/completeness facts without parsing refs or reconstructing occurrence, node, group, ordering, or search semantics.

#### Scenario: One returned outline ref invokes read
- **WHEN** a structured outline succeeds
- **AND** the current result contains exactly one distinct non-empty ref string
- **THEN** navigation invokes read exactly once with that ref

#### Scenario: Repeated Current find occurrences share one read target
- **WHEN** the find model remains Current occurrence-oriented
- **AND** every occurrence on the current returned page carries the same non-empty ref string
- **THEN** navigation treats that returned-page ref as unique
- **AND** invokes read exactly once

#### Scenario: Approved current-page find scope
- **WHEN** the approved find auto-read scope is current-page
- **AND** exact-ref deduplication over all eligible final logical units on that returned page produces one non-empty ref
- **THEN** navigation invokes read exactly once with that ref
- **AND** does not interpret the ref as globally unique across other pages

#### Scenario: Approved query-global find scope
- **WHEN** the approved find auto-read scope is query-global
- **AND** the approved adapter/navigation boundary provides authoritative proof that the complete query result maps to one exact non-empty ref
- **THEN** navigation invokes read exactly once with that ref
- **AND** does not derive the proof from current-page cardinality alone

#### Scenario: Incomplete global evidence suppresses read
- **WHEN** the approved find auto-read scope is query-global
- **AND** the current result, group, count, or uniqueness evidence is partial or otherwise not proven complete
- **THEN** navigation does not invoke read
- **AND** returns the validated base response unchanged

#### Scenario: No eligible ref keeps the base response
- **WHEN** the structured result contains no eligible non-empty ref
- **THEN** navigation does not invoke read
- **AND** returns the validated base response unchanged

#### Scenario: Multiple eligible refs keep the base response
- **WHEN** the approved scope proves more than one distinct exact ref string
- **THEN** navigation does not invoke read
- **AND** returns the validated base response unchanged

#### Scenario: Unstructured outline keeps its content response
- **WHEN** outline returns the unstructured content branch
- **THEN** navigation does not invoke read
- **AND** returns the validated base response unchanged

#### Scenario: Unapproved find scope retains Current behavior
- **WHEN** the find model, auto-read scope, and required work have not all been explicitly approved and finalized
- **THEN** navigation evaluates only exact refs in the Current returned find page
- **AND** it does not scan the source, inspect later pages, or request adapter-private ref interpretation to establish query-global uniqueness
