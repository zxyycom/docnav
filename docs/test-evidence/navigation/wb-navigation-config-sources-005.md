### Case WB-NAVIGATION-CONFIG-SOURCES-005: Override missing config source preserves override diagnostic

Entry:
- `crates/shared/navigation/src/tests/navigation/config_sources.rs > override_missing_config_source_preserves_override_diagnostic`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation loads config sources with descriptor origin”所涉及的稳定行为边界。

Proves:
- 原生入口 `override_missing_config_source_preserves_override_diagnostic` 直接验证“Override missing config source preserves override diagnostic”所描述的结果。
