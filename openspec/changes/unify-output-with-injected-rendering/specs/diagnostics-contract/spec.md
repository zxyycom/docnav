本 delta 的目标是让 primary diagnostic 在 protocol 与 rendered 两条路径中保持同一 identity；当前文档只在 `openspec/changes/unify-output-with-injected-rendering/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## MODIFIED Requirements

### Requirement: DiagnosticCode owns identity and canonical details

Every stable diagnostic code MUST have a single identity owner and canonical detail shape. Protocol projection MUST serialize that identity through the protocol error contract. Rendered projection MUST pass the same primary `DiagnosticRecord` to the selected renderer. Added context MUST preserve code identity and canonical detail semantics.

#### Scenario: One diagnostic enters both output paths

- **WHEN** the same diagnostic is projected through `ProtocolJson` and `Rendered`
- **THEN** protocol output uses the canonical diagnostic code and details
- **THEN** renderer input contains the same diagnostic identity and structured meaning
- **THEN** renderer text remains presentation rather than a second diagnostic contract
