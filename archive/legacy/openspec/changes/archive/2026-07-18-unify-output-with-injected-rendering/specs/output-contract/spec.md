## RENAMED Requirements

- FROM: `### Requirement: Readable-view and readable-json share one payload source`
- TO: `### Requirement: Rendered output consumes the protocol response`
- FROM: `### Requirement: Renderer config owns readable-view framing`
- TO: `### Requirement: Selected renderer owns presentation text`

## MODIFIED Requirements

### Requirement: Document output modes are fixed

Document operations MUST expose exactly the public output modes `readable-view` and `protocol-json`. Shared output MUST represent them as `Rendered(RenderStrategy)` and `ProtocolJson`. Core CLI MUST map omitted output and `readable-view` to `Rendered` with the built-in renderer; `protocol-json` MUST select `ProtocolJson`.

#### Scenario: Omitted output uses the built-in renderer

- **WHEN** a caller omits output mode for a document operation
- **THEN** core selects `Rendered`
- **THEN** core supplies the built-in `readable-view` renderer

#### Scenario: Protocol JSON bypasses rendering

- **WHEN** a caller requests `protocol-json`
- **THEN** output serializes the protocol response
- **THEN** no renderer is invoked

#### Scenario: Removed readable-json value is rejected

- **WHEN** CLI or config input selects `readable-json`
- **THEN** the input follows normal invalid-value handling
- **THEN** no readable-json output plan is constructed

### Requirement: Rendered output consumes the protocol response

Before a document output plan emits data, the document success or failure MUST be represented as one immutable `ProtocolResponse`. `ProtocolJson` MUST serialize that response, and `Rendered` MUST pass the response unchanged to its selected renderer. `ProtocolResponse` is the complete renderer input contract.

#### Scenario: Renderer receives a success response

- **WHEN** a document operation produces `ProtocolResponse::Success`
- **AND** `Rendered` is selected
- **THEN** the selected renderer receives that response including its typed operation result

#### Scenario: Renderer receives a failure response

- **WHEN** a document operation failure is projected as `ProtocolResponse::Failure`
- **AND** `Rendered` is selected
- **THEN** the selected renderer receives that response including its existing protocol error and optional operation

#### Scenario: Early document failure uses the same contract

- **WHEN** a document failure occurs before navigation returns a completed response
- **THEN** core uses the existing protocol projection to construct `ProtocolResponse::Failure`
- **THEN** the selected output plan consumes that failure response

### Requirement: Selected renderer owns presentation text

The selected renderer MUST return one complete UTF-8 `String` or `RenderFailure` before the first stdout write. Output orchestration MUST write a successful string exactly as returned without adding framing、separators or a trailing newline. The built-in renderer MUST preserve the repository-owned `readable-view` text contract; a custom renderer owns its own presentation contract.

#### Scenario: Built-in renderer applies readable-view framing

- **WHEN** the built-in renderer emits a configured content block
- **THEN** its header、block reference and delimiters follow the readable-view contract

#### Scenario: Custom renderer controls its text

- **WHEN** linked code supplies a custom renderer and rendering succeeds
- **THEN** stdout equals the returned UTF-8 string

### Requirement: Output orchestration is above rendering

Document output orchestration MUST execute the selected output plan and control document stdout/stderr. `ProtocolJson` MUST serialize the supplied `ProtocolResponse` without invoking a renderer. `Rendered` MUST invoke exactly its selected renderer before writing stdout. A returned `RenderFailure` MUST leave stdout empty and MUST NOT trigger another renderer. A writer failure after successful rendering MUST remain a distinct I/O failure.

#### Scenario: Protocol output is independent

- **WHEN** `ProtocolJson` is selected
- **THEN** stdout contains one protocol response or failure envelope
- **THEN** renderer availability and behavior have no effect

#### Scenario: Renderer fails before stdout

- **WHEN** the selected renderer returns `RenderFailure`
- **THEN** stdout remains empty
- **THEN** output orchestration returns the render failure
- **THEN** no second renderer is invoked

#### Scenario: Writer fails after rendering

- **WHEN** rendering succeeds and the stdout writer fails
- **THEN** output orchestration reports the writer I/O failure
- **THEN** it does not reclassify the failure as `RenderFailure`

#### Scenario: Non-document output remains owner-specific

- **WHEN** `docnav` or an adapter emits help、version、manifest or probe output
- **THEN** that owner keeps its existing mode and framing

### Requirement: Readable output supports unstructured outline content

The built-in `readable-view` renderer MUST represent declared unstructured outline full-read results from `ProtocolResponse::Success` as content, not as outline entries or pagination state.

#### Scenario: Built-in renderer emits unstructured content

- **WHEN** outline returns unstructured full-read content and the built-in renderer is selected
- **THEN** readable-view emits the content through its normal block framing
- **THEN** it does not invent outline entries or pagination state

## ADDED Requirements

### Requirement: Renderer dependency is resolved by linked code

Each `Rendered` plan MUST contain one renderer function or trait value selected by linked code before output orchestration begins. Core CLI `readable-view` MUST use the built-in renderer. A direct linked caller MAY supply a custom renderer through the shared output API without creating another public output value or serialized renderer id.

#### Scenario: Public input selects the built-in renderer

- **WHEN** CLI or configuration input selects `readable-view`
- **THEN** core composition constructs `Rendered` with the built-in renderer

#### Scenario: Linked code supplies a custom renderer

- **WHEN** a linked caller constructs `Rendered` with a custom renderer
- **THEN** output orchestration uses that renderer for the invocation
- **THEN** no new CLI/config value or serialized strategy id is created
