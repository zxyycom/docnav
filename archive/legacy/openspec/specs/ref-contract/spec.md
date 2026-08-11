# ref-contract Specification

## Purpose
Define the cross-layer ref contract: refs are adapter-generated opaque strings, public callers pass them unchanged, shared layers validate only shared input requirements, and adapter-specific docs own grammar, parsing, matching, and error classification.
## Requirements
### Requirement: Refs are opaque across shared layers
Shared Docnav layers MUST treat refs as opaque non-empty strings. Adapter-specific ref grammar is parsed only by the selected adapter.

#### Scenario: Caller reads an outline ref
- **WHEN** outline returns a ref
- **THEN** the caller can pass that string unchanged to read
- **THEN** shared layers preserve the exact string until the selected adapter parses it

### Requirement: Explicit ref input is validated only at shared boundary
Shared input boundaries MUST reject missing or empty explicit refs before adapter dispatch. Non-empty refs MUST pass through unchanged until the selected adapter applies its grammar.

#### Scenario: Missing ref
- **WHEN** a read operation requires a ref and the caller omits it
- **THEN** core or navigation reports an invalid request diagnostic
- **THEN** adapter-specific ref parsing is not invoked

#### Scenario: Adapter-specific ref string
- **WHEN** a caller provides a non-empty ref
- **THEN** shared layers pass it through unchanged
- **THEN** the selected adapter decides whether its grammar accepts it

### Requirement: Adapters own ref generation and parsing
Each adapter MUST own ref grammar, uniqueness strategy, structural snapshot semantics, parse errors, match errors, and ambiguity handling for its format.

#### Scenario: Invalid adapter ref grammar
- **WHEN** the selected adapter receives a non-empty ref that violates its grammar
- **THEN** the adapter reports the adapter-owned invalid-ref diagnostic
- **THEN** shared layers project the diagnostic without reinterpreting the grammar

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
