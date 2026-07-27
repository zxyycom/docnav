### Case WB-CORE-INVOCATION-LOG-018: Invocation readable view stdout stays free of log events

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/output.rs > invocation_readable_view_stdout_stays_free_of_log_events`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_readable_view_stdout_stays_free_of_log_events` 直接验证“Invocation readable view stdout stays free of log events”所描述的结果。
