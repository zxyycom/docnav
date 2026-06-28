本 tasks 先建立 parity matrix，再迁移 runtime 字段级校验，最后用测试和验证证明可以移除 runtime `jsonschema` dependency。

## 1. Scope and Proof Plan

- [ ] 1.1 列出 manifest、probe、protocol request/response 当前 runtime schema validation 覆盖的 schema keywords 和失败用例。
- [ ] 1.2 将每个 schema-backed 规则归属到 typed-field validation、semantic validation 或 CI-only schema material。
- [ ] 1.3 定义移除 runtime `jsonschema` dependency 前必须保持的 parity：failure stage、error category、field path、stdout/stderr placement 和 protocol envelope。
- [ ] 1.4 识别当前测试中依赖 `jsonschema` 库隐式覆盖、但项目没有直接断言的输入类型和字段约束等价类。
- [ ] 1.5 判断 typed-field core 是否缺少当前 runtime contract 需要的字段级能力；只为缺口补最小扩展。

## 2. Runtime Migration

- [ ] 2.1 为 manifest、probe、protocol request/response 建立 typed-field definition sets，覆盖已归属的字段级规则。
- [ ] 2.2 将 `docnav-protocol` decode helpers 接入 typed-field runtime validation，并保留 contract-owned semantic validation。
- [ ] 2.3 将 public JSON Schema validation 保留在 examples、fixtures、CI drift checks 或 tooling 验证链路。
- [ ] 2.4 parity 通过后，移除或降级 `docnav-protocol` 对 generic `jsonschema` validator 的普通运行时依赖。

## 3. Verification

- [ ] 3.1 为 typed-field core 添加等价类测试，覆盖 missing required、null handling、wrong type、enum、numeric range、string length、pattern、array length 和 unmapped/unknown path 处理。
- [ ] 3.2 为 `docnav-protocol` surface 添加 parity tests，覆盖 manifest、probe、protocol request/response 的 unknown fields、missing required fields、wrong types、version constants、field constraints、semantic validation 和 operation/result pairing。
- [ ] 3.3 按 `docs/testing/case-maintenance.md` 更新测试 case 归属、账本和源码 `@case` 标记。
- [ ] 3.4 运行 schema/example/fixture、protocol boundary 和 dependency-scope 验证。
