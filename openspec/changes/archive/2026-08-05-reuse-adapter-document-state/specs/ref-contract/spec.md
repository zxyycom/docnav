## MODIFIED Requirements

### Requirement: Find and outline refs use the shared pass-through flow

Refs returned by a validated successful outline or find result MUST be complete,
non-empty opaque strings. Shared layers MUST validate only shared ref input
shape and MUST pass the exact string unchanged to the selected adapter.

For every emitted ref `r` over document view `V`, read with valid existing read
input MUST return a validated successful result that echoes `r` and resolves the
adapter-documented selection when evaluated against:

1. the same prepared view `V`; or
2. an independently prepared compatible view that uses identical source and
   identical relevant adapter/ref facts.

This compatible-view guarantee is owned by the adapter's generation, parsing,
lookup, and selection contract. Shared layers MUST NOT parse the ref or invoke
read after every emitted entry at production runtime. Built-in adapters MUST
prove the guarantee through shared black-box conformance evidence.

The guarantee MUST NOT be interpreted as cross-mutation or cross-ref-semantics
stability. Same path alone does not establish compatibility. On an incompatible
view, the adapter MAY follow its documented missing, invalid, ambiguous, or
newly resolved behavior. Multiple refs MAY select one region and one ref MAY
appear for multiple find occurrences.

#### Scenario: Outline or find ref is passed unchanged to read

- **WHEN** outline or find returns a non-empty ref
- **AND** the caller submits the same ref to read
- **THEN** shared layers pass the exact ref unchanged to the selected adapter
- **THEN** shared layers do not parse, normalize, reconstruct, or infer the ref from display text

#### Scenario: Producer ref is read on the same view

- **WHEN** a validated producer result emits ref `r` over prepared view `V`
- **AND** read receives valid input, `r`, and the same view `V`
- **THEN** read returns a validated success and echoes `r`
- **THEN** the selected adapter resolves the producer-documented selection

#### Scenario: Producer ref is read after compatible re-preparation

- **WHEN** a caller submits emitted ref `r` in another invocation
- **AND** the selected adapter independently prepares identical source and relevant ref facts
- **THEN** the resulting view is compatible
- **THEN** read returns a validated success without hidden producer-only state

#### Scenario: Producer ref is read on an incompatible view

- **WHEN** source mutation, relevant configuration, or ref-semantic change makes the read view incompatible with the producer view
- **THEN** the adapter applies its documented current-view ref and error behavior
- **THEN** a missing, invalid, ambiguous, or differently resolved outcome does not violate this requirement

#### Scenario: Conformance is not a production double-read

- **WHEN** outline or find returns refs in normal production execution
- **THEN** core does not call read for every result solely to verify the adapter
- **THEN** contract tests and adapter-owner evidence classify compatible-view disagreement as a defect
