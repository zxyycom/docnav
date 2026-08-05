## MODIFIED Requirements

### Requirement: Markdown adapter provides v0 document operations

The Markdown adapter MUST implement outward outline, read, find, and info for
Markdown documents through the linked adapter contract. Selected execution MUST
be able to reuse one invocation-private Markdown document containing compatible
source, line, heading, section, and ref facts without prescribing the private
algorithm or method count.

Every ref emitted by Markdown outline or find MUST be complete and canonical.
Read with valid existing input MUST accept and successfully materialize that ref
on the same prepared Markdown view and on an independently prepared compatible
view with identical source and relevant facts. The ref plus compatible view and
existing read input MUST suffice without producer-only in-memory state.

#### Scenario: Supported Markdown document

- **WHEN** the selected adapter is Markdown and the document is supported
- **THEN** outline, read, find, and info are available through the standard document operation flow
- **THEN** eligible work may reuse one private Markdown document view

#### Scenario: Markdown outline ref round-trips

- **WHEN** Markdown outline emits a heading, `HEAD:leading`, or `doc:full` ref
- **THEN** read page `1` with the exact ref succeeds on the same view
- **THEN** read also succeeds after independently preparing identical Markdown source and relevant facts
- **THEN** the read selection corresponds to the documented section, document head, or full document

#### Scenario: Markdown find ref round-trips

- **WHEN** Markdown find emits the visible-region ref for a source occurrence
- **THEN** read with the exact ref succeeds on a compatible Markdown view
- **THEN** the selected content is the documented containing heading section, document head, or full fallback region

#### Scenario: Markdown view becomes incompatible

- **WHEN** a later invocation prepares changed Markdown structure that is incompatible with the producer view
- **THEN** existing Markdown structural snapshot and ref-error semantics apply
- **THEN** that outcome does not weaken the compatible-view guarantee

### Requirement: Markdown supports declared unstructured full-read outline

Markdown unstructured full-read outline support MUST be declared through
adapter capability metadata before navigation can use it. Normal structured
outline behavior MUST remain unchanged when policy does not apply. Cost
measurement, full-content production, and structured-outline fallback over one
captured invocation view MUST reuse compatible Markdown preparation rather than
reacquire/decode/parse solely because the stage changed.

Full-read content/cost implementation remains an auxiliary capability and MAY
use the existing hook shape or a later owner-approved source/fact projection.
This change MUST NOT classify that shape as foundational ref identity behavior.

#### Scenario: Policy triggers unstructured full read

- **WHEN** navigation pre-dispatch selects unstructured full-read for a Markdown document
- **THEN** Markdown supplies the existing full content and cost facts through its declared capability
- **THEN** the result is not represented as heading entries
- **THEN** the capability does not generate or reinterpret Markdown refs

#### Scenario: Policy does not trigger

- **WHEN** unstructured full-read policy does not apply
- **THEN** Markdown uses normal structured outline behavior
- **THEN** compatible preparation from an earlier selected-adapter stage remains reusable

#### Scenario: Auxiliary full-read behavior fails

- **WHEN** cost or content evaluation returns an owner-compatible diagnostic
- **THEN** navigation preserves existing failure/fallback semantics
- **THEN** private Markdown state is released without a public cleanup or snapshot fact
