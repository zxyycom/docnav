本 delta spec 是 `separate-entry-pipeline-from-parameter-resolution` 的未审核临时文档，目标是澄清 typed-field definitions 不拥有标准入口生命周期或原始输入改写；当前 change 只在 `openspec/changes/separate-entry-pipeline-from-parameter-resolution/` 下形成临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Typed field projections validate input views without owning entry lifecycle
Typed field definitions MUST remain field-level metadata and projection machinery. They MAY validate caller-provided input views and produce field-attributed validation failures, but MUST NOT classify CLI commands, decide config loading, mutate raw argv or JSON input, own protocol envelopes, own manifest/probe policy, or render output.

#### Scenario: Parameter source resolution consumes typed-field metadata
- **WHEN** entry parameter source resolution consumes typed field definitions
- **THEN** typed-fields provide field identity, processing metadata, validation metadata, defaults metadata, and attributed validation failures
- **THEN** parameter source resolution owns source priority, source info, config source handling, passthrough handoff, and operation argument binding
- **THEN** the entry owner owns lifecycle, request construction, diagnostics projection, and output behavior

#### Scenario: Typed-field extraction does not mutate raw input
- **WHEN** a typed-field projection extracts values from a caller-provided JSON input view
- **THEN** extraction may return typed values or validation failures
- **THEN** extraction does not add defaults to the caller-owned raw JSON value
- **THEN** extraction does not remove unmapped fields from the caller-owned raw JSON value
