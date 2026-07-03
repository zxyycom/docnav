# navigation-input-resolution Specification

## Purpose
定义 document navigation input resolution 的长期契约：`docnav` core 只解析命令类型，并为 navigation command 提供 config source descriptors/paths 与 registry；raw project/user config source loading、routing 输入解析、adapter selection、selected adapter typed-field 参数声明读取、来源优先级、typed validation/extraction、operation argument binding、request construction 和 adapter dispatch 由 `docnav-navigation` 拥有。

## Requirements
### Requirement: Core only hands raw navigation inputs to the navigation layer
`docnav` core MUST classify each invocation before navigation input resolution. For non-navigation commands, core MUST keep the owner-defined command behavior in core. For navigation commands, core MUST pass the raw command, project/user config source descriptors/paths and current static adapter registry to `docnav-navigation`.

#### Scenario: Navigation command enters input resolution
- **WHEN** caller executes `docnav outline docs/guide.md`
- **THEN** core classifies the invocation as a navigation command
- **THEN** core supplies project/user config source descriptors/paths
- **THEN** core passes raw command, config source descriptors/paths and registry to `docnav-navigation`
- **THEN** `docnav-navigation` loads raw project/user config sources

#### Scenario: Non-navigation command stays in core
- **WHEN** caller executes help, version, init, doctor, config without document context, or static adapter inspection
- **THEN** core handles that command in its owner boundary
- **THEN** the command does not enter navigation input resolution

### Requirement: Navigation input resolution selects the adapter before parameter extraction
`docnav-navigation` MUST parse routing-required input, select the adapter from the current static registry, then read only the selected adapter's typed-field parameter declarations for source resolution.

#### Scenario: Declared adapter selects one registry entry
- **WHEN** direct input or config source declares an adapter id
- **THEN** `docnav-navigation` looks up that id in the current static registry
- **THEN** selection succeeds only if the selected adapter probe accepts the document
- **THEN** failure returns adapter selection diagnostics without falling back to later registry entries

#### Scenario: Automatic discovery uses registry order
- **WHEN** no declared adapter id exists
- **THEN** `docnav-navigation` traverses static registry entries in release order
- **THEN** the first adapter probe returning `supported: true` selects that adapter
- **THEN** later adapters are not part of the selected parameter declaration set

### Requirement: Selected adapter typed-field declarations own parameter facts
Selected adapter typed-field declarations MUST provide parameter identity, owner, namespace/key, operation applicability, source mappings, default metadata, value kind, requiredness/nullability, constraints and operation argument binding metadata. `docnav-navigation` MUST use those declarations as the parameter fact source.

#### Scenario: Typed-field metadata drives validation
- **WHEN** selected Markdown declares `options.max_heading_level` with integer range `1..6`
- **AND** a config or direct source provides `options.max_heading_level`
- **THEN** navigation input resolution validates and extracts the typed value before request construction
- **THEN** the Markdown handler receives the typed value rather than the raw source value

#### Scenario: Same option key can have different owners
- **WHEN** multiple selected-adapter declarations expose native option sources with the same public key across owners or type variants
- **THEN** navigation input resolution preserves owner, namespace, key and type variant metadata
- **THEN** it does not collapse those declarations into one core-owned parameter

### Requirement: Source priority is explicit over project over user over built-in
Navigation input resolution MUST combine available source candidates with fixed priority `explicit > project > user > built_in`. Missing default config sources MUST be absent. Present invalid config sources MUST produce blocking diagnostics and MUST NOT be recovered by lower-priority sources.

#### Scenario: Direct input wins
- **WHEN** the same parameter has explicit input, project config, user config and built-in default candidates
- **THEN** navigation input resolution uses the explicit value
- **THEN** the resolved source info records explicit input

#### Scenario: Project config wins over user config
- **WHEN** a parameter has project config and user config candidates but no explicit value
- **THEN** navigation input resolution uses the project config value
- **THEN** the resolved source info records project config

#### Scenario: Built-in default fills absence
- **WHEN** a declared parameter has no explicit, project or user source
- **AND** the selected adapter declaration or navigation default provides a built-in default
- **THEN** navigation input resolution validates and uses that default

### Requirement: Config sources are source inputs, not the owner
Project and user config files MUST be modeled as raw source inputs loaded by `docnav-navigation` from core-supplied descriptors/paths. Config shape, known config paths and unmapped-field behavior MUST be interpreted through selected declarations and source mapping rules; config files MUST NOT own final parameter resolution, request construction or adapter option validation.

#### Scenario: Config JSON maps to declared paths
- **WHEN** project config contains a value at a selected declaration's config path
- **THEN** navigation input resolution maps that value to the declared parameter identity
- **THEN** validation and extraction use the same typed-field declaration as explicit input

#### Scenario: Unknown config field is blocking input
- **WHEN** present config contains an unknown or unmapped public field
- **THEN** navigation input resolution returns a source-scoped blocking diagnostic
- **THEN** it does not silently pass the field to an adapter handler

### Requirement: Request construction consumes typed resolution results
`docnav-navigation` MUST bind resolved typed values into `OperationArguments`, construct a `RequestEnvelope`, and dispatch the selected adapter operation handler. Request construction MUST NOT mutate raw command input, raw config source objects or protocol examples.

#### Scenario: Read arguments are bound from typed values
- **WHEN** selected input resolution has typed `ref`, `limit`, `page` and `options` values for `read`
- **THEN** request construction writes those values to read `OperationArguments`
- **THEN** source info remains available for diagnostics and audit

#### Scenario: Disabled pagination finalizes before dispatch
- **WHEN** resolved `pagination.enabled` is false
- **THEN** navigation input resolution finalizes the effective limit before request construction
- **THEN** the final value is not written back to raw argv or config source JSON

### Requirement: Diagnostics preserve owner and source
Navigation input resolution MUST report missing required values, unmapped public input, config source failures, unsupported selected-adapter options, type mismatch, range invalid and operation-inapplicable parameters as blocking diagnostics with source attribution.

#### Scenario: Invalid native option is rejected before handler dispatch
- **WHEN** selected adapter declarations reject a native option due to unsupported operation, wrong type or range invalid
- **THEN** navigation input resolution returns a diagnostic with selected adapter/source metadata
- **THEN** the adapter handler is not called with that invalid raw value

#### Scenario: Adapter selection precedes option validation
- **WHEN** declared adapter id is missing from the static registry
- **AND** the same request contains an invalid-looking native option
- **THEN** navigation input resolution returns adapter selection diagnostics
- **THEN** option validation for that adapter does not run
