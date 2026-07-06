# output-contract Specification

## Purpose
Define Docnav document operation output modes, readable payloads, readable-view framing, readable-json shape, renderer configuration, stdout/stderr boundaries, and failure projection. `protocol-contract` owns raw protocol envelopes; adapter capabilities own format facts.

## Requirements
### Requirement: Document output modes are fixed
Document operations MUST expose exactly the supported output modes: default readable-view for humans, readable-json for structured readable payloads, and protocol-json for raw machine protocol.

#### Scenario: Default human output
- **WHEN** a caller omits output mode for a document operation
- **THEN** Docnav emits readable-view
- **THEN** stdout contains human-readable content derived from the readable payload

#### Scenario: Protocol output
- **WHEN** a caller requests protocol-json
- **THEN** stdout contains the raw protocol envelope
- **THEN** readable framing is not mixed into stdout

### Requirement: Readable-view and readable-json share one payload source
Readable-view and readable-json MUST derive from the same typed readable payload. Their representation can differ only after owner, status, primary diagnostic, content facts, and continuation semantics are fixed.

#### Scenario: Successful read
- **WHEN** read succeeds
- **THEN** readable-json exposes the typed readable success payload
- **THEN** readable-view renders the same payload through the configured renderer

#### Scenario: Failure output
- **WHEN** a document operation fails
- **THEN** readable-json exposes the primary diagnostic projection
- **THEN** readable-view renders that same diagnostic for human use

### Requirement: Renderer config owns readable-view framing
Readable-view block framing, block field selection, block references, and renderer conformance vectors MUST be declared by repository renderer configuration. Operation implementations MUST use that shared framing when they render through the generic readable-view path.

#### Scenario: Rendering a content block
- **WHEN** readable-view emits a content block
- **THEN** the block delimiters follow renderer config
- **THEN** the block field path is one declared by renderer config

### Requirement: Output orchestration is above rendering
Document output orchestration MUST choose mode, status projection, stdout/stderr channel behavior, and renderer invocation before writing process output. Renderer helpers receive a selected projection and return text or structured output without owning CLI exit semantics or protocol envelope shape.

#### Scenario: Renderer failure
- **WHEN** readable-view rendering fails
- **THEN** output orchestration reports a stable output diagnostic
- **THEN** protocol-json semantics remain independent of readable renderer failure

### Requirement: Readable output supports unstructured outline content
Readable output MUST represent declared unstructured outline full-read results as content, not as fake outline entries or pagination state.

#### Scenario: Unstructured outline result
- **WHEN** protocol outline returns unstructured full-read content
- **THEN** readable-json exposes the content block
- **THEN** readable-view renders the content using normal block framing
