### Case WB-CORE-CONFIG-SOURCE-008: Direct config file rejects empty invocation log content capture root

Entry:
- `crates/docnav/src/config/store/tests.rs > direct_config_file_rejects_empty_invocation_log_content_capture_root`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Core config source validation preserves navigation-owned fields”所涉及的稳定行为边界。

Proves:
- 原生入口 `direct_config_file_rejects_empty_invocation_log_content_capture_root` 直接验证“Direct config file rejects empty invocation log content capture root”所描述的结果。
