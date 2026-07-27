### Case WB-NAVIGATION-CONFIG-SOURCES-011: Navigation rejects nested non object config shapes

Entry:
- `crates/shared/navigation/src/tests/navigation/config_sources.rs > navigation_rejects_nested_non_object_config_shapes`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation loads config sources with descriptor origin”所涉及的稳定行为边界。

Proves:
- 原生入口 `navigation_rejects_nested_non_object_config_shapes` 直接验证“Navigation rejects nested non object config shapes”所描述的结果。
