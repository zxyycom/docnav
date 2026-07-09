本 spec delta 定义 `core-cli` 的新增要求：`docnav config` 必须收缩为单一只读 inspect surface，并通过参数汇总产出的 config-source projection 展示配置来源状态、source summary、source-attributed validation result 和当前输入可解析出的参数事实。

本 spec delta 只拥有 core CLI 可观察行为：accepted config subcommands、inspect 是否只读、inspect 展示哪些 source facts、legacy command 如何被拒绝，以及 inspect 不成为 dispatch preview。参数汇总、adapter declaration 和 typed-field helper 的内部形状由对应 capability delta 拥有。

## ADDED Requirements

### Requirement: Config surface is read-only inspect

Core CLI MUST expose `docnav config inspect` as the only long-term config subcommand. The command MUST NOT mutate config files, accept key/value edits, delete fields, or preserve single-key get/list editor semantics. Legacy `docnav config get`, `docnav config set`, `docnav config unset`, and `docnav config list` MUST be removed as accepted subcommands in this breaking change.

#### Scenario: Config inspect reports selected sources

- **WHEN** a caller runs `docnav config inspect`
- **THEN** core CLI obtains the selected project and user config source facts through the shared config source selection/loading primitives
- **THEN** the output includes each source's scope, path, origin, existence/load state, and source-attributed validation diagnostics or a bounded diagnostic summary when present
- **THEN** no config file is modified

#### Scenario: Legacy config mutators are not accepted

- **WHEN** a caller runs `docnav config set defaults.output readable-json`
- **THEN** core CLI rejects the subcommand through the normal CLI parse/error boundary
- **THEN** no config file is modified

### Requirement: Config inspect validates through parameter aggregation metadata

Core CLI config inspection MUST validate config source keys and values through the config-source projection produced by owner-provided parameter aggregation metadata where that projection expresses the field. The inspection output MUST report invalid value kind, enum, range, nullability, adapter declaration, and owner-specific config constraint failures without reimplementing field semantics in core CLI. Object/array shape diagnostics for current config arrays MAY remain owner-specific when existing owner validation already preserves source path and parity with navigation resolution.

#### Scenario: Inspect reports invalid typed value

- **WHEN** a selected config file contains `defaults.pagination.limit` with value `0`
- **THEN** `docnav config inspect` validates the value through the config-source projection produced by parameter aggregation
- **THEN** the output identifies `defaults.pagination.limit`, the selected source path, and the typed validation reason

#### Scenario: Inspect reports nested shape failure

- **WHEN** a selected config file contains an invalid `outline.mode_rules[]` item shape
- **THEN** `docnav config inspect` validates the nested config source shape through the current owner validation path or config-source projection for that supported subset
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

#### Scenario: Bare native option path is a normal unknown path

- **WHEN** a selected config source contains `options.max_heading_level`
- **THEN** config inspection treats that path through the normal unknown/invalid config path handling
- **THEN** inspection does not infer an adapter id, rewrite the path, or apply migration behavior

### Requirement: Config inspect remains source-scoped

Core CLI config inspection MUST remain a source inspection command. The command MUST report selected config sources, source summaries, load states, source-attributed validation diagnostics, and currently resolvable parameter facts. Adapter-id namespaced fields MAY appear as source fields validated through parameter aggregation metadata; selected adapter/operation dispatch remains owned by navigation input resolution.

#### Scenario: Adapter-id option appears as source field

- **WHEN** a selected config source contains `options.markdown.max_heading_level`
- **AND** a caller runs `docnav config inspect`
- **THEN** the output reports that path as a source field with validation facts and currently resolvable parameter facts when metadata is available
- **THEN** no config file is modified

#### Scenario: Inspect does not preview dispatch

- **WHEN** a selected config source contains `options.markdown.max_heading_level`
- **AND** a caller runs `docnav config inspect` without invoking a document operation
- **THEN** the output reports source validation facts and any parameter facts currently resolvable from the selected sources
- **THEN** the output does not claim that the value was dispatched to an adapter
