本 change 收敛 JSON contract 的 runtime field validation：标准参数与 manifest/probe/protocol JSON 共享 `docnav-typed-fields` 的字段级提取、类型转换和约束校验；JSON Schema 保留为公开契约和验证材料。

## Why

`docnav-typed-fields` 最初服务于标准参数系统，但标准参数迁移后，Docnav 已经形成 typed-field 驱动的 runtime 字段提取、类型转换和字段约束机制。manifest、probe、protocol request/response 继续用 generic JSON Schema validator 执行同类字段规则，会留下重复的 runtime 字段事实源、双层校验路径和额外依赖治理成本。

## What Changes

- 为 manifest、probe、protocol request/response 建立 typed-field runtime validation path。
- 字段级规则由 typed-field extraction/validation 承接：path、presence、type、enum、range、length、pattern 等。
- 跨字段和 envelope 规则仍由对应 semantic validation 承接：protocol envelope、operation/result pairing、diagnostic details 等。
- public JSON Schema 文件继续作为 contract material、example/fixture validation、CI drift check 和第三方对齐材料。
- 移除 runtime `jsonschema` dependency 前，补齐 typed-field core 和 `docnav-protocol` surface parity tests，接手过去由库隐式承担的输入类型和字段约束证明。
- 不生成完整 JSON Schema 文件，不修改既有 public schema、examples 或 protocol behavior 的合同含义。

## Capabilities

### New Capabilities

- `typed-json-contract-validation`: manifest、probe、protocol request/response JSON contract 的 typed-field runtime validation 边界。

### Modified Capabilities

归档时如需并入长期 owner，可拆入 `adapter-protocol`、`docnav-contracts` 或对应主规范；核心变化是 runtime validation source，不改变 public JSON Schema 的合同材料职责。

## Impact

- 影响 `docnav-protocol` decode helpers，以及 core/SDK 调用这些 helper 的 manifest、probe、protocol request/response 边界校验。
- runtime 字段失败从 JSON Schema validator 输出迁到 typed-field validation + semantic validation。
- 项目测试需要直接证明输入类型和字段约束等价类；不能继续只依赖合法 fixtures 或外部 validator。
- runtime `jsonschema` dependency 的移除以 parity evidence 为门禁。
