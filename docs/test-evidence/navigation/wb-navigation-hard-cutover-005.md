### Case WB-NAVIGATION-HARD-CUTOVER-005: Valid explicit native value does not hide invalid user config

Entry:
- `crates/shared/navigation/src/tests/navigation/hard_cutover.rs > valid_explicit_native_value_does_not_hide_invalid_user_config`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Core catalog cutover preserves resolver parity”所涉及的稳定行为边界。

Proves:
- 原生入口 `valid_explicit_native_value_does_not_hide_invalid_user_config` 直接验证“Valid explicit native value does not hide invalid user config”所描述的结果。
