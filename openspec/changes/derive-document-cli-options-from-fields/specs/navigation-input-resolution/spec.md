## MODIFIED Requirements

### Requirement: Core hands raw navigation inputs to navigation

Core CLI MUST hand document operation facts、fixed positional and normalized path facts、config source descriptors/paths、the static adapter registry and normalized typed/invalid document CLI candidates to navigation. Adapter-native CLI input MUST cross this boundary as a candidate carrying canonical field identity、locator and source attribution. Navigation remains responsible for routing、selected resolution and request construction.

#### Scenario: Outline handoff

- **WHEN** core structurally parses `docnav outline <path>` and its generated document options
- **THEN** it identifies the operation、path facts and canonical candidate identities
- **THEN** navigation receives the input package for routing、selected resolution and request construction

#### Scenario: Native option handoff is normalized

- **WHEN** structural parsing captures a registry-declared adapter native option
- **THEN** navigation receives a typed or invalid candidate with canonical field identity、locator and source attribution
- **THEN** selected applicability can be checked by field identity

### Requirement: Navigation selects adapter before adapter parameter extraction

Navigation MUST select the adapter from routing inputs and registry facts before extracting any adapter-owned candidate into selected operation parameters. The operation-scoped registry CLI projection MAY normalize declared native flags before selection so that explicit input retains canonical field identity, but those candidates MUST remain unselected facts until navigation compares them with the selected adapter/current-operation declarations.

#### Scenario: Multiple adapters exist

- **WHEN** the registry contains multiple candidate adapters and structural parsing captures operation-applicable declared flags
- **THEN** navigation selects the adapter according to the existing selection rules
- **THEN** only the selected adapter/current-operation declarations contribute candidates to selected resolution
- **THEN** a supplied candidate outside that set follows the strict unsupported/unused input rule

#### Scenario: Selected definition provides capability facts

- **WHEN** navigation has selected an adapter
- **THEN** navigation reads optional capability declarations from the selected adapter definition
- **THEN** pre-dispatch policy uses only those declared support facts

#### Scenario: Navigation receives selected definition as the fact source

- **WHEN** core registry returns a selected adapter entry
- **THEN** the selected entry provides the adapter definition used for declaration registration、full-read pre-dispatch and dispatch
- **THEN** navigation uses definition-provided adapter-owned native option and capability semantics

### Requirement: Navigation exposes parameter aggregation projections

Navigation MUST participate in a parameter aggregation boundary derived from common navigation typed fields、outline mode config fields and adapter-id namespaced typed-field declarations. The aggregation MUST preserve processing paths、field identity、owner、adapter id when applicable、value kind、constraints、defaults、owner-specific shape validation handoff when applicable、source binding facts and owner-authored CLI processing metadata. Adapter-native semantics and actual source attribution remain with their existing owners.

For each document operation, navigation MUST produce an operation-scoped registry CLI field set from applicable common declarations and all registry adapter declarations applicable to that operation. After adapter selection, navigation MUST produce a selected resolution field set from current-operation common declarations and the selected adapter/current-operation declarations. Config-source projections MUST retain their existing adapter-id namespace and validation behavior.

#### Scenario: CLI projection includes operation-applicable registry fields

- **WHEN** navigation builds the CLI field set for a document operation
- **THEN** it contains applicable common fields and every registry native field declared for that operation
- **THEN** each projected flag maps to one canonical field identity before argv parsing
- **THEN** duplicate locators across common fields or registry adapters and other declaration conflicts fail deterministically with owner attribution

#### Scenario: Selected field set reuses the same declarations

- **WHEN** navigation selects an adapter after structural CLI parsing
- **THEN** the selected resolution set is built from the same common/native declarations used by the registry CLI set
- **THEN** it preserves identity、locator、value kind、constraints、default and operation applicability

#### Scenario: Config-source projection includes common fields

- **WHEN** navigation builds the config-source projection for document operation inputs
- **THEN** metadata for `defaults.pagination.enabled`、`defaults.pagination.limit`、`defaults.output` and declared outline mode config fields is derived from the same field facts used by navigation resolution
- **THEN** consumers can validate config source values without redefining those field facts

#### Scenario: Config-source projection includes adapter-id options

- **WHEN** navigation builds config-source metadata from the adapter registry
- **THEN** native option declarations are projected under `options.<adapter-id>.<option-key>`
- **THEN** equal option keys from different adapter ids remain distinct config paths

## ADDED Requirements

### Requirement: Explicit document option candidates obey selected declarations

After adapter selection, navigation MUST compare invocation-local explicit CLI candidate identities with the selected adapter/current-operation `FieldDefSet`. Candidates present in the selected set MUST enter existing source priority、merge、canonical validation and typed materialization. Any supplied explicit candidate absent from the selected set MUST produce a strict unsupported/unused caller-input diagnostic before request construction or dispatch. Adapter selection MUST continue to use routing inputs rather than infer an adapter from a native-option candidate.

This strict explicit-input rule MUST NOT change config-source behavior. Values under other known adapter-id config namespaces MAY remain valid independent source facts under the existing config contract and MUST NOT affect selected operation arguments.

#### Scenario: Resolve a selected explicit candidate

- **WHEN** a supplied document option belongs to the selected adapter and current operation
- **THEN** navigation passes its typed/invalid candidate into canonical resolution with explicit source priority
- **THEN** the resolved typed value can participate in request construction and handler handoff

#### Scenario: Reject an option owned by another adapter

- **WHEN** structural parsing captures a known registry flag but adapter selection chooses an adapter whose current-operation field set does not contain that field identity
- **THEN** navigation reports a strict unsupported/unused caller-input diagnostic
- **THEN** request construction and dispatch do not run

#### Scenario: Selected invalid candidate blocks canonically

- **WHEN** a selected `Replace` winner or required merge contributor carries an invalid decoded value or violates canonical constraints
- **THEN** existing canonical resolution reports the attributed field failure
- **THEN** materialization returns no operation arguments

#### Scenario: Other adapter config namespace remains independent

- **WHEN** a loaded config source contains facts for another known adapter id while the selected adapter inputs are valid
- **THEN** those facts retain the existing config-source treatment
- **THEN** they are not forwarded to the selected adapter and the explicit CLI strictness rule does not reclassify them
