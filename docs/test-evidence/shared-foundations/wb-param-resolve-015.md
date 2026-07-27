### Case WB-PARAM-RESOLVE-015: Later source wins at equal priority

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/resolution/precedence.rs > later_source_wins_at_equal_priority`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical resolution preserves one ordered merge chain”所涉及的稳定行为边界。

Proves:
- 原生入口 `later_source_wins_at_equal_priority` 直接验证“Later source wins at equal priority”所描述的结果。
