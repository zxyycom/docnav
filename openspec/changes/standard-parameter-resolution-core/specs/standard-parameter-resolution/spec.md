## ADDED Requirements

### Requirement: Standard parameter resolution produces typed values from constructed sources
The standard parameter resolver MUST combine constructed direct input, project config, user config, and default sources into typed runtime values using typed-field metadata, while preserving source information and diagnostics for the final standard parameter result.

#### Scenario: Direct input overrides lower-priority sources
- **WHEN** the same standard parameter identity is present in direct input, project config, user config, and default sources
- **THEN** the resolver returns the direct input value as the final typed value
- **THEN** the resolver records direct input as the source info for that final value

#### Scenario: Config sources override user config and default
- **WHEN** a standard parameter identity is absent from direct input and present in both project config and user config
- **THEN** the resolver returns the project config value as the final typed value
- **THEN** the resolver records project config as the source info for that final value

#### Scenario: Default source fills absent declared values
- **WHEN** a standard parameter identity has no mapped direct input, project config, or user config value
- **THEN** the resolver uses the declared static or dynamic default when one exists
- **THEN** the default result is validated through the same typed-field metadata as other mapped values

#### Scenario: Mapped invalid value fails standard parameter validation
- **WHEN** a mapped source value violates the declared typed-field kind, enum, range, requiredness, or default constraint
- **THEN** the resolver returns a standard parameter validation diagnostic for that identity
- **THEN** the invalid mapped value is not exposed as a safe typed runtime value

### Requirement: Source construction maps registered inputs
The standard parameter source layer MUST construct direct input, project config, user config, and default sources from standard parameter registration and typed-field extraction metadata before resolution.

#### Scenario: Config JSON maps through registered config binding
- **WHEN** a project or user config JSON object contains a value at a registered config path
- **THEN** source construction maps that value to the registered standard parameter identity
- **THEN** the source records project config or user config as its source kind

#### Scenario: Unregistered config field remains passthrough
- **WHEN** a config JSON object contains a field that is not mapped by any standard parameter registration
- **THEN** source construction does not validate it as a standard parameter
- **THEN** the field is retained, discarded, or delegated according to the entry passthrough policy

#### Scenario: Direct input maps through entry binding
- **WHEN** direct CLI input or adapter invoke arguments include a value mapped by the entry registration
- **THEN** source construction maps that value to the registered standard parameter identity as direct input
- **THEN** unmapped direct input remains outside standard parameter validation

#### Scenario: Default source includes static and dynamic defaults
- **WHEN** a registration has a static default or caller-provided dynamic default
- **THEN** source construction places that default into the default source
- **THEN** the default is validated through the same typed-field metadata as other mapped source values

### Requirement: Config source loading reports skipped sources without owning output
The standard parameter source layer MUST load configured project and user config sources, skip unavailable or invalid sources according to standard parameter rules, and return structured diagnostic data while leaving warning formatting, output channels, and exit behavior to the entry owner.

#### Scenario: Missing default config source is absent
- **WHEN** the default project or user config path does not exist
- **THEN** the config source is treated as absent
- **THEN** no skipped-source diagnostic is returned for that missing default source

#### Scenario: Invalid explicit config source is skipped
- **WHEN** an explicit project or user config override is missing, unreadable, invalid JSON, or not a JSON object
- **THEN** the config source is skipped
- **THEN** the result includes structured source-skipped diagnostic data with source level, path origin, path, and reason code
- **THEN** remaining available sources continue into standard parameter resolution

### Requirement: Standard parameter passthrough remains owner-scoped
The standard parameter resolver MUST leave unmapped input outside standard parameter validation and return passthrough according to the entry policy so that the owning CLI, adapter, protocol, or config layer can retain, discard, warn about, or validate it.

#### Scenario: Unmapped input remains outside standard parameter validation
- **WHEN** an input field is not mapped to a standard parameter identity
- **THEN** the standard parameter resolver does not validate it as a standard parameter
- **THEN** the resolver returns it through the entry policy as retained, discarded, or delegated passthrough

#### Scenario: Adapter native option remains delegated
- **WHEN** an adapter direct CLI or invoke argument includes a native option that has no standard parameter mapping
- **THEN** the resolver keeps that option outside typed-field standard parameter validation
- **THEN** the entry owner remains responsible for any native option validation or ignored-argument warning

### Requirement: Operation argument binding preserves source semantics
The standard parameter resolver MUST model operation argument binding as a mapping from standard parameter identity to protocol request `arguments` path while preserving the source info produced by resolution.

#### Scenario: Bound direct argument can be serialized to protocol arguments
- **WHEN** a direct input value is mapped to a standard parameter identity that has an operation argument binding
- **THEN** the binding identifies the protocol request `arguments` path for that direct value
- **THEN** the resolver preserves the direct source info for that value

#### Scenario: Config and default values preserve resolved source info
- **WHEN** a final standard parameter value comes from project config, user config, or default
- **THEN** operation argument binding preserves the resolved source info for that value
- **THEN** any decision to serialize or omit that value from a protocol request remains owned by the later request construction layer
