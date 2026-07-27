### Case WB-CORE-INVOCATION-LOG-005: Invocation log config type error is blocking core config error

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/config.rs > invocation_log_config_type_error_is_blocking_core_config_error`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_log_config_type_error_is_blocking_core_config_error` 直接验证“Invocation log config type error is blocking core config error”所描述的结果。
