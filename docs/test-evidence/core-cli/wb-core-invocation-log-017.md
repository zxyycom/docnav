### Case WB-CORE-INVOCATION-LOG-017: Invocation output write failure logs output projection without completion

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/output.rs > invocation_output_write_failure_logs_output_projection_without_completion`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_output_write_failure_logs_output_projection_without_completion` 直接验证“Invocation output write failure logs output projection without completion”所描述的结果。
