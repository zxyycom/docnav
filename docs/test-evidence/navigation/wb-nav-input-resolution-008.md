### Case WB-NAV-INPUT-RESOLUTION-008: Navigation blocks dispatch when native option type cannot materialize

Entry:
- `crates/shared/navigation/src/tests/navigation/native_options/validation.rs > navigation_blocks_dispatch_when_native_option_type_cannot_materialize`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation input resolution 保持来源解析边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `navigation_blocks_dispatch_when_native_option_type_cannot_materialize` 直接验证“Navigation blocks dispatch when native option type cannot materialize”所描述的结果。
