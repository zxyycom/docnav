### Case WB-CORE-INVOCATION-LOG-014: Invocation failure logs bounded layer code and summary

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/failure.rs > invocation_failure_logs_bounded_layer_code_and_summary`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_failure_logs_bounded_layer_code_and_summary` 直接验证“Invocation failure logs bounded layer code and summary”所描述的结果。
