## ADDED Requirements

### Requirement: Field definitions carry immutable typed consumer extensions

Typed fields MUST let consumers attach immutable extension metadata to a field declaration by Rust payload type and retrieve it from the built `FieldDef` through that same type. Extension payloads MUST satisfy `Send + Sync + 'static`. Extension metadata MUST survive builder cloning、declaration type erasure、field build and definition-set aggregation without changing canonical field identity、processing、validation、default or merge semantics.

Each field MUST accept at most one payload for a given extension type. Duplicate attachment of the same type MUST fail deterministically during field build; lookup of an absent type MUST return absence. Built definitions and their clones MUST share immutable extension payloads rather than expose mutation or replacement. Typed fields MUST NOT define string extension keys、set/replace behavior or a second `FieldDefSet` lookup API in this change.

Typed fields MUST treat extension payloads as consumer-owned opaque values. The core MUST NOT interpret Docnav、Clap、operation、adapter or presentation meaning.

#### Scenario: Project builder attaches typed metadata

- **WHEN** a consumer defines a project-specific field builder extension and attaches its metadata to a canonical field
- **THEN** the built field preserves that metadata beside its canonical facts
- **THEN** the consumer retrieves it through the same payload type and derives its project projection

#### Scenario: Extension survives declaration aggregation

- **WHEN** an extended field builder passes through `FieldDefDeclaration` type erasure and `FieldDefSet` construction
- **THEN** typed retrieval from the aggregated field returns the same immutable consumer payload
- **THEN** canonical schema and processing projections remain unchanged

#### Scenario: Duplicate extension type is rejected

- **WHEN** the same extension payload type is attached twice to one field builder
- **THEN** field construction returns a deterministic duplicate-extension failure
- **THEN** the builder does not choose one payload implicitly

#### Scenario: Missing extension is distinguishable

- **WHEN** a consumer queries a field that does not carry the requested extension type
- **THEN** retrieval reports absence without inventing a default payload
- **THEN** the consumer decides whether the field is non-applicable or the project projection is structurally invalid

#### Scenario: Clone shares immutable metadata

- **WHEN** a field definition or declaration containing an extension is cloned
- **THEN** each clone retrieves the same immutable payload value
- **THEN** no clone exposes a mutation or replacement path

#### Scenario: Typed-fields does not own CLI semantics

- **WHEN** a Docnav extension payload contains help or Boolean presentation facts
- **THEN** typed-fields stores and returns the opaque value
- **THEN** Docnav-owned builder and projection code interpret those facts
