本 design 记录 typed JSON contract validation 的目标终态和迁移门禁：runtime 字段级校验收敛到 `docnav-typed-fields`，semantic validation 与 JSON Schema 继续承担各自 owner 职责。

## Context

当前 `docnav-protocol` runtime decode path 先用 `jsonschema` 校验 `docs/schemas/` 中的 manifest、probe、protocol request/response schema，再进行 serde decode 和 semantic validation。标准参数迁移后，`docnav-typed-fields` 已经承接字段级 path extraction、类型转换、required/default、字段约束和错误归属。JSON contract 继续用 generic schema validator 执行同类字段规则，会让 runtime 长期保留两套字段校验机制。

## Goals / Non-Goals

**Goals:**

- 让 manifest、probe、protocol request/response 的 runtime 字段级校验使用 typed-field extraction/validation。
- 用 typed field metadata 表达字段 path、presence、type、enum、range、length、pattern 等字段级规则。
- 用 semantic validation 承接跨字段、protocol envelope、operation/result pairing 和 diagnostic details 等规则。
- 保留 JSON Schema 作为 public contract material、example/fixture validation、CI drift check 和第三方对齐材料。
- 用 parity tests 证明过去由 `jsonschema` 库隐式承担的输入类型和字段约束失败行为。
- parity 覆盖建立后，从 `docnav-protocol` 的普通运行时依赖中移除 generic `jsonschema` validator；需要 schema validator 的检查留在 dev/test/tooling 链路。

**Non-Goals:**

- 不生成完整 JSON Schema 文件。
- 不改变 public JSON Schema 的合同含义、examples/fixtures、protocol envelope、stable error category、field path、stdout/stderr placement 或 exit behavior。
- 不把 operation/result pairing、diagnostic details 等跨字段规则塞进字段级 metadata。
- 不把 `docnav-typed-fields` 扩展成通用 JSON Schema engine；只实现 Docnav runtime contract 需要的字段级规则。

## Decisions

1. Runtime 字段级校验来源改为 typed-field extraction/validation。
   - Impact: manifest、probe、protocol request/response 的字段级失败由 typed-field 机制产生，并由对应边界映射到既有错误输出。

2. Semantic validation 继续承接跨字段和 envelope 规则。
   - Impact: typed-field metadata 只承接字段级事实；operation/result pairing、diagnostic details 和 surface-owned semantic rules 仍由对应 owner 校验。

3. JSON Schema 文件保留为契约和验证材料。
   - Impact: examples、fixtures、CI drift checks 和第三方实现仍可使用 public schema；生产 decode path 不依赖 schema 文件校验。

4. `jsonschema` 普通运行时依赖是迁移期依赖。
   - Impact: parity tests 覆盖现有 schema-backed 失败行为后，应移除 `docnav-protocol` runtime dependency，或降级到 dev/test/tooling 链路。

5. 输入类型证明责任转移到项目测试。
   - Impact: typed-field core 和 `docnav-protocol` surface tests 必须覆盖代表性 schema-backed failures，不能仅依赖合法 fixtures 或 serde decode 成功证明。

## Parity Matrix

Runtime schema-backed keywords covered before migration:

- manifest: `required`、`type`、`const`、`enum`、`minLength`、`minItems`、`pattern`、`additionalProperties`、`items`、`uniqueItems`。
- probe: `required`、`type`、`const`、`enum`、`minimum`、`maximum`、`minItems`、`additionalProperties`、`items`。
- protocol request: `required`、`type`、`const`、`enum`、`minimum`、`minLength`、`additionalProperties`、`oneOf`、`allOf`、`$ref`。
- protocol response: `required`、`type`、`const`、`enum`、`minimum`、`minLength`、`additionalProperties`、`items`、`uniqueItems`、`oneOf`、`allOf`、`$ref`、error details conditional requirements。

Rule ownership after migration:

- typed-field runtime validation: field path/presence/type, string enum and version constants, numeric range, string/array length, regex pattern, array unique-items, required nullable fields, and item-level field definitions for manifest/probe/result arrays.
- contract-owned runtime helpers around typed-fields: object unknown-field checks and array item iteration, so typed-field remains a field definition/validation core rather than a full JSON Schema engine.
- semantic validation: protocol operation/arguments pairing, operation/result pairing, probe supported/format and reason semantics, protocol error details rules, and other cross-field owner rules.
- CI-only schema material: public JSON Schema files, examples, fixtures, and drift/tooling validation.

Parity to preserve before removing runtime `jsonschema`:

- field-contract failures still surface as `DecodePipelineStage::Schema` and keep schema filename identifiers for existing caller mappings.
- request/response protocol failures keep protocol envelope behavior; malformed request field failures still map to `INVALID_REQUEST` through adapter/core owners.
- boundary diagnostics and stdout/stderr placement keep existing owner behavior; adapter output writers still use `validate_*_value` but that runtime path no longer invokes a generic JSON Schema validator.
- project-owned tests cover typed-field equivalence classes and manifest/probe/protocol request/response surface failures; public JSON Schema remains explicitly exercised in fixture/schema tests.

Typed-field core gaps found and resolved:

- required-but-nullable fields were needed for probe `format`, response `page`, and failure `operation`; this is now expressible through `ExpectedFieldShape::required_nullable()`.
- array `uniqueItems` was needed for manifest and info capabilities; this is now an opt-in array constraint.
- array item rules are handled by item-level typed-field definition sets plus contract-owned iteration, avoiding a general JSON Schema engine.

## Risks / Trade-offs

- [Risk] 字段规则迁移不完整 → Mitigation: 每个 runtime surface 先列出 schema-backed keywords，并归属到 typed-field validation、semantic validation 或 CI-only schema material。
- [Risk] error behavior 改变 → Mitigation: parity tests 覆盖 schema-backed failures 的 stage、category、field path、stdout/stderr placement 和 protocol envelope。
- [Risk] typed-field definitions 与 public JSON Schema 文件漂移 → Mitigation: schema/example/fixture CI 和 metadata parity tests 作为移除 runtime validator 前置。
- [Risk] 过去由 `jsonschema` 库覆盖的输入类型边界缺少项目级测试 → Mitigation: 移除 runtime dependency 前补齐 typed-field core 等价类和 `docnav-protocol` surface parity tests。

## Open Questions

None.
