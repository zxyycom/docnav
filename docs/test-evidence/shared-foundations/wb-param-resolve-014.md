### Case WB-PARAM-RESOLVE-014: Higher priority source wins

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/resolution/precedence.rs > higher_priority_source_wins`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical resolution preserves one ordered merge chain”所涉及的稳定行为边界。

Proves:
- 原生入口 `higher_priority_source_wins` 直接验证“Higher priority source wins”所描述的结果。
