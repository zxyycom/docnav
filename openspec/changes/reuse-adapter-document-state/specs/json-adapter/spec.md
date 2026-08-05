## ADDED Requirements

### Requirement: JSON emitted refs round-trip through one compatible prepared view

Eligible JSON work MUST be able to reuse one private prepared document after
`docnav-json` is selected and execution reaches document access. That document
contains the decoded JSONC source, logical tree, source regions, comment
attribution, canonical-ref facts, and source-derived metadata. The
implementation MAY split or combine private algorithms as needed; method count
and callable shape are not part of this requirement.

Every ref emitted by JSON outline or find MUST be complete and canonical under
the Current base/direct-comment/tail grammar. Read with valid existing input
MUST accept and successfully materialize that ref on the same prepared JSON
view and on an independently prepared compatible view with identical source and
relevant facts. Resolution MUST require no producer call order, in-memory node
pointer, or producer-only state.

The resolved selection MUST preserve Current JSON semantics: base refs produce
strict JSON; direct-comment and tail refs produce their comment-aware JSONC
projection. Correspondence to find evidence MAY be container-level or
normalized and does not require the returned content to contain literal source
punctuation, whitespace, comment boundary text, or original scalar spelling.

#### Scenario: JSON outline ref round-trips

- **WHEN** JSON outline emits a base, direct-comment, or tail ref
- **THEN** read page `1` with the exact ref succeeds on the same prepared view
- **THEN** read also succeeds after independently preparing identical JSON/JSONC source and relevant facts
- **THEN** the selection uses the Current view-specific materialization

#### Scenario: JSON find ref round-trips

- **WHEN** JSON find attributes an original-source occurrence to a ref
- **THEN** read with the exact ref succeeds on a compatible JSON view
- **THEN** the selected base/direct/tail region corresponds to the occurrence under the Current JSON owner rules
- **THEN** repeated occurrences MAY reuse the same ref

#### Scenario: Multiple JSON refs or matches share a selection

- **WHEN** more than one emitted ref or occurrence resolves to the same logical/container region
- **THEN** each emitted ref independently satisfies canonicality and read success
- **THEN** the contract does not require one-to-one ref/region identity

#### Scenario: JSON view becomes incompatible

- **WHEN** later source or relevant ref facts differ from the producer view
- **THEN** Current JSON missing/invalid/current-view resolution behavior applies
- **THEN** that incompatible-view outcome does not violate compatible-view consistency

#### Scenario: JSON auxiliary facts reuse preparation

- **WHEN** JSON info, cost, or unstructured full-read behavior participates in the same invocation
- **THEN** it may reuse compatible prepared source/model facts
- **THEN** its existing outward result semantics remain unchanged
- **THEN** it does not become an alternative ref identity owner
