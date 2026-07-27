### Case WB-NAV-INPUT-RESOLUTION-010: Navigation blocks invalid catalog value for other known adapter

Entry:
- `crates/shared/navigation/src/tests/navigation/native_options/validation.rs > navigation_blocks_invalid_catalog_value_for_other_known_adapter`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation input resolution 保持来源解析边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `navigation_blocks_invalid_catalog_value_for_other_known_adapter` 直接验证“Navigation blocks invalid catalog value for other known adapter”所描述的结果。
