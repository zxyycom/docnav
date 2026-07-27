### Case WB-NAVIGATION-CONFIG-SOURCES-010: Explicit config path selection preserves parameter priority

Entry:
- `crates/shared/navigation/src/tests/navigation/config_sources.rs > explicit_config_path_selection_preserves_parameter_priority`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation loads config sources with descriptor origin”所涉及的稳定行为边界。

Proves:
- 原生入口 `explicit_config_path_selection_preserves_parameter_priority` 直接验证“Explicit config path selection preserves parameter priority”所描述的结果。
