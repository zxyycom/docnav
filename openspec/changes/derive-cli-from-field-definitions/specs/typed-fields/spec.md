## ADDED Requirements

### Requirement: Field definitions carry opaque consumer extension metadata

Typed fields MUST let consumers attach opaque extension metadata to a field declaration and retrieve it from the built `FieldDef` / `FieldDefSet` by a deterministic extension key or type. Extension metadata MUST survive builder cloning、declaration type erasure、field build and definition-set aggregation without changing canonical field identity、processing、validation、default or merge semantics.

Typed fields MUST treat extension payloads as consumer-owned values. The core MUST NOT interpret Docnav、Clap、operation、adapter or presentation meaning. It MUST distinguish missing metadata from present metadata, reject duplicate declaration deterministically, and provide an explicit set/replace path when a consumer intentionally updates an extension.

#### Scenario: Project builder attaches metadata

- **WHEN** a consumer defines a project-specific field builder extension and attaches its metadata to a canonical field
- **THEN** the built field set preserves that metadata beside the field
- **THEN** the consumer can derive its project projection from the extended field definition

#### Scenario: Extension survives declaration aggregation

- **WHEN** an extended field builder passes through `FieldDefDeclaration` type erasure and `FieldDefSet` construction
- **THEN** typed retrieval returns the same consumer payload for that field
- **THEN** canonical schema and processing projections remain unchanged

#### Scenario: Duplicate and replacement are explicit

- **WHEN** the same extension key is declared twice without using the update API
- **THEN** field construction returns a deterministic duplicate-extension failure
- **THEN** a consumer that intentionally updates metadata uses the explicit set/replace path

#### Scenario: Missing extension is distinguishable

- **WHEN** a consumer queries a field that does not carry its extension
- **THEN** retrieval reports absence without inventing a default payload
- **THEN** the consumer decides whether the field is non-applicable or the project projection is structurally invalid

#### Scenario: Typed-fields does not own CLI semantics

- **WHEN** a Docnav extension payload contains help or Boolean presentation facts
- **THEN** typed-fields stores and returns the opaque value
- **THEN** Docnav-owned builder and projection code interpret those facts
