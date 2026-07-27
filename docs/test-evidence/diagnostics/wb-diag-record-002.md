### Case WB-DIAG-RECORD-002: Diagnostic record validates details and uses code defaults

Entry:
- `crates/shared/diagnostics/src/tests/record.rs > diagnostic_record_validates_details_and_uses_code_defaults`

Contract:
- `docs/architecture.md` 定义或约束“Diagnostic record construction validates typed details”所涉及的稳定行为边界。

Proves:
- 原生入口 `diagnostic_record_validates_details_and_uses_code_defaults` 直接验证“Diagnostic record validates details and uses code defaults”所描述的结果。
