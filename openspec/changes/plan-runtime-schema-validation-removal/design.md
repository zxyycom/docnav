本 design 仅说明未来评估运行时 JSON Schema 校验迁移的技术方向；它只在 `openspec/changes/plan-runtime-schema-validation-removal/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

Docnav 当前通过 `jsonschema` 在运行时校验 protocol request/response、manifest 和 probe 等 JSON payload。该做法直接复用 JSON Schema 契约，正确性边界清晰，但 release binary 会携带完整 JSON Schema Draft 2020-12 引擎及其 URI、IDNA、regex、numeric 和引用解析依赖。

当前讨论只面向未来体积优化。现行规范仍要求当前 schema、字段和语义校验通过；本 change 不授权现在删除运行时校验，也不改变 adapter、CLI 或 MCP 的对外行为。

## Goals / Non-Goals

**Goals:**

- 为未来从 release runtime 移除通用 JSON Schema 引擎记录可审核路线。
- 保持 Docnav 协议 fail-closed：未知字段、缺失字段、类型错误、版本常量错误和 operation/result 不匹配仍必须失败。
- 将运行时校验职责迁移到 Rust 类型反序列化、字段约束和协议语义校验。
- 保留 CI 或测试阶段的 JSON Schema 校验，继续证明 schemas、examples 和 fixture 与公共契约一致。

**Non-Goals:**

- 不在当前 change 中移除 `jsonschema` 依赖。
- 不改变 JSON Schema 文件的方言、字段集合或公共契约。
- 不把 schema 校验下放给 `docnav-mcp`，MCP bridge 仍保持格式无关和协议转发边界。
- 不接受“少校验换体积”的实现；未来迁移必须提供等价覆盖证明。

## Decisions

1. 未来 release runtime 以协议专用校验替代通用 JSON Schema 引擎。

   理由：Docnav 当前运行时需要的是固定协议 envelope、manifest、probe 和 adapter result shape，而不是任意用户提供的 JSON Schema。Rust 类型、`deny_unknown_fields`、非零/范围类型、枚举和显式 semantic validation 可以覆盖当前主要契约，同时显著降低静态链接体积。

   备选方案：继续使用 `jsonschema`。该方案维护成本最低，但保留当前体积成本。

   备选方案：替换为 `boon` 等另一个完整 JSON Schema validator。该方案仍需携带通用 schema 引擎，生态和行为差异还需要额外验证，不应作为默认路线。

2. JSON Schema 继续作为契约和 CI 校验材料。

   理由：删除 release runtime 中的通用 validator 不等于删除 schema。Schema 仍应用于文档、示例、fixture、schema 编译和跨语言契约验证，防止 Rust 专用校验与公开协议漂移。

3. 未来实施前必须先建立 schema keyword 覆盖表。

   理由：只有把当前 schema 使用到的 `type`、`required`、`additionalProperties`、`const`、`enum`、`minLength`、`minItems`、`minimum`、`oneOf`、`allOf`、`if/then`、`$ref` 和 `pattern` 映射到 Rust 校验点，才能证明迁移没有降低输入防护。

4. 运行时错误稳定性必须被单独验证。

   理由：通用 JSON Schema error path 和手写语义错误的输出形态不同。未来实现可以不复制第三方库的完整文案，但必须保留稳定错误分类、失败阶段和足够定位字段的诊断。

## Risks / Trade-offs

- [Risk] Rust typed validator 与 JSON Schema 文件漂移 → Mitigation：在 CI 中保留 schema fixture 校验，并增加 schema keyword inventory 检查。
- [Risk] 错误详情不再包含第三方 validator 的完整 schema path → Mitigation：明确稳定错误 ID、阶段、字段路径和 human-readable reason 的最小要求。
- [Risk] 未来 schema 新增当前 typed validator 未覆盖的 keyword → Mitigation：schema 变更必须同步更新覆盖表和负例测试，否则验证失败。
- [Risk] 过早移除 `jsonschema` 造成 adapter 边界变松 → Mitigation：tasks 中设置阻塞级 parity 审计，审计未通过前不得执行实现。

## Migration Plan

1. 仅保留本 change 作为计划，不执行代码迁移。
2. 未来重新启动时，先盘点所有当前 schema 使用的 keyword、正例 fixture 和负例 fixture。
3. 为 `docnav-protocol`、`docnav-adapter-sdk` 和 `docnav` 输出边界补齐 typed + semantic validation。
4. 将 `jsonschema` 从 release runtime 依赖迁移为 dev/test/CI 校验依赖，或改由外部验证脚本承担。
5. 对比迁移前后的错误分类、失败阶段和 release binary 体积。
6. 只有在 parity 审计通过后，才允许移除 release runtime 的通用 JSON Schema validator。

## Open Questions

- 未来 CI 校验继续使用 Rust `jsonschema` 作为 dev-dependency，还是改用 Node Ajv 等外部工具。
- 未来错误诊断是否要求保留 schema path，还是只要求稳定字段路径和错误分类。
- `pattern` 的覆盖范围是否只限当前 extension/ref 等有限规则，还是需要支持更通用的 schema pattern 语义。
