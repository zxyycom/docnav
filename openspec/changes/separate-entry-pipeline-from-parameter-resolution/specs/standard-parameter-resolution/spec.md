本 delta spec 是 `separate-entry-pipeline-from-parameter-resolution` 的未审核临时文档，目标是把现有标准参数解析重新限定为入口参数来源解析；当前 change 只在 `openspec/changes/separate-entry-pipeline-from-parameter-resolution/` 下形成临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Entry parameter source resolution has no lifecycle ownership
The parameter resolver MUST be documented and implemented as entry parameter source resolution. It MUST own source construction, source priority, typed validation, source info, diagnostic handoff, passthrough handoff, and operation argument binding metadata. Entry lifecycle classification, transport decode, command help, handler dispatch, request construction policy, output mode rendering, and exit code policy MUST remain with entry owners.

#### Scenario: Resolver consumes input views from entry owners
- **WHEN** core CLI or adapter SDK has classified an invocation as a document operation or adapter invoke
- **THEN** the entry owner provides a direct input view and configured source descriptors to entry parameter source resolution
- **THEN** the resolver returns derived typed runtime values, source info, diagnostics, and passthrough handoff
- **THEN** the entry owner remains responsible for request construction, output projection, and exit behavior

#### Scenario: Resolver is not used as global entry dispatcher
- **WHEN** an invocation is help, manifest, probe, version, init, doctor, config without document context, or adapter management
- **THEN** the standard entry pipeline does not require that invocation to enter parameter source resolution
- **THEN** the invocation keeps its own owner-defined parsing and output boundary

### Requirement: Configuration source merge channel is a parameter-source subflow
The configuration source merge channel MUST be defined as the project/user config source subflow inside entry parameter source resolution. It MUST read or receive loaded config sources, validate that available config roots are JSON objects, project registered config paths into source values, preserve skipped-source diagnostics, and contribute values to the shared source priority order. Direct input mapping, defaults, typed validation, passthrough policy, and operation argument binding MUST remain separate parts of entry parameter source resolution.

#### Scenario: Config merge contributes only config sources
- **WHEN** project and user config sources contain registered config paths
- **THEN** the configuration source merge channel maps those paths to project_config and user_config source values
- **THEN** direct input and default values are provided by separate parameter source subflows
- **THEN** final typed runtime values are produced by entry parameter source resolution, not by config source merge alone

#### Scenario: Skipped config source remains recoverable handoff
- **WHEN** an explicit project config override is missing, unreadable, invalid JSON, or not a JSON object
- **THEN** the configuration source merge channel skips that config source
- **THEN** it produces a recoverable skipped-source diagnostic handoff
- **THEN** other available sources continue into entry parameter source resolution

### Requirement: Parameter source resolution keeps raw inputs immutable
Entry parameter source resolution MUST NOT mutate raw CLI argv tokens, raw decoded stdin JSON, protocol request envelopes, request `arguments`, or caller-owned config objects. Any normalized, supplemented, or finalized value MUST be represented as a derived typed runtime value with source info.

#### Scenario: Defaults are derived values
- **WHEN** a registered field has a static or dynamic default
- **AND** no direct input, project config, or user config value exists for that field
- **THEN** entry parameter source resolution may return a derived typed runtime value from default
- **THEN** no raw input object is modified to include that default

#### Scenario: Passthrough does not rewrite caller input
- **WHEN** direct input contains unmapped fields
- **THEN** entry parameter source resolution leaves those fields outside typed validation
- **THEN** passthrough handoff may describe retained, discarded, or delegated input
- **THEN** the resolver does not delete or rewrite the caller-owned raw input
