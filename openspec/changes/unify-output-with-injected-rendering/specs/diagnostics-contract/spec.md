## MODIFIED Requirements

### Requirement: DiagnosticCode owns identity and canonical details

Every stable diagnostic code MUST have a single identity owner and canonical detail shape. Protocol projection MUST serialize that identity through the protocol error contract. Rendered projection MUST pass the same primary `DiagnosticRecord` to the selected renderer. Added context MUST preserve code identity and canonical detail semantics. A returned `RenderFailure` MUST map to the output-owned boundary identity `output_render_failed`; renderer-specific causes MAY appear only as bounded diagnostic details and MUST NOT define additional public codes.

#### Scenario: One diagnostic enters both output paths

- **WHEN** the same diagnostic is projected through `ProtocolJson` and `Rendered`
- **THEN** protocol output uses the canonical diagnostic code and details
- **THEN** renderer input contains the same diagnostic identity and structured meaning
- **THEN** renderer text remains presentation rather than a second diagnostic contract

#### Scenario: Renderer implementation returns a failure

- **WHEN** the selected renderer returns `RenderFailure`
- **THEN** output orchestration reports `output_render_failed`
- **THEN** renderer-private failure data does not create another public diagnostic identity
