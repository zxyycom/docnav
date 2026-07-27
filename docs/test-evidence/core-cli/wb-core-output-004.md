### Case WB-CORE-OUTPUT-004: Document readable view uses shared output facade

Entry:
- `crates/docnav/src/output/tests.rs > document_readable_view_uses_shared_output_facade`

Contract:
- `docs/output.md` 定义或约束“Core 输出编排保持通道边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `document_readable_view_uses_shared_output_facade` 直接验证“Document readable view uses shared output facade”所描述的结果。
