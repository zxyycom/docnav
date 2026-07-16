## MODIFIED Requirements

### Requirement: Core CLI selects output mode and process exit behavior

Core CLI MUST parse the document output modes `readable-view` and `protocol-json`, construct the corresponding output plan, and map diagnostics to process exit behavior without redefining protocol、operation result or renderer semantics. Omitted output or `readable-view` MUST construct `Rendered` with the built-in renderer supplied by core composition. `protocol-json` MUST construct `ProtocolJson` and bypass renderer invocation.

#### Scenario: Omitted output uses core composition

- **WHEN** a caller omits output mode for a valid document operation
- **THEN** core constructs `Rendered` with the built-in `readable-view` renderer

#### Scenario: Explicit readable-view selects rendered output

- **WHEN** a caller requests `--output readable-view`
- **THEN** core constructs `Rendered` with the built-in `readable-view` renderer
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
- **THEN** core leaves stdout empty and surfaces `output_render_failed` on stderr
- **THEN** core uses the mapped failure exit behavior
