### Case WB-NAV-OUTLINE-MODE-006: Threshold missing measurement and runtime unavailable fall back to structured

Entry:
- `crates/shared/navigation/src/tests/navigation/outline_mode.rs > threshold_missing_measurement_and_runtime_unavailable_fall_back_to_structured`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation outline_mode selectors and pre-dispatch stable”所涉及的稳定行为边界。

Proves:
- 原生入口 `threshold_missing_measurement_and_runtime_unavailable_fall_back_to_structured` 直接验证“Threshold missing measurement and runtime unavailable fall back to structured”所描述的结果。
