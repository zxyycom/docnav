本 delta spec 定义 `replace-probe-traversal-with-inferred-routing` 对 `output-contract` 尚未应用的 Target：删除已废止 probe output 的正向 framing 契约，同时保持非文档输出与 document output orchestration 的既有 owner 边界；它不表示 Current 主规范或实现已经迁移。

## MODIFIED Requirements

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

- **WHEN** `docnav` or an adapter emits help、version or manifest output
- **THEN** that owner keeps its existing mode and framing
- **THEN** no removed probe output mode or payload is retained
