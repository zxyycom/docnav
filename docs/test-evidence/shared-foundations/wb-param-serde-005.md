### Case WB-PARAM-SERDE-005: Non config locator returns a public error

Entry:
- `crates/shared/cli-config-resolution-serde/src/tests.rs > non_config_locator_returns_a_public_error`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“serde config-path mapping preserves candidate facts”所涉及的稳定行为边界。

Proves:
- 原生入口 `non_config_locator_returns_a_public_error` 直接验证“Non config locator returns a public error”所描述的结果。
