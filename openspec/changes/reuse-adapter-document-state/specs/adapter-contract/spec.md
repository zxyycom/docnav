**Interpretation:** This approved Target delta adds reusable private document
state and ref consistency responsibilities without standardizing the number or
shape of adapter algorithms. Existing outward operations/capabilities remain
the Current compatibility surface unless another owner changes them.

## MODIFIED Requirements

### Requirement: Linked adapter handlers receive prepared operation input

Linked adapter strategy functions MUST receive one core-prepared,
operation-specific closed typed input after adapter selection, source
resolution, merge/default handling, standard type materialization, request
binding, and configured core validation have completed. The existing shared
operation contract MUST define its Rust types so navigation and adapters can
share the boundary, while core-owned bindings MUST populate every
strategy-visible value through compile-time fields, typed accessors, or closed
enum variants. Shared placement MUST NOT transfer product parameter ownership
away from core. “Prepared input” means the strategy does not process raw
sources or parameter declarations; it MUST NOT imply that every adapter-specific
semantic precondition has already been checked. Protocol envelopes, serialized
options, generic parameter lookup, raw configuration source material, parameter
declarations, and source-priority metadata MUST remain outside the strategy data
boundary.

After final selection and applicable input/path resolution, linked execution
MUST create at most one invocation-private adapter document for the normalized
path. Adapter behavior MUST preserve its Current ordering between
algorithm-specific validation and first document access. That first required
access MUST initialize at most one compatible private source/model/index view,
which eligible later handlers or extensions MAY reuse. Parser/source/index
state MUST remain outside closed operation input and MUST NOT become a second
caller-data argument, generic lookup bag, or caller-visible handle.

#### Scenario: Strategy receives outline input

- **WHEN** navigation dispatches an outline operation to a selected adapter
- **THEN** the strategy receives the normalized document path and typed outline arguments
- **THEN** applicable core-defined adapter-scoped values are already present in prepared operation input
- **THEN** raw source resolution, default handling, and standard type materialization are complete
- **THEN** the strategy does not query a generic parameter bag or protocol request for those values
- **THEN** the strategy may still validate or revalidate adapter-specific semantics

#### Scenario: Input cannot be standardized

- **WHEN** caller input cannot be decoded, merged, defaulted, or materialized as the standard operation input type
- **THEN** navigation or the owning input boundary reports the diagnostic
- **THEN** the linked adapter strategy is not invoked with the malformed raw value

#### Scenario: Strategy consumes a core-defined format parameter

- **WHEN** core defines `max_heading_level` for Markdown outline/find and resolution succeeds
- **THEN** the Markdown strategy receives the prepared integer through a compile-time operation-input field or typed accessor
- **THEN** source priority, merge, default, and binding work are already complete
- **THEN** the strategy may validate or repeat the range check in its Current order relative to document access

#### Scenario: Strategy rejects a semantic failure

- **WHEN** a standard typed value satisfies core materialization but violates an adapter algorithm precondition not guaranteed by core validation
- **THEN** the selected strategy validates the value in its Current order
- **THEN** it returns a standard diagnostic through the adapter contract

#### Scenario: Composed work reuses private state

- **WHEN** navigation invokes eligible later selected-adapter work after private state initializes
- **THEN** that work may use the existing compatible source/model/index/ref facts
- **THEN** it does not reacquire, decode, or parse the complete view solely because the stage changed
- **THEN** private state remains absent from caller input and public output

## ADDED Requirements

### Requirement: Adapter ref producers and read obey compatible-view consistency

Any adapter behavior that successfully emits a caller-visible ref MUST act as a
ref producer. Every emitted ref MUST be complete, non-empty, canonical under the
adapter's documented grammar, and readable with valid existing read input on
the same prepared view and on an independently prepared compatible view. Read
MUST echo the exact ref and return a validated success selecting the
adapter-documented region. Producer/read disagreement on a compatible view is
an adapter contract violation.

The read result MUST require no hidden producer call order, pointer, unexposed
producer-only option, or state unavailable from an independently prepared
compatible view. Multiple refs MAY select one region and one ref MAY represent
multiple producer occurrences. Incompatible-view stale behavior remains owned
by the adapter's documented ref/error contract.

#### Scenario: Outline ref round-trips

- **WHEN** a validated outline success emits ref `r` over document view `V`
- **THEN** read page `1` with valid input and exact ref `r` succeeds on `V`
- **THEN** independently preparing identical source and relevant adapter facts produces a compatible view on which the same read also succeeds
- **THEN** the read result echoes `r` unchanged and selects the outline entry's documented region

#### Scenario: Find ref round-trips

- **WHEN** a validated find success emits ref `r` for match evidence over compatible view `V`
- **THEN** read with exact ref `r` succeeds on `V`
- **THEN** the selection corresponds to the evidence according to the adapter owner
- **THEN** correspondence does not require universal literal-query containment when the owner defines a normalized or container-level selection

#### Scenario: Paging preserves producer refs

- **WHEN** a producer truncates labels or optional facts to satisfy a page budget
- **THEN** every emitted ref remains complete and canonical
- **THEN** each emitted ref still satisfies compatible-view read consistency

#### Scenario: Ref is evaluated on an incompatible view

- **WHEN** source, relevant adapter facts, or ref semantics make the later view incompatible
- **THEN** the adapter may return its documented stale, invalid, missing, ambiguous, or newly resolved outcome
- **THEN** that outcome does not violate the compatible-view law

### Requirement: Auxiliary adapter extensions do not compete with ref identity

Cost, metadata, full-read/source facts, preview facts, rendering inputs, and
other auxiliary behavior MAY use methods, hooks, accessors, or shared helpers
appropriate to their owner. This contract MUST NOT classify their count or
callable shape as the adapter's foundational architecture. An auxiliary
extension that emits a caller-visible ref becomes a ref producer and MUST obey
the same compatible-view laws. Otherwise it MUST NOT parse, rewrite, or become
an alternative identity owner for refs. Readable rendering remains owned by the
output layer.

#### Scenario: Cost extension consumes selected facts

- **WHEN** an adapter or shared helper supplies cost facts
- **THEN** those facts may describe existing operation content or selection
- **THEN** they do not change ref grammar, identity, or read resolution

#### Scenario: Rendering consumes adapter facts

- **WHEN** readable output renders adapter-produced result facts
- **THEN** output remains the presentation owner
- **THEN** display text does not become ref identity or a source from which shared layers reconstruct refs

#### Scenario: Future extension emits a ref

- **WHEN** an auxiliary capability begins emitting caller-visible refs
- **THEN** it is also treated as a ref producer
- **THEN** every emitted ref must pass the same compatible-view conformance evidence
