### Case WB-NAV-ADAPTER-SOURCE-003: Explicit missing adapter error carries invocation failure layer

Entry:
- `crates/shared/navigation/src/tests/navigation/adapter_source.rs > explicit_missing_adapter_error_carries_invocation_failure_layer`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation adapter selection 保持静态来源边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `explicit_missing_adapter_error_carries_invocation_failure_layer` 直接验证“Explicit missing adapter error carries invocation failure layer”所描述的结果。
