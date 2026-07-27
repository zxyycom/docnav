### Case WB-PARAM-RESOLVE-009: Merged value is revalidated

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/resolution/merge.rs > merged_value_is_revalidated`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical resolution preserves one ordered merge chain”所涉及的稳定行为边界。

Proves:
- 原生入口 `merged_value_is_revalidated` 直接验证“Merged value is revalidated”所描述的结果。
