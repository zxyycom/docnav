### Case WB-CORE-INVOCATION-LOG-003: Invocation logging config enabled uses validated core config

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/config.rs > invocation_logging_config_enabled_uses_validated_core_config`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_logging_config_enabled_uses_validated_core_config` 直接验证“Invocation logging config enabled uses validated core config”所描述的结果。
