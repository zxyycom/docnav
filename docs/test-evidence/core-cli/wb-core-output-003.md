### Case WB-CORE-OUTPUT-003: Non document json writes value directly

Entry:
- `crates/docnav/src/output/tests.rs > non_document_json_writes_value_directly`

Contract:
- `docs/output.md` 定义或约束“Core 输出编排保持通道边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `non_document_json_writes_value_directly` 直接验证“Non document json writes value directly”所描述的结果。
