## ADDED Requirements

### Requirement: Linked adapter execution reuses one view while ref laws define correctness

After Current lexical no-probe selection, architecture MUST create at most one
selected-adapter document boundary and initialize at most one compatible private
document view when execution first requires document access. Navigation MUST
own stage ordering, policy, result validation, fallback, and the reachability
bound. The selected adapter MUST own acquisition/decoding/parsing,
validation-versus-access ordering, private source/model/index/source-region
facts, ref generation/resolution, format search/navigation semantics, ordered
bounded results, selection materialization, and diagnostics.

Prepared-state reuse MUST be treated as the same-view and resource mechanism,
not as proof that producer and read algorithms agree. Every adapter ref producer
and read consumer MUST additionally satisfy the `ref-contract`
compatible-view canonicality, round-trip, no-hidden-context, and correspondence
laws. Architecture MUST NOT prescribe their function count or private helper
shape.

Cost, metadata, full-read/source facts, preview facts, and rendering inputs MAY
remain auxiliary extension surfaces. They MUST NOT become competing ref
identity owners; any extension that emits refs inherits the ref-producer laws.
Private state MUST remain inside the linked process, end with the bounded
invocation, and remain absent from protocol, output, ref, continuation, logging,
schema, caller input, and cross-invocation caches.

#### Scenario: One invocation produces and consumes a ref

- **WHEN** navigation runs a ref-producing operation and eligible nested read
- **THEN** both may reuse the same prepared view
- **THEN** navigation passes the ref unchanged
- **THEN** the adapter's ref contract, not shared-state existence alone, guarantees the read round trip

#### Scenario: Another invocation prepares an equivalent document

- **WHEN** a later invocation selects the same adapter and independently prepares identical source and relevant facts
- **THEN** the later view is compatible for the emitted ref
- **THEN** read remains successful without access to the earlier in-memory state

#### Scenario: Auxiliary facts are projected

- **WHEN** cost, info, full-read, preview, or readable rendering consumes adapter-produced facts
- **THEN** the corresponding existing owner retains its semantics
- **THEN** those facts do not reinterpret or reconstruct ref identity

#### Scenario: Future execution crosses an invocation or process boundary

- **WHEN** an integration wants to retain prepared state across a request, prompt, or process boundary
- **THEN** this capability does not authorize that retention or serialization
- **THEN** compatibility must come from the opaque ref and independently prepared compatible view, not a public state handle
