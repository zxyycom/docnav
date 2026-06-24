本 change 只起草用 typed field validation 覆盖 JSON contract runtime 校验的想法和审计入口；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## Why

manifest、probe、protocol request/response 等非标准参数 JSON 也需要 typed path/value 校验。它们可以复用 typed field definition 的字段约束能力，但不应被称为标准参数，也不应继承 CLI/config/default/source merge 语义。

## What Changes

- 起草 typed JSON contract validation 方向，用 typed field metadata 表达 manifest、probe 和 protocol JSON 的字段校验。
- JSON Schema 保留为契约材料、示例/fixture/CI 校验和第三方对齐材料。
- runtime 主校验逐步走 typed decoder + semantic validation；是否移除 generic `jsonschema` runtime dependency 由审计和 parity tests 决定。
- 不在本 change 生成完整 schema 文件。
- 不修改现有 schema、examples 或 protocol behavior。

## Capabilities

### New Capabilities

- `typed-json-contract-validation`: manifest、probe、protocol JSON contract 的 typed runtime validation 边界。

### Modified Capabilities

当前草案不直接修改已归档主 spec；审计门禁会确认是否应拆到 `adapter-protocol` 和 `docnav-contracts` delta。

## Impact

- 未来会影响 `docnav-protocol` decode helpers、manifest/probe parsing、protocol response validation 和 CI fixture strategy。
- 可为后续 runtime schema validator removal 提供实现路径。
- 当前 change 不改变 public JSON schema 文件和 observable error mapping。
