### Case WB-CORE-OUTPUT-008: App error normalizes non protocol diagnostic before document output

Entry:
- `crates/docnav/src/output/tests.rs > app_error_normalizes_non_protocol_diagnostic_before_document_output`

Contract:
- `docs/output.md` 定义或约束“Core 输出编排保持通道边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `app_error_normalizes_non_protocol_diagnostic_before_document_output` 直接验证“App error normalizes non protocol diagnostic before document output”所描述的结果。
