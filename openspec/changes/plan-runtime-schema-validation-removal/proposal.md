本 change 仅记录未来评估运行时 JSON Schema 校验迁移的计划；它只在 `openspec/changes/plan-runtime-schema-validation-removal/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

当前 release binary 受通用 `jsonschema` 运行时依赖影响明显增大，而 Docnav 当前协议运行时实际需要的校验能力远小于完整 JSON Schema Draft 2020-12 引擎。需要先记录一个未来迁移方向，避免后续围绕体积优化时直接删除校验或误改现行契约。

## What Changes

- 记录未来可将 release runtime 中的通用 JSON Schema 引擎迁移为 Docnav 协议专用的 typed validation 与 semantic validation。
- 保留当前阶段的规范立场：现行 schema、示例、协议字段和 adapter 契约不因本 change 自动改变。
- 要求未来真正实施前先证明校验语义覆盖，包括未知字段、必需字段、类型、版本常量、operation/result 绑定、分页字段、manifest/probe 输出和错误映射。
- 要求未来真正实施后，CI 仍通过 JSON Schema 或等价工具校验 schemas、examples 和协议输出 fixture，避免 release runtime 体积优化削弱公共契约。
- 非目标：本 change 不要求现在移除 `jsonschema` 依赖，不修改现有 Rust 实现，不修改现行主规范，不改变 adapter 对外行为。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `adapter-protocol`: 未来可能调整协议与 adapter SDK 的运行时校验实现策略，但当前 change 仍为未审核计划，不改变现行 requirement。

## Impact

- 受影响范围：未来可能涉及 `docnav-protocol`、`docnav-adapter-sdk`、`docnav` adapter 输出校验、schema/example 验证脚本和 release binary 体积。
- 当前影响：仅新增本 change 目录下的计划性 OpenSpec 文档；不影响当前 CLI、adapter、MCP、schema、examples、主 spec 或 release 流程。
