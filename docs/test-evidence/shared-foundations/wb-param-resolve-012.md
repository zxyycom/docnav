### Case WB-PARAM-RESOLVE-012: Deny conflict accepts equal values

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/resolution/merge.rs > deny_conflict_accepts_equal_values`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical resolution preserves one ordered merge chain”所涉及的稳定行为边界。

Proves:
- 原生入口 `deny_conflict_accepts_equal_values` 直接验证“Deny conflict accepts equal values”所描述的结果。
