### Case WB-NAVIGATION-HARD-CUTOVER-003: Valid explicit common value does not hide invalid project config

Entry:
- `crates/shared/navigation/src/tests/navigation/hard_cutover.rs > valid_explicit_common_value_does_not_hide_invalid_project_config`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Core catalog cutover preserves resolver parity”所涉及的稳定行为边界。

Proves:
- 原生入口 `valid_explicit_common_value_does_not_hide_invalid_project_config` 直接验证“Valid explicit common value does not hide invalid project config”所描述的结果。
