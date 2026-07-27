### Case WB-CORE-CONFIG-SOURCE-004: Bare native option config path is unknown

Entry:
- `crates/docnav/src/config/store/tests.rs > bare_native_option_config_path_is_unknown`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Core config source validation preserves navigation-owned fields”所涉及的稳定行为边界。

Proves:
- 原生入口 `bare_native_option_config_path_is_unknown` 直接验证“Bare native option config path is unknown”所描述的结果。
