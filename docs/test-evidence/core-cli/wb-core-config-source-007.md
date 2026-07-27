### Case WB-CORE-CONFIG-SOURCE-007: Direct config file rejects empty invocation log path

Entry:
- `crates/docnav/src/config/store/tests.rs > direct_config_file_rejects_empty_invocation_log_path`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Core config source validation preserves navigation-owned fields”所涉及的稳定行为边界。

Proves:
- 原生入口 `direct_config_file_rejects_empty_invocation_log_path` 直接验证“Direct config file rejects empty invocation log path”所描述的结果。
