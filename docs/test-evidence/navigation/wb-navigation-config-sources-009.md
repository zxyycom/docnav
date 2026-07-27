### Case WB-NAVIGATION-CONFIG-SOURCES-009: Explicit default config value diagnostics preserve selected source path

Entry:
- `crates/shared/navigation/src/tests/navigation/config_sources.rs > explicit_default_config_value_diagnostics_preserve_selected_source_path`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation loads config sources with descriptor origin”所涉及的稳定行为边界。

Proves:
- 原生入口 `explicit_default_config_value_diagnostics_preserve_selected_source_path` 直接验证“Explicit default config value diagnostics preserve selected source path”所描述的结果。
