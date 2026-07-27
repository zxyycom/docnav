### Case WB-NAV-INPUT-RESOLUTION-014: Navigation rejects config option not applicable to operation

Entry:
- `crates/shared/navigation/src/tests/navigation/native_options/validation.rs > navigation_rejects_config_option_not_applicable_to_operation`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation input resolution 保持来源解析边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `navigation_rejects_config_option_not_applicable_to_operation` 直接验证“Navigation rejects config option not applicable to operation”所描述的结果。
