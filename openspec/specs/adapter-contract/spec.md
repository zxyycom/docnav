# adapter-contract Specification

## Purpose
Define linked adapter interface boundaries: static descriptors, manifest format/routing metadata, adapter-owned native option declarations, operation handler inputs, structured operation results, adapter diagnostics, and optional full-read support hooks. `protocol-contract` owns raw envelopes; `output-contract` owns public output rendering.
## Requirements
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

### Requirement: Adapter definition owns registry-facing adapter facts

Adapter definition, manifest, and descriptor metadata MUST describe adapter identity, supported format facts, pathname routing hints, capability declarations, and the linked strategy implementation. The adapter definition MUST be the registry-facing aggregation point for those adapter behavior facts. Each format descriptor MUST expose a project-owned normalized format identity, one or more dotted `extensions[]`, and a `filenames[]` array that MAY be empty. Each extension value MUST be a basename suffix with a leading dot, MAY contain additional dots, and MUST NOT contain a path separator. Filename values MUST contain one exact basename without a path separator and MUST NOT equal `.` or `..`. Automatic routing MUST compare exact filenames case-sensitively first. Otherwise it MUST compare the complete basename and extension suffixes after ASCII case normalization and MUST choose the longest matching suffix. Across the core static registry, one normalized format identity MUST resolve to at most one adapter definition; one ASCII-normalized suffix or one exact filename MUST resolve to at most one format identity within its hint kind. Registry construction, doctor, and release validation MUST reject exact duplicates before document routing. Different-length suffix overlap MUST remain valid and deterministic rather than be classified as a conflict. Exact filename and suffix are distinct hint kinds, so one basename MAY have an exact-filename mapping that overrides its generic suffix mapping.

The fixed adapter strategy interface MUST provide outline, read, find, and info functions and MUST NOT define a probe function, probe result, probe reason, probe version, or selection-detection hook. Adapter-private helpers MAY construct manifest or capability values, but shared layers MUST consume adapter behavior facts through the exported definition/factory. Adapter implementation source MUST remain a core static-registry fact. Caller-configurable document-operation parameter facts MUST come from the separate core catalog. Pathname matching MUST remain navigation-private routing mechanics and MUST NOT transfer document decode, parse, ref, or operation semantics away from the selected adapter.

#### Scenario: Core lists built-in adapters

- **WHEN** `docnav adapter list` inspects adapters
- **THEN** implementation source comes from the core static registry
- **THEN** manifest metadata describes adapter capability, normalized format support, complete-basename suffixes, and exact filenames
- **THEN** listing does not execute a selection probe
- **THEN** document-operation parameter facts remain in the separate core catalog

#### Scenario: Registry consumes one adapter definition

- **WHEN** a built-in adapter is registered with core
- **THEN** the registry receives one adapter definition containing identity, format descriptors, a linked strategy implementation, and optional capabilities
- **THEN** the fixed strategy interface provides the required operations and exposes no probe surface
- **THEN** caller-configurable parameter facts come from the core catalog

#### Scenario: Automatic routing uses manifest pathname hints

- **WHEN** navigation receives a document pathname without explicit adapter intent
- **THEN** registry-derived exact-filename and normalized-suffix lookup views map the complete basename to a manifest format identity
- **THEN** registry matching compares that identity exactly with definition format descriptors
- **THEN** an adapter definition is not executed merely to decide whether it is a candidate
- **THEN** registry order does not choose between equal format identities

#### Scenario: Exact filename overrides a generic suffix

- **WHEN** one complete basename matches a declared exact filename and also ends with another declared suffix
- **THEN** the exact filename mapping supplies the routing identity
- **THEN** the cross-kind overlap is not treated as a duplicate declaration

#### Scenario: Registry declares duplicate format identity

- **WHEN** two built-in adapter definitions declare the same normalized format identity
- **THEN** core registry construction, doctor, and release validation reject the registry
- **THEN** no document invocation uses registry order to choose one definition

#### Scenario: Registry declares duplicate pathname hint

- **WHEN** two format descriptors declare the same ASCII-normalized suffix or the same exact filename
- **THEN** core registry construction, doctor, and release validation reject the registry
- **THEN** runtime does not reinterpret the conflict as document ambiguity

