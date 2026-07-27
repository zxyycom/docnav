### Case WB-CORE-INVOCATION-LOG-011: Invocation failed auto read keeps only the successful root event

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/content.rs > invocation_failed_auto_read_keeps_only_the_successful_root_event`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_failed_auto_read_keeps_only_the_successful_root_event` 直接验证“Invocation failed auto read keeps only the successful root event”所描述的结果。
