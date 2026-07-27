### Case WB-NAV-OUTLINE-MODE-011: Unregistered outline config key returns source scoped diagnostic

Entry:
- `crates/shared/navigation/src/tests/navigation/outline_mode.rs > unregistered_outline_config_key_returns_source_scoped_diagnostic`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation outline_mode selectors and pre-dispatch stable”所涉及的稳定行为边界。

Proves:
- 原生入口 `unregistered_outline_config_key_returns_source_scoped_diagnostic` 直接验证“Unregistered outline config key returns source scoped diagnostic”所描述的结果。
