### Case WB-DIAG-RECORD-004: Format record accepts candidate failures

Entry:
- `crates/shared/diagnostics/src/tests/record.rs > format_record_accepts_candidate_failures`

Contract:
- `docs/architecture.md` 定义或约束“Diagnostic record construction validates typed details”所涉及的稳定行为边界。

Proves:
- 原生入口 `format_record_accepts_candidate_failures` 直接验证“Format record accepts candidate failures”所描述的结果。
