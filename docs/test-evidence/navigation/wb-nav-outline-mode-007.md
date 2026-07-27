### Case WB-NAV-OUTLINE-MODE-007: Cost threshold triggers hook full read and preserves selector cost

Entry:
- `crates/shared/navigation/src/tests/navigation/outline_mode.rs > cost_threshold_triggers_hook_full_read_and_preserves_selector_cost`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation outline_mode selectors and pre-dispatch stable”所涉及的稳定行为边界。

Proves:
- 原生入口 `cost_threshold_triggers_hook_full_read_and_preserves_selector_cost` 直接验证“Cost threshold triggers hook full read and preserves selector cost”所描述的结果。
