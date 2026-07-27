### Case WB-PARAM-RESOLVE-008: Append applies canonical constraints only after merging contributors

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/resolution/merge.rs > append_applies_canonical_constraints_only_after_merging_contributors`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical resolution preserves one ordered merge chain”所涉及的稳定行为边界。

Proves:
- 原生入口 `append_applies_canonical_constraints_only_after_merging_contributors` 直接验证“Append applies canonical constraints only after merging contributors”所描述的结果。
