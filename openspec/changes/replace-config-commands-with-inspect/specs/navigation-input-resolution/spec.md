本 spec delta 定义 `navigation-input-resolution` 的新增要求：navigation 必须把 config source validation 和 adapter-id native option validation 建立在同一份 owner-provided parameter aggregation metadata 上，并产出 CLI/input 与 config-source 两类 projection。

本 spec delta 只拥有 navigation input resolution 的边界：如何消费参数汇总 projection、如何验证 selected operation 需要的 config values、如何读取 selected adapter namespace，以及其它 adapter namespace 不进入 selected handler。`config inspect` 的输出 shape 由 `core-cli` delta 拥有；adapter declaration 事实源由 `adapter-contract` delta 拥有。

## ADDED Requirements

### Requirement: Navigation exposes parameter aggregation projections

Navigation MUST participate in a parameter aggregation boundary derived from common navigation typed fields, outline mode config fields, and adapter-id namespaced typed-field declarations. The aggregation MUST preserve processing paths, field identity, owner, adapter id when applicable, value kind, constraints, defaults, current owner-specific shape validation handoff when applicable, and source binding facts, and MUST be able to produce CLI/input and config-source projections without taking ownership of adapter-native option semantics or actual source attribution policy.

#### Scenario: Config-source projection includes common fields

- **WHEN** navigation builds the config-source projection for document operation inputs
- **THEN** metadata for `defaults.pagination.enabled`, `defaults.pagination.limit`, `defaults.output`, and declared outline mode config fields is derived from the same field facts used by navigation resolution
- **THEN** consumers can validate config source values without redefining those field facts

#### Scenario: Config-source projection includes adapter-id options

- **WHEN** navigation builds config-source metadata from the adapter registry
- **THEN** native option declarations are projected under `options.<adapter-id>.<option-key>`
- **THEN** equal option keys from different adapter ids remain distinct config paths

### Requirement: Config source validation uses the config-source projection

Navigation MUST validate config source keys and declared values through the config-source projection before constructing operation arguments when those fields are consumed for the selected operation. Unknown fields, unknown adapter ids, selected-adapter `options.<adapter-id>.*` fields not declared for the selected operation, owner-specific object/array shape failures in the supported subset, and typed validation failures MUST produce blocking diagnostics with config source attribution. Known adapter-id namespaces for adapters other than the selected adapter MAY remain valid source facts but MUST NOT affect selected operation argument construction.

#### Scenario: Config native option fails selected declaration

- **WHEN** a project config file contains `options.markdown.max_heading_level` with a value outside the Markdown adapter declaration range
- **THEN** navigation reports a blocking typed validation diagnostic for that config source and field
- **THEN** adapter dispatch does not occur

#### Scenario: Config option unsupported by selected adapter

- **WHEN** a user config file contains an `options.markdown.*` key not declared by the selected Markdown adapter operation
- **THEN** navigation reports an unsupported native option diagnostic for that config source
- **THEN** the raw option value is not forwarded to the adapter handler

#### Scenario: Unknown adapter namespace is blocking

- **WHEN** a config source contains `options.unknown_adapter.max_heading_level`
- **THEN** navigation reports an unknown adapter id diagnostic with config source attribution
- **THEN** adapter dispatch does not occur

### Requirement: Navigation consumes selected adapter namespace

When navigation constructs operation arguments for a selected adapter and operation, it MUST consume native option values from that selected adapter's `options.<adapter-id>.*` namespace and validate them against the selected operation field set. Values stored under other known adapter ids MUST remain separate source facts and MUST NOT be forwarded to the selected adapter handler.

#### Scenario: Selected adapter reads its own namespace

- **WHEN** a config file contains `options.markdown.max_heading_level` and `options.other.max_heading_level`
- **AND** navigation selects adapter id `markdown` for an outline operation
- **THEN** navigation validates and consumes the value under `options.markdown.max_heading_level`
- **THEN** the value under `options.other.max_heading_level` is not forwarded to the Markdown adapter handler or used to validate the Markdown selected operation
