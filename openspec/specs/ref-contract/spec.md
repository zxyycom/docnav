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

### Requirement: Find and outline refs must roundtrip to read
Refs returned by outline or find MUST be valid read inputs for the same document state and selected adapter, unless the adapter explicitly reports that the referenced region is no longer available.

#### Scenario: Find match
- **WHEN** find returns a match ref
- **THEN** read with that ref returns content containing or corresponding to the match
- **THEN** the same adapter owns any stale or unmatched ref diagnostic
