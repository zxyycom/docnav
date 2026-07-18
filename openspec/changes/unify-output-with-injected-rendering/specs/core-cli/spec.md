## MODIFIED Requirements

### Requirement: Core CLI selects output mode and process exit behavior

Core CLI MUST accept exactly the document output modes `readable-view` and `protocol-json`, construct the corresponding output plan, and map failures to existing process exit behavior. Omitted output or `readable-view` MUST construct `Rendered` with the built-in renderer. `protocol-json` MUST construct `ProtocolJson`. A document failure that occurs before navigation returns a response MUST be projected into `ProtocolResponse::Failure` before the selected output plan executes.

#### Scenario: Omitted output uses the built-in renderer

- **WHEN** a caller omits output mode for a document operation
- **THEN** core constructs `Rendered` with the built-in `readable-view` renderer

#### Scenario: Explicit readable-view uses the built-in renderer

- **WHEN** a caller requests `--output readable-view`
- **THEN** core constructs `Rendered` with the built-in renderer

#### Scenario: Protocol JSON bypasses rendering

- **WHEN** a caller requests `--output protocol-json`
- **THEN** core constructs `ProtocolJson`
- **THEN** the protocol response is emitted without invoking a renderer

#### Scenario: Early document failure follows the recognized output mode

- **WHEN** a document failure occurs before navigation returns a response
- **THEN** core constructs `ProtocolResponse::Failure` through the existing protocol error projection
- **THEN** `ProtocolJson` serializes that response or `Rendered` passes it to the built-in renderer

#### Scenario: Removed readable-json value is rejected

- **WHEN** CLI or config input supplies `readable-json`
- **THEN** core reports the normal invalid-value diagnostic
- **THEN** no readable-json plan、alias or fallback is constructed

#### Scenario: Non-document output remains unchanged

- **WHEN** help、version or another non-document command succeeds
- **THEN** core keeps that command's existing owner-specific output behavior
