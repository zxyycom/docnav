本 delta spec 是 `separate-entry-pipeline-from-parameter-resolution` 的未审核临时文档，目标是把 navigation entry lifecycle 与 navigation input resolution 分开；当前 change 只在 `openspec/changes/separate-entry-pipeline-from-parameter-resolution/` 下形成临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Navigation input resolution has no non-navigation lifecycle ownership
Navigation input resolution MUST own source construction, source priority, typed validation/extraction, source info, diagnostic handoff, explicit adapter native option source handling, and operation argument binding metadata for navigation commands. Entry lifecycle classification, transport decode, command help, non-navigation command behavior, output mode rendering and exit code policy MUST remain with entry owners.

#### Scenario: Navigation receives input views from core
- **WHEN** core CLI has classified an invocation as a navigation command
- **THEN** core provides raw command, config source descriptors/paths and registry to `docnav-navigation`
- **THEN** navigation input resolution returns typed operation values, source info, diagnostics and binding metadata
- **THEN** core remains responsible for output projection and exit behavior

#### Scenario: Resolver is not used as global entry dispatcher
- **WHEN** an invocation is help, version, init, doctor, config without document context, or static adapter inspection
- **THEN** the invocation does not enter navigation input resolution
- **THEN** the invocation keeps its own owner-defined parsing and output boundary

### Requirement: Configuration source merge is a navigation source subflow
The configuration source merge channel MUST be defined as the project/user config source subflow inside navigation input resolution. It MUST load config sources from core-provided descriptors/paths, validate that available config roots are JSON objects, project registered config paths into source values, surface invalid config-source diagnostic handoffs, and contribute values to the shared source priority order. Direct input mapping, defaults, typed validation/extraction and operation argument binding MUST remain separate parts of navigation input resolution.

#### Scenario: Config merge contributes only config sources
- **WHEN** project and user config sources contain registered config paths
- **THEN** the configuration source merge channel maps those paths to project_config and user_config source values
- **THEN** direct input and default values are provided by separate source subflows
- **THEN** final typed runtime values are produced by navigation input resolution, not by config source merge alone

#### Scenario: Invalid config source produces blocking handoff
- **WHEN** an explicit project config override is missing, unreadable, invalid JSON, or not a JSON object
- **THEN** the configuration source merge channel records that config source issue
- **THEN** navigation input resolution exposes it as a blocking config-source diagnostic handoff
- **THEN** other available sources do not make the invalid config source recoverable

### Requirement: Navigation input resolution keeps raw inputs immutable
Navigation input resolution MUST NOT mutate raw CLI argv tokens, raw decoded stdin JSON, protocol request envelopes, request `arguments`, or caller-owned config objects. Any normalized, supplemented, or finalized value MUST be represented as a derived typed runtime value with source info.

#### Scenario: Defaults are derived values
- **WHEN** a registered field has a static or dynamic default
- **AND** no direct input, project config, or user config value exists for that field
- **THEN** navigation input resolution may return a derived typed runtime value from default
- **THEN** no raw input object is modified to include that default

#### Scenario: Adapter native options require selected adapter declarations
- **WHEN** the selected adapter declares native option source descriptors for a document operation
- **THEN** navigation input resolution may project only those declared keys as adapter-native source values
- **THEN** undeclared option keys produce blocking input diagnostics

#### Scenario: Unmapped public input does not become implicit passthrough
- **WHEN** direct input contains public fields that are not mapped to a declared parameter, registered config path, or selected adapter native option source
- **THEN** navigation input resolution returns an unmapped-input diagnostic
- **THEN** core reports a blocking input diagnostic instead of delegating the input implicitly
- **THEN** resolution does not delete or rewrite the caller-owned raw input
