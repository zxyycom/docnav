## ADDED Requirements

### Requirement: Field definitions preserve owner-authored CLI processing metadata

Typed field declarations MUST allow the field owner to associate optional framework-neutral CLI input metadata with a CLI flag processing strategy. The built `FieldDef` and `FieldDefSet` processing projection MUST preserve authored help、value name and Boolean input encoding beside the canonical identity、flag locator、value kind、constraints、default and merge facts. Accepted values、default values、constraints and merge semantics MUST remain canonical field facts.

The field owner MUST author CLI presentation/capture facts; typed-fields MUST only validate、preserve and project them. Typed-fields MUST remain independent of Clap and consumer policies such as command topology、operation applicability、public diagnostics and output behavior. Field build or projection MUST deterministically reject CLI metadata attached to a non-CLI processing strategy、Boolean encoding incompatible with the canonical value kind、duplicate metadata for the same field/processing id or an incomplete token mapping.

#### Scenario: Preserve CLI facts through field-set aggregation

- **WHEN** a common or adapter-owned field declaration attaches help、value name and a compatible CLI input encoding to its CLI flag processing strategy
- **THEN** the built field-set projection returns those authored facts beside the canonical field metadata
- **THEN** declaration type erasure、builder cloning and field-set aggregation preserve one authoring source

#### Scenario: Config-only field carries no CLI metadata

- **WHEN** a field declares config-path processing without a public CLI flag
- **THEN** it can build without CLI input metadata
- **THEN** CLI projection treats the field as non-applicable

#### Scenario: Reject incompatible Boolean encoding

- **WHEN** a non-Boolean field declares a Boolean switch or token mapping、or a Boolean token mapping omits a deterministic true/false interpretation
- **THEN** field construction or CLI processing projection fails deterministically
- **THEN** no CLI candidate is produced from the invalid declaration
