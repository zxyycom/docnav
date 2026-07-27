### Case WB-CORE-CONFIG-SOURCE-010: Default missing config path is absent

Entry:
- `crates/docnav/src/config/store/tests.rs > default_missing_config_path_is_absent`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Core config source validation preserves navigation-owned fields”所涉及的稳定行为边界。

Proves:
- 原生入口 `default_missing_config_path_is_absent` 直接验证“Default missing config path is absent”所描述的结果。
