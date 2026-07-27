### Case WB-NAV-INPUT-RESOLUTION-002: Navigation resolves selected catalog option and dispatches

Entry:
- `crates/shared/navigation/src/tests/navigation/native_options/resolution.rs > navigation_resolves_selected_catalog_option_and_dispatches`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation input resolution 保持来源解析边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `navigation_resolves_selected_catalog_option_and_dispatches` 直接验证“Navigation resolves selected catalog option and dispatches”所描述的结果。
