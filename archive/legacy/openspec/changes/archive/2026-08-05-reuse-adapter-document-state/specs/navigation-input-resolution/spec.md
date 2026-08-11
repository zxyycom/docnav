## ADDED Requirements

### Requirement: Navigation carries one compatible adapter view through composition

Navigation MUST complete adapter selection, filesystem-backed path/access
normalization, typed input resolution, and core-owned validation before
creating the selected adapter's invocation-private document boundary. Selection
MUST perform no target metadata lookup, open, read, decode, parse, or adapter
document creation. The selected adapter MUST preserve each behavior's Current
ordering between adapter-owned semantic validation and first document access,
and MUST initialize compatible private state at most once when access is
required.

Eligible selected-adapter work in the same invocation MUST reuse that captured
view without refreshing it. For unique-ref composition, the producer result and
nested read MUST use the same compatible view, ref MUST remain unchanged, and
the adapter MUST satisfy the ref-contract round-trip law. Same-view reuse MUST
NOT be treated as sufficient evidence without adapter conformance coverage.

Preparation or selected execution failure MUST preserve the existing diagnostic
or fallback owner. It MUST NOT rerun pathname routing, select another registry
definition, expose private state, or silently rebuild the same view. Auxiliary
cost/info/full-read behavior MAY use its existing operation/hook/fact shape and
MUST preserve its current result semantics.

#### Scenario: Routing finishes without document state

- **WHEN** explicit or automatic routing selects one adapter
- **THEN** selection has used only registry and pathname facts
- **THEN** no adapter document or private document state exists yet
- **THEN** selected-adapter document access starts only in later execution

#### Scenario: Adapter rejects before document access

- **WHEN** Current selected behavior rejects an adapter-owned precondition before opening the path
- **THEN** the reusable boundary preserves that diagnostic ordering
- **THEN** no source acquisition or private parse state is created

#### Scenario: Unique-ref composition uses one view

- **WHEN** validated outline/find eligibility triggers nested read
- **THEN** navigation passes the exact candidate ref and starts read at page `1`
- **THEN** producer and read use the same prepared view
- **THEN** read satisfies compatible-view success and current nested/composed validation semantics

#### Scenario: Non-successful composition preserves the base response

- **WHEN** nested execution or composed response validation fails for a reason outside a compatible-view producer/read disagreement
- **THEN** navigation preserves the existing validated-base fallback
- **THEN** it releases private state under the bounded lifecycle
- **THEN** it adds no private failure or cleanup fact to public output

#### Scenario: The path changes after preparation

- **WHEN** the path is replaced, mutated, deleted, repaired, or made invalid after successful preparation
- **THEN** the current invocation continues on its captured view
- **THEN** navigation does not refresh or reroute
- **THEN** a later invocation evaluates compatibility and stale-ref behavior against its newly prepared view

#### Scenario: Navigation uses its existing UTF-8 fallback

- **WHEN** owner-defined full-read policy selects the navigation-owned default fallback
- **THEN** navigation preserves the existing raw UTF-8 fallback and result semantics
- **THEN** the fallback is not treated as a ref producer or proof of adapter-state reuse
