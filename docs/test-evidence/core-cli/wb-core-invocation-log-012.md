### Case WB-CORE-INVOCATION-LOG-012: Invocation unwritable log path does not change operation result

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/content.rs > invocation_unwritable_log_path_does_not_change_operation_result`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_unwritable_log_path_does_not_change_operation_result` 直接验证“Invocation unwritable log path does not change operation result”所描述的结果。
