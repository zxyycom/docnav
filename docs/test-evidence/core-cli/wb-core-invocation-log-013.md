### Case WB-CORE-INVOCATION-LOG-013: Invocation capture failure does not change operation result

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/content.rs > invocation_capture_failure_does_not_change_operation_result`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_capture_failure_does_not_change_operation_result` 直接验证“Invocation capture failure does not change operation result”所描述的结果。
