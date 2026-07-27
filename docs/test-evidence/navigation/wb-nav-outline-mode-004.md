### Case WB-NAV-OUTLINE-MODE-004: Threshold filtering and unit merge keep structured when minimum not met

Entry:
- `crates/shared/navigation/src/tests/navigation/outline_mode.rs > threshold_filtering_and_unit_merge_keep_structured_when_minimum_not_met`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation outline_mode selectors and pre-dispatch stable”所涉及的稳定行为边界。

Proves:
- 原生入口 `threshold_filtering_and_unit_merge_keep_structured_when_minimum_not_met` 直接验证“Threshold filtering and unit merge keep structured when minimum not met”所描述的结果。
