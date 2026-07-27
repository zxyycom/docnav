### Case WB-NAV-INPUT-RESOLUTION-006: Navigation reports unknown adapter id under options as config diagnostic

Entry:
- `crates/shared/navigation/src/tests/navigation/native_options/validation.rs > navigation_reports_unknown_adapter_id_under_options_as_config_diagnostic`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation input resolution 保持来源解析边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `navigation_reports_unknown_adapter_id_under_options_as_config_diagnostic` 直接验证“Navigation reports unknown adapter id under options as config diagnostic”所描述的结果。
