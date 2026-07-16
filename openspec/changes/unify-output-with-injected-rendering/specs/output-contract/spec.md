## RENAMED Requirements

- FROM: `Readable-view and readable-json share one payload source`
- TO: `Rendered output consumes completed document outcomes`
- FROM: `Renderer config owns readable-view framing`
- TO: `Selected renderer owns presentation text`

## MODIFIED Requirements

### Requirement: Document output modes are fixed

Document operations MUST expose exactly the output modes `readable-view` and `protocol-json`. Core CLI MUST map omitted output and `readable-view` to `Rendered(RenderStrategy)` with the built-in `readable-view` renderer. A direct linked caller MAY construct `Rendered` with a custom renderer without adding another public output value. `protocol-json` MUST construct `ProtocolJson` and bypass rendering.

#### Scenario: Omitted output uses the default renderer

- **WHEN** a caller omits output mode for a document operation
- **THEN** Docnav selects the rendered path
- **THEN** core composition supplies the built-in `readable-view` renderer

#### Scenario: Explicit readable-view selects rendered output

- **WHEN** a caller requests `readable-view`
- **THEN** core composition supplies a `Rendered` plan with the built-in `readable-view` renderer

#### Scenario: Protocol JSON bypasses rendering

- **WHEN** a caller requests `protocol-json`
- **THEN** stdout contains the raw protocol envelope
- **THEN** no renderer is invoked

### Requirement: Rendered output consumes completed document outcomes

`RenderInput` MUST contain one completed typed operation success outcome or one primary `DiagnosticRecord` plus immutable `RenderContext`. It MUST preserve adapter-owned facts、operation status、ref and pagination semantics. Renderer-only helper views MUST remain private code contracts without a public serialized shape or schema.

#### Scenario: Render a successful operation

- **WHEN** a document operation produces a successful typed outcome
- **THEN** the selected renderer receives that outcome and immutable render context

#### Scenario: Render a primary diagnostic

- **WHEN** a document operation produces one primary `DiagnosticRecord` after rendered output is selected
- **THEN** the selected renderer receives the same diagnostic identity and canonical details
- **THEN** process exit behavior remains derived from the diagnostic class

### Requirement: Selected renderer owns presentation text

The selected renderer MUST return one complete UTF-8 text value or `RenderFailure` before process output is written. Output orchestration MUST write a successful value exactly as returned. The built-in `readable-view` renderer MUST apply its repository-owned framing and conformance rules; a custom renderer MUST define its own presentation contract.

#### Scenario: Built-in renderer applies readable-view framing

- **WHEN** the built-in renderer emits a configured content block
- **THEN** its block delimiters and field path follow readable-view configuration

#### Scenario: Custom renderer controls its text

- **WHEN** linked code supplies a custom renderer and rendering succeeds
- **THEN** stdout contains exactly the returned UTF-8 text

### Requirement: Output orchestration is above rendering

Document output orchestration MUST choose the output path、invoke the selected renderer for `Rendered`、serialize the protocol envelope for `ProtocolJson`、control stdout/stderr and map render failure before writing process output. A selected custom renderer MUST remain the only presentation owner for that invocation. A returned `RenderFailure` MUST leave stdout empty, surface as the output-owned `output_render_failed` diagnostic through the CLI internal-failure mapping, and MUST NOT trigger another renderer.

#### Scenario: Protocol output remains independent

- **WHEN** a document outcome is emitted through `ProtocolJson`
- **THEN** stdout contains one protocol response or failure envelope
- **THEN** renderer availability and behavior have no effect

#### Scenario: Rendered output commits one complete value

- **WHEN** a selected renderer succeeds
- **THEN** output orchestration writes its complete value after in-memory rendering finishes

#### Scenario: Custom renderer fails

- **WHEN** a custom renderer returns `RenderFailure`
- **THEN** stdout remains empty
- **THEN** output orchestration reports `output_render_failed`
- **THEN** no second renderer is invoked

#### Scenario: Non-document output remains owner-specific

- **WHEN** `docnav` or an adapter emits help、version、manifest or probe output
- **THEN** that owner keeps its existing mode and framing

### Requirement: Readable output supports unstructured outline content

The built-in `readable-view` renderer MUST represent declared unstructured outline full-read results as content, not as outline entries or pagination state. A custom renderer consumes the same immutable outcome under its own presentation contract.

#### Scenario: Built-in renderer emits unstructured content

- **WHEN** outline returns unstructured full-read content and the built-in renderer is selected
- **THEN** readable-view emits the content through its normal block framing
- **THEN** it does not invent outline entries or pagination state

## ADDED Requirements

### Requirement: Renderer dependency is resolved by linked code

Each `Rendered` plan MUST contain one renderer function or trait value selected by linked code before output orchestration begins. Public CLI and configuration inputs MUST select only documented output modes; core CLI `readable-view` MUST use the built-in renderer. A direct linked caller MAY supply a custom renderer through the shared output API, and its implementation identity MUST remain outside public input and serialized metadata contracts.

#### Scenario: Public input selects the built-in renderer

- **WHEN** CLI or configuration input selects `readable-view`
- **THEN** core composition constructs the rendered path with the built-in renderer
- **THEN** the input does not identify a renderer implementation

#### Scenario: Linked code supplies a custom renderer

- **WHEN** a linked caller directly constructs a `Rendered` plan with a custom renderer
- **THEN** output orchestration uses that renderer for the invocation
- **THEN** no new CLI/config output value or serialized strategy id is created
