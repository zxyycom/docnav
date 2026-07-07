本 spec delta 定义 `navigation-input-resolution` 的新增要求：navigation 必须把 config source validation 和 adapter-id native option validation 建立在同一份 owner-provided config metadata projection 上。

当前 change 只在 `openspec/changes/replace-config-commands-with-inspect/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Navigation exposes config metadata from typed fields

Navigation MUST expose a config metadata projection derived from common navigation typed fields, outline mode config fields, and adapter-id namespaced typed-field declarations. The projection MUST preserve config processing paths, field identity, owner, adapter id when applicable, value kind, constraints, defaults, compound shape metadata, and source binding facts without taking ownership of adapter-native option semantics or actual source attribution policy.

#### Scenario: Config metadata includes common fields

- **WHEN** navigation builds config metadata for document operation inputs
- **THEN** metadata for `defaults.pagination.enabled`, `defaults.pagination.limit`, `defaults.output`, and declared outline mode config fields is derived from the same field facts used by navigation resolution
- **THEN** consumers can validate config source values without redefining those field facts

#### Scenario: Config metadata includes adapter-id options

- **WHEN** navigation builds config metadata from the adapter registry
- **THEN** native option declarations are projected under `options.<adapter-id>.<option-key>`
- **THEN** equal option keys from different adapter ids remain distinct config paths

### Requirement: Config source validation uses config metadata

Navigation MUST validate config source keys and declared values through the config metadata projection before constructing operation arguments. Unknown fields, unknown adapter ids, undeclared `options.<adapter-id>.*`, invalid object/array shape, and typed validation failures MUST produce blocking diagnostics with config source attribution.

#### Scenario: Config native option fails selected declaration

- **WHEN** a project config file contains `options.markdown.max_heading_level` with a value outside the Markdown adapter declaration range
- **THEN** navigation reports a blocking typed validation diagnostic for that config source and field
- **THEN** adapter dispatch does not occur

#### Scenario: Config option unsupported by selected adapter

- **WHEN** a user config file contains an `options.markdown.*` key not declared by the selected Markdown adapter operation
- **THEN** navigation reports an unsupported native option diagnostic for that config source
- **THEN** the raw option value is not forwarded to the adapter handler

### Requirement: Navigation consumes selected adapter namespace

When navigation constructs operation arguments for a selected adapter and operation, it MUST consume native option values from that selected adapter's `options.<adapter-id>.*` namespace and validate them against the selected operation field set. Values stored under other adapter ids MUST NOT be forwarded to the selected adapter handler.

#### Scenario: Selected adapter reads its own namespace

- **WHEN** a config file contains `options.markdown.max_heading_level` and `options.other.max_heading_level`
- **AND** navigation selects adapter id `markdown` for an outline operation
- **THEN** navigation validates and consumes the value under `options.markdown.max_heading_level`
- **THEN** the value under `options.other.max_heading_level` is not forwarded to the Markdown adapter handler
