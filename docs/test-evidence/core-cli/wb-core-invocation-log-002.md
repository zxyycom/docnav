### Case WB-CORE-INVOCATION-LOG-002: Invocation logging disabled creates no log side effect

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/config.rs > invocation_logging_disabled_creates_no_log_side_effect`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_logging_disabled_creates_no_log_side_effect` 直接验证“Invocation logging disabled creates no log side effect”所描述的结果。
