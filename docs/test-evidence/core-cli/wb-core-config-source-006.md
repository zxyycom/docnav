### Case WB-CORE-CONFIG-SOURCE-006: Navigation owned outline config is accepted

Entry:
- `crates/docnav/src/config/store/tests.rs > navigation_owned_outline_config_is_accepted`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Core config source validation preserves navigation-owned fields”所涉及的稳定行为边界。

Proves:
- 原生入口 `navigation_owned_outline_config_is_accepted` 直接验证“Navigation owned outline config is accepted”所描述的结果。
