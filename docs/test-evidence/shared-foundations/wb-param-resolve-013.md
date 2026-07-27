### Case WB-PARAM-RESOLVE-013: Missing required value returns no partial values

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/resolution/missing.rs > missing_required_value_returns_no_partial_values`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical resolution preserves one ordered merge chain”所涉及的稳定行为边界。

Proves:
- 原生入口 `missing_required_value_returns_no_partial_values` 直接验证“Missing required value returns no partial values”所描述的结果。
