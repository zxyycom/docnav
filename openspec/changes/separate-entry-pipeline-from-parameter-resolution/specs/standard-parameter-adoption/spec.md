本 delta spec 是 `separate-entry-pipeline-from-parameter-resolution` 的未审核临时文档，目标是更新 core CLI 与 adapter SDK 对参数来源解析的采用边界；当前 change 只在 `openspec/changes/separate-entry-pipeline-from-parameter-resolution/` 下形成临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Entrypoints consume parameter source resolution without surrendering owner policy
Core CLI and adapter SDK entrypoints MUST consume entry parameter source registration and typed runtime values for shared document operation parameters while preserving each entrypoint's owner policy. Entrypoints MUST decide whether parameter source resolution applies only after entry classification, and MUST keep raw input immutable while consuming derived values.

#### Scenario: Direct CLI migration keeps strict input boundary
- **WHEN** a migrated direct CLI invocation contains unknown argv, extra positional input, or a known flag that is not applicable to the selected operation
- **THEN** the entrypoint reports a blocking primary input diagnostic at the owner boundary
- **THEN** parameters actually consumed by the selected operation are strictly validated through the parameter source resolution result
- **THEN** rejected raw argv tokens are not rewritten into derived operation values

#### Scenario: Invoke migration keeps strict protocol behavior
- **WHEN** a migrated adapter invoke request omits a registered optional argument that config or defaults can supply
- **THEN** the operation handler may receive a derived typed runtime value
- **THEN** the raw stdin request is not modified
- **WHEN** the migrated invoke request contains an invalid registered argument value
- **THEN** the entrypoint returns protocol-shaped failure using invoke-owned strict input semantics

#### Scenario: Help and machine commands remain outside adoption path
- **WHEN** a migrated entrypoint handles help, manifest, or probe
- **THEN** it does not invoke document parameter source resolution
- **THEN** it preserves the existing plain text or machine schema output boundary
