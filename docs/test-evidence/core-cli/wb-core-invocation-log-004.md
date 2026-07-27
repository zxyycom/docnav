### Case WB-CORE-INVOCATION-LOG-004: Invocation cli content root without cli log does not override config log

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/config.rs > invocation_cli_content_root_without_cli_log_does_not_override_config_log`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_cli_content_root_without_cli_log_does_not_override_config_log` 直接验证“Invocation cli content root without cli log does not override config log”所描述的结果。
