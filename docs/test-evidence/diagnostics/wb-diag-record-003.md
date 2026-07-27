### Case WB-DIAG-RECORD-003: Diagnostic record rejects empty summary

Entry:
- `crates/shared/diagnostics/src/tests/record.rs > diagnostic_record_rejects_empty_summary`

Contract:
- `docs/architecture.md` 定义或约束“Diagnostic record construction validates typed details”所涉及的稳定行为边界。

Proves:
- 原生入口 `diagnostic_record_rejects_empty_summary` 直接验证“Diagnostic record rejects empty summary”所描述的结果。
