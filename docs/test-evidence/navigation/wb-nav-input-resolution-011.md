### Case WB-NAV-INPUT-RESOLUTION-011: Navigation reports explicit range failure with adapter compatible diagnostic

Entry:
- `crates/shared/navigation/src/tests/navigation/native_options/validation.rs > navigation_reports_explicit_range_failure_with_adapter_compatible_diagnostic`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation input resolution 保持来源解析边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `navigation_reports_explicit_range_failure_with_adapter_compatible_diagnostic` 直接验证“Navigation reports explicit range failure with adapter compatible diagnostic”所描述的结果。
