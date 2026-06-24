本 spec delta 仅记录未来评估运行时 JSON Schema 校验迁移时必须满足的协议保护条件；它只在 `openspec/changes/plan-runtime-schema-validation-removal/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Runtime schema validator migration gate

Future removal of a generic JSON Schema validator from release runtime MUST preserve the current Docnav protocol contract through typed request/response decoding, explicit field constraints, and semantic validation before the runtime dependency is removed.

#### Scenario: Current behavior remains unchanged while this change is only a plan

- **WHEN** this change exists only under `openspec/changes/plan-runtime-schema-validation-removal/`
- **THEN** existing runtime JSON Schema validation behavior remains the active implementation contract

#### Scenario: Future migration proves schema keyword coverage before dependency removal

- **WHEN** a future implementation proposes removing the generic JSON Schema validator from release runtime
- **THEN** the implementation MUST map every schema keyword used by current protocol, manifest, probe, and readable-output schemas to typed validation, semantic validation, or retained CI schema validation
- **THEN** the implementation MUST include positive and negative tests for unknown fields, missing required fields, wrong field types, invalid version constants, invalid operation/result pairs, invalid pagination fields, and invalid manifest/probe payloads

#### Scenario: Future release runtime remains fail-closed

- **WHEN** adapter invoke stdin, manifest output, probe output, or protocol response JSON violates the current Docnav contract
- **THEN** runtime validation MUST reject the payload before it is treated as a valid protocol value
- **THEN** the failure MUST preserve the stable error category and process-boundary behavior required by the current contract

#### Scenario: CI keeps JSON Schema contract validation

- **WHEN** release runtime no longer links a generic JSON Schema validator
- **THEN** CI or test validation MUST still compile the public schemas and validate schema examples or equivalent fixtures
- **THEN** schema, example, and fixture drift MUST fail validation before release artifacts are accepted
