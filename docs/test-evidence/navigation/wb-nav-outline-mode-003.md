### Case WB-NAV-OUTLINE-MODE-003: Later structured path rule opts out of cost threshold

Entry:
- `crates/shared/navigation/src/tests/navigation/outline_mode.rs > later_structured_path_rule_opts_out_of_cost_threshold`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation outline_mode selectors and pre-dispatch stable”所涉及的稳定行为边界。

Proves:
- 原生入口 `later_structured_path_rule_opts_out_of_cost_threshold` 直接验证“Later structured path rule opts out of cost threshold”所描述的结果。
