### Case WB-CORE-INVOCATION-LOG-015: Invocation linked handler structured diagnostic logs adapter dispatch failure

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/failure.rs > invocation_linked_handler_structured_diagnostic_logs_adapter_dispatch_failure`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_linked_handler_structured_diagnostic_logs_adapter_dispatch_failure` 直接验证“Invocation linked handler structured diagnostic logs adapter dispatch failure”所描述的结果。
