### Case WB-NAVIGATION-CONFIG-SOURCES-003: Default missing config sources are absent without diagnostics

Entry:
- `crates/shared/navigation/src/tests/navigation/config_sources.rs > default_missing_config_sources_are_absent_without_diagnostics`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation loads config sources with descriptor origin”所涉及的稳定行为边界。

Proves:
- 原生入口 `default_missing_config_sources_are_absent_without_diagnostics` 直接验证“Default missing config sources are absent without diagnostics”所描述的结果。
