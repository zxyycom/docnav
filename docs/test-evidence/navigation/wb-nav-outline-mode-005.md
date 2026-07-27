### Case WB-NAV-OUTLINE-MODE-005: Threshold adapter mismatch does not request cost measurements

Entry:
- `crates/shared/navigation/src/tests/navigation/outline_mode.rs > threshold_adapter_mismatch_does_not_request_cost_measurements`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation outline_mode selectors and pre-dispatch stable”所涉及的稳定行为边界。

Proves:
- 原生入口 `threshold_adapter_mismatch_does_not_request_cost_measurements` 直接验证“Threshold adapter mismatch does not request cost measurements”所描述的结果。
