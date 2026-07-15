本 delta 的目标是让 core CLI 在 `readable-view` rendered path 与 `protocol-json` path 之间选择，并由代码组合提供 renderer；当前文档只在 `openspec/changes/unify-output-with-injected-rendering/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## MODIFIED Requirements

### Requirement: Core CLI selects output mode and process exit behavior

Core CLI MUST parse the document output modes `readable-view` and `protocol-json`, construct the corresponding output plan, and map diagnostics to process exit behavior without redefining protocol、operation result or renderer semantics. Omitted output or `readable-view` MUST construct `Rendered` with the renderer supplied by core composition. `protocol-json` MUST construct `ProtocolJson` and bypass renderer invocation.

#### Scenario: Omitted output uses core composition

- **WHEN** a caller omits output mode for a valid document operation
- **THEN** core constructs `Rendered` with the built-in `readable-view` renderer

#### Scenario: Explicit readable-view selects rendered output

- **WHEN** a caller requests `--output readable-view`
- **THEN** core constructs `Rendered` with its code-supplied renderer
- **THEN** the CLI value does not supply renderer implementation identity

#### Scenario: Protocol JSON bypasses rendering

- **WHEN** a caller requests `--output protocol-json`
- **THEN** core emits protocol stdout without invoking a renderer

#### Scenario: Document failure follows selected output plan

- **WHEN** a document operation fails after a valid output context exists
- **THEN** `ProtocolJson` emits the protocol failure envelope or `Rendered` invokes its renderer with the primary diagnostic
- **THEN** core uses the CLI exit mapping for the surfaced diagnostic class

#### Scenario: Render failure follows output mapping

- **WHEN** the selected renderer returns `RenderFailure`
- **THEN** core leaves stdout empty and surfaces the output-owned diagnostic on stderr
- **THEN** core uses the mapped failure exit behavior

### Requirement: Config surface is read-only inspect

Core CLI MUST expose `docnav config inspect` as the only long-term config subcommand. The command MUST NOT mutate config files, accept key/value edits, delete fields, or preserve single-key get/list editor semantics. Legacy `docnav config get`, `docnav config set`, `docnav config unset`, and `docnav config list` MUST be removed as accepted subcommands in this breaking change.

#### Scenario: Inspect selected config sources

- **WHEN** a caller runs `docnav config inspect`
- **THEN** core CLI obtains the selected project and user config source facts through the shared config source selection/loading primitives
- **THEN** the output includes each source's scope, path, origin, existence/load state, and source-attributed validation diagnostics or a bounded diagnostic summary when present
- **THEN** no config file is modified

#### Scenario: Reject legacy config mutation

- **WHEN** a caller runs `docnav config set defaults.output protocol-json`
- **THEN** core CLI rejects the subcommand through the normal CLI parse/error boundary
- **THEN** no config file is modified
