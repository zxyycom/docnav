### Case WB-NAVIGATION-CONFIG-SOURCES-004: Explicit missing config source is blocking diagnostic

Entry:
- `crates/shared/navigation/src/tests/navigation/config_sources.rs > explicit_missing_config_source_is_blocking_diagnostic`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation loads config sources with descriptor origin”所涉及的稳定行为边界。

Proves:
- 原生入口 `explicit_missing_config_source_is_blocking_diagnostic` 直接验证“Explicit missing config source is blocking diagnostic”所描述的结果。
