# output-contract Specification

## Purpose
Define Docnav document operation output modes, protocol-response rendering, renderer injection, readable-view presentation, stdout/stderr boundaries, and failure projection. `protocol-contract` owns raw protocol envelopes; adapter capabilities own format facts.

## Requirements
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

### Requirement: Renderer dependency is resolved by linked code

Each `Rendered` plan MUST contain one renderer function or trait value selected by linked code before output orchestration begins. Core CLI `readable-view` MUST use the built-in renderer. A direct linked caller MAY supply a custom renderer through the shared output API without creating another public output value or serialized renderer id.

#### Scenario: Public input selects the built-in renderer

- **WHEN** CLI or configuration input selects `readable-view`
- **THEN** core composition constructs `Rendered` with the built-in renderer

#### Scenario: Linked code supplies a custom renderer

- **WHEN** a linked caller constructs `Rendered` with a custom renderer
- **THEN** output orchestration uses that renderer for the invocation
- **THEN** no new CLI/config value or serialized strategy id is created

### Requirement: both output plans consume one navigation response

After navigation selects a validated base or composed `ProtocolResponse`, `ProtocolJson` and the built-in `Rendered` plan MUST consume that same immutable response. Output orchestration MUST NOT issue another read or maintain renderer-only selection/failure facts.

#### Scenario: protocol-json serializes successful auto-read
- **WHEN** navigation returns a composed result with `auto_read`
- **AND** the caller selects `protocol-json`
- **THEN** `ProtocolJson` serializes the complete outer response as the only stdout JSON value
- **AND** includes the protocol-owned `auto_read` object unchanged

#### Scenario: protocol-json preserves the base response otherwise
- **WHEN** navigation returns the base response without `auto_read`
- **THEN** `ProtocolJson` serializes that base response without any sibling auto-read metadata

#### Scenario: readable-view derives from the same facts
- **WHEN** a caller selects `readable-view`
- **THEN** the built-in renderer derives its base and optional auto-read presentation from the same response
- **AND** does not invent selection, skipped or failed facts

### Requirement: readable-view maps successful auto-read deterministically

The built-in renderer MUST preserve the existing base outline/find readable fields. When `auto_read` is present, it MUST add a readable `auto_read` object and use `/auto_read/read/content` as the nested content block pointer.

#### Scenario: successful auto-read uses a nested block
- **WHEN** the response contains `auto_read`
- **THEN** the readable header maps reason, nested read ref, content type, cost summary and page from the protocol result
- **AND** replaces nested content with a block reference at `/auto_read/read/content`
- **AND** emits exactly one length-delimited block with that pointer and the original content bytes

#### Scenario: absent auto-read preserves the base projection
- **WHEN** the response contains no `auto_read`
- **THEN** the readable header uses the existing base outline/find projection
- **AND** no auto-read header field or content block is emitted

#### Scenario: unstructured outline keeps its base content block
- **WHEN** unstructured outline returns its existing base response
- **THEN** its content remains at `/content`
- **AND** no auto-read header field or block is emitted

### Requirement: output failures retain existing ownership

Renderer failure and writer failure MUST use the existing output failure boundaries. Auto-read MUST NOT introduce a second output attempt or fallback renderer.

#### Scenario: nested content framing invariant fails
- **WHEN** the built-in renderer cannot resolve or frame `/auto_read/read/content`
- **THEN** it returns `RenderFailure` before the first stdout write
- **AND** output orchestration preserves the existing empty-stdout and no-fallback behavior