#### Scenario: Compound suffix is more specific

- **WHEN** no exact filename matches and basename `model.schema.JSON` is checked against `.json` and `.schema.json`
- **THEN** ASCII normalization makes both suffixes eligible
- **THEN** `.schema.json` supplies the routing identity because it is the longest match
- **THEN** the overlap is not rejected as a duplicate pathname hint

#### Scenario: Adapter implementation uses private helpers

- **WHEN** an adapter implementation splits definition construction across private helper functions or modules
- **THEN** it exports one registry-facing definition or definition factory
- **THEN** registry, navigation, and dispatch consume adapter-owned behavior facts through that definition
- **THEN** core catalog remains the only parameter-definition input

### Requirement: Adapter results preserve format semantics
Adapters MUST return structured operation results or adapter diagnostics that preserve format-owned facts such as refs, content type, parse boundaries, cost facts, and operation-specific item metadata. Core and output layers MUST project those facts without replacing adapter semantics.

#### Scenario: Adapter returns read content
- **WHEN** a linked adapter returns read content with `content_type`
- **THEN** core and output surfaces preserve that content type
- **THEN** display rendering may summarize the content without changing its machine facts

### Requirement: Adapter operation support is explicit
Adapter definitions MUST declare supported document operations and capability groups, including unstructured full-read support, content, cost measurement, and result facts used by navigation pre-dispatch policy. Required operation handler handles and capability group handles MUST be reachable from the same adapter definition. Navigation uses declared support facts when selecting adapter-level capabilities. Capability groups MUST aggregate related hooks under one declared owner boundary.

#### Scenario: Adapter supports unstructured full read
- **WHEN** an adapter declares a full-read capability group
- **THEN** navigation may use that declaration for opt-in full-read pre-dispatch
- **THEN** the adapter still owns the content and cost facts it returns
- **THEN** support, content, cost measurement, and result facts are interpreted within the declared full-read capability boundary

#### Scenario: Capability boundary is unavailable
- **WHEN** policy requires a capability outside the selected adapter definition
- **THEN** navigation reports the unsupported boundary
- **THEN** fallback behavior must come from a declared owner rather than inference

#### Scenario: Full-read capability complements operation handlers
- **WHEN** an adapter declares an optional full-read capability group
- **THEN** the adapter still declares the required `outline`, `read`, `find`, and `info` operation handlers
- **THEN** navigation uses the optional capability only for the policy path that explicitly permits it

### Requirement: Adapter handlers remain downstream of typed validation

Value decoding, nullability required for materialization, default, source-precedence, merge-strategy handling, and standard type materialization for caller-configurable document-operation parameters MUST complete before adapter dispatch. Core catalog MAY also require context-independent enum, range, shape, or other validation before dispatch. Adapter strategy functions MUST receive only the closed operation-specific standard typed input as caller data and MAY validate or repeat validation of adapter-specific semantics. Accepted parameters, source locators, defaults, merge rules, standard-input bindings, and declaration metadata MUST remain owned by the core catalog.

#### Scenario: Invalid config value cannot be materialized

- **WHEN** a config source provides a value that cannot be materialized as the core-defined standard type
- **THEN** navigation or the consuming input boundary reports the diagnostic before adapter dispatch
- **THEN** the adapter strategy is not invoked with the malformed raw value

#### Scenario: Core defers adapter semantics

- **WHEN** core performs only structural or minimal validation for an adapter-scoped value
- **THEN** the selected strategy receives the well-typed standard value
- **THEN** the strategy validates every additional precondition required by its algorithm

#### Scenario: Core and adapter repeat a rule

- **WHEN** both core and the selected strategy check the same adapter-scoped constraint
- **THEN** both checks accept the same value domain
- **THEN** either rejection maps to a compatible observable diagnostic

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

Auxiliary adapter extensions MUST remain subordinate to ref identity. Cost,
metadata, full-read/source facts, preview facts, rendering inputs, and other
auxiliary behavior MAY use methods, hooks, accessors, or shared helpers
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
