### Case WB-CORE-INVOCATION-LOG-006: Invocation cli log records config load failure before runtime config

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/config.rs > invocation_cli_log_records_config_load_failure_before_runtime_config`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_cli_log_records_config_load_failure_before_runtime_config` 直接验证“Invocation cli log records config load failure before runtime config”所描述的结果。
