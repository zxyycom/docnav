本 spec delta 定义 `core-cli` 的新增要求：`docnav config` 必须收缩为单一只读 inspect surface，并通过 owner-provided config metadata 展示配置来源状态、source summary 和 source-attributed validation result。

当前 change 只在 `openspec/changes/replace-config-commands-with-inspect/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Config surface is read-only inspect

Core CLI MUST expose `docnav config inspect` as the only long-term config subcommand. The command MUST NOT mutate config files, accept key/value edits, delete fields, or preserve single-key get/list editor semantics. Legacy `docnav config get`, `docnav config set`, `docnav config unset`, and `docnav config list` MUST be removed as accepted subcommands in this breaking change.

#### Scenario: Config inspect reports selected sources

- **WHEN** a caller runs `docnav config inspect`
- **THEN** core CLI obtains the selected project and user config source facts through the same config source selection/loading boundary used by document operations
- **THEN** the output includes each source's scope, path, origin, existence/load state, and first validation issue when present
- **THEN** no config file is modified

#### Scenario: Legacy config mutators are not accepted

- **WHEN** a caller runs `docnav config set defaults.output readable-json`
- **THEN** core CLI rejects the subcommand through the normal CLI parse/error boundary
- **THEN** no config file is modified

### Requirement: Config inspect validates through owner metadata

Core CLI config inspection MUST validate config source keys and values through owner-provided config metadata for supported config fields. The inspection output MUST report invalid value kind, enum, range, nullability, object/array shape, adapter declaration, and owner-specific config constraint failures without reimplementing field semantics in core CLI.

#### Scenario: Inspect reports invalid typed value

- **WHEN** a selected config file contains `defaults.pagination.limit` with value `0`
- **THEN** `docnav config inspect` validates the value through owner-provided config metadata
- **THEN** the output identifies `defaults.pagination.limit`, the selected source path, and the typed validation reason

#### Scenario: Inspect reports nested shape failure

- **WHEN** a selected config file contains an invalid `outline.mode_rules[]` item shape
- **THEN** `docnav config inspect` validates the nested config source shape through owner-provided config metadata when expressible
- **THEN** the output identifies the nested config path and source path

### Requirement: Config inspect preserves adapter option ownership

Core CLI config inspection MUST treat `options.<adapter-id>.<option-key>` keys as adapter-owned native option sources. The adapter id segment MUST be resolved using the existing adapter registry id without aliases. Equal option keys from different adapter ids MUST remain distinct config paths. Bare `options.<option-key>` paths MUST NOT receive migration, compatibility, or special diagnostic behavior beyond the normal unknown/invalid config path handling.

#### Scenario: Adapter-id native option is inspected

- **WHEN** a selected config file contains `options.markdown.max_heading_level`
- **THEN** inspection resolves `markdown` through the adapter registry-backed metadata projection
- **THEN** inspection validates that value through the Markdown adapter option declaration when metadata is available

#### Scenario: Same option key in different adapters is deterministic

- **WHEN** selected config sources contain `options.markdown.mode` and `options.other.mode`
- **THEN** config inspection keeps both paths distinct
- **THEN** declarations from one adapter id do not validate or rewrite the other adapter id namespace

### Requirement: Config inspect remains source-scoped

Core CLI config inspection MUST remain a source inspection command. The command MUST report selected config sources, source summaries, load states, and source-attributed validation diagnostics. Adapter-id namespaced fields MAY appear as source fields validated through owner-provided metadata; selected adapter/operation argument construction remains owned by navigation input resolution.

#### Scenario: Adapter-id option appears as source field

- **WHEN** a selected config source contains `options.markdown.max_heading_level`
- **AND** a caller runs `docnav config inspect`
- **THEN** the output reports that path as a source field with validation facts when metadata is available
- **THEN** no config file is modified
