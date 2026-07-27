### Case WB-NAVIGATION-CONFIG-SOURCES-006: Explicit invalid json config source is blocking diagnostic

Entry:
- `crates/shared/navigation/src/tests/navigation/config_sources.rs > explicit_invalid_json_config_source_is_blocking_diagnostic`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation loads config sources with descriptor origin”所涉及的稳定行为边界。

Proves:
- 原生入口 `explicit_invalid_json_config_source_is_blocking_diagnostic` 直接验证“Explicit invalid json config source is blocking diagnostic”所描述的结果。
